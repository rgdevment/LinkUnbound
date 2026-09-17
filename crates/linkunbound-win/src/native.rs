//! The one place in the workspace where `unsafe` is allowed. Every call here is
//! a Win32 entry point with no safe wrapper; the `Unsafe stays where it was
//! audited` step of `rules.yml` names this file and fails on any other.
#![allow(unsafe_code)]

use std::os::windows::ffi::OsStrExt;
use std::path::Path;

use windows::Win32::Foundation::{
    CloseHandle, ERROR_PIPE_BUSY, GENERIC_WRITE, GlobalFree, HANDLE, HLOCAL, HWND, LocalFree, POINT,
};
use windows::Win32::Graphics::Gdi::{
    BI_RGB, BITMAP, BITMAPINFO, BITMAPINFOHEADER, DIB_RGB_COLORS, DeleteObject, GetDC, GetDIBits,
    GetMonitorInfoW, GetObjectW, HBITMAP, HMONITOR, MONITOR_DEFAULTTONEAREST, MONITORINFO,
    MonitorFromPoint, ReleaseDC,
};
use windows::Win32::Security::Authorization::ConvertSidToStringSidW;
use windows::Win32::Security::{GetTokenInformation, TOKEN_QUERY, TOKEN_USER, TokenUser};
use windows::Win32::Storage::FileSystem::{
    CreateFileW, FILE_SHARE_NONE, FlushFileBuffers, OPEN_EXISTING, SECURITY_IDENTIFICATION,
    SECURITY_SQOS_PRESENT, WriteFile,
};
use windows::Win32::System::Console::{ATTACH_PARENT_PROCESS, AttachConsole};
use windows::Win32::System::DataExchange::{
    CloseClipboard, EmptyClipboard, OpenClipboard, SetClipboardData,
};
use windows::Win32::System::Memory::{GMEM_MOVEABLE, GlobalAlloc, GlobalLock, GlobalUnlock};
use windows::Win32::System::Ole::CF_UNICODETEXT;
use windows::Win32::System::Pipes::WaitNamedPipeW;
use windows::Win32::System::Threading::{
    AttachThreadInput, GetCurrentProcess, GetCurrentThreadId, OpenProcess, OpenProcessToken,
    PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION, QueryFullProcessImageNameW,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{GetKeyState, SetFocus, VK_SHIFT, VkKeyScanW};
use windows::Win32::UI::Shell::{SHCNE_ASSOCCHANGED, SHCNF_IDLIST, SHChangeNotify};
use windows::Win32::UI::Shell::{SHFILEINFOW, SHGFI_ICON, SHGFI_LARGEICON, SHGetFileInfoW};
use windows::Win32::UI::WindowsAndMessaging::{
    ASFW_ANY, AllowSetForegroundWindow, GWL_EXSTYLE, GetCursorPos, GetForegroundWindow,
    GetWindowLongPtrW, GetWindowThreadProcessId, HWND_TOPMOST, SWP_NOMOVE, SWP_NOSIZE,
    SetForegroundWindow, SetWindowLongPtrW, SetWindowPos, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
};
use windows::Win32::UI::WindowsAndMessaging::{DestroyIcon, GetIconInfo, HICON, ICONINFO};

/// A windowed program has no console of its own; this borrows the one it was started from, when
/// there is one, so `cargo tauri dev` still reads what it prints and a double click in Explorer
/// opens nothing. Fails, harmlessly, when nothing started it from a console.
pub fn attach_parent_console() -> bool {
    unsafe { AttachConsole(ATTACH_PARENT_PROCESS) }.is_ok()
}

/// Tells the shell the association keys changed. Without it Explorer keeps
/// serving the previous default until something else invalidates its cache.
pub fn notify_associations_changed() {
    unsafe {
        SHChangeNotify(SHCNE_ASSOCCHANGED, SHCNF_IDLIST, None, None);
    }
}

#[must_use]
pub fn cursor() -> Option<(i32, i32)> {
    let mut point = POINT::default();
    unsafe { GetCursorPos(&mut point) }.ok()?;
    Some((point.x, point.y))
}

/// The work area, not the screen: otherwise the picker lands under the taskbar.
#[must_use]
pub fn work_area_at(x: i32, y: i32) -> Option<(i32, i32, i32, i32)> {
    let monitor: HMONITOR = unsafe { MonitorFromPoint(POINT { x, y }, MONITOR_DEFAULTTONEAREST) };
    let mut info = MONITORINFO {
        cbSize: u32::try_from(std::mem::size_of::<MONITORINFO>()).ok()?,
        ..Default::default()
    };
    if !unsafe { GetMonitorInfoW(monitor, &mut info) }.as_bool() {
        return None;
    }
    let area = info.rcWork;
    Some((
        area.left,
        area.top,
        area.right - area.left,
        area.bottom - area.top,
    ))
}

/// A tool window is kept out of the taskbar and the alt-tab list. The picker is
/// summoned by a click and dismissed by one: an entry standing there outlives
/// the window it names, with no icon of its own to show.
pub fn keep_off_the_taskbar(window: isize) {
    let hwnd = HWND(window as *mut std::ffi::c_void);
    let style = unsafe { GetWindowLongPtrW(hwnd, GWL_EXSTYLE) };
    unsafe {
        SetWindowLongPtrW(hwnd, GWL_EXSTYLE, style | WS_EX_TOOLWINDOW.0 as isize);
    }
    let _ = unsafe {
        SetWindowPos(
            hwnd,
            Some(HWND_TOPMOST),
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE,
        )
    };
}

/// The digit a key would type unshifted. Slint hands over the character the
/// layout produced, so `Shift+1` arrives as `!` on a Spanish keyboard, `@` on a
/// US one, and so on: without asking Windows which key that was, holding Shift
/// silently broke the very shortcut it is meant to modify.
#[must_use]
pub fn digit_behind(typed: char) -> Option<u32> {
    let scan = unsafe { VkKeyScanW(typed as u16) };
    if scan == -1 {
        return None;
    }
    let virtual_key = u32::from(u16::try_from(scan).ok()? & 0x00ff);
    // VK_1 through VK_9 share their codes with the ASCII digits.
    (0x31..=0x39)
        .contains(&virtual_key)
        .then(|| virtual_key - 0x30)
}

#[must_use]
pub fn is_in_front(window: isize) -> bool {
    unsafe { GetForegroundWindow() }.0 as isize == window
}

/// Whether the window in front is one of this process's own — the notice, say — or the settings
/// window the picker itself opened, rather than somebody else's taking the picker's place.
#[must_use]
pub fn front_is_ours() -> bool {
    let front = unsafe { GetForegroundWindow() };
    let mut pid = 0u32;
    unsafe { GetWindowThreadProcessId(front, Some(&raw mut pid)) };
    if pid == std::process::id() {
        return true;
    }
    foreground_process_path().is_some_and(|path| {
        Path::new(&path)
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| {
                name.eq_ignore_ascii_case(&format!("{}.exe", linkunbound_core::SETTINGS))
            })
    })
}

/// The window in front, whoever's it is: what the picker was shown over, so that its going
/// somewhere else can be told from its staying put.
#[must_use]
pub fn front_window() -> isize {
    unsafe { GetForegroundWindow() }.0 as isize
}

/// The account this process runs as, spelled the way a security descriptor string reads it.
#[must_use]
pub fn current_user_sid() -> Option<String> {
    let mut token = HANDLE::default();
    unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &raw mut token) }.ok()?;
    let mut needed = 0u32;
    let _ = unsafe { GetTokenInformation(token, TokenUser, None, 0, &raw mut needed) };
    let mut buffer = vec![0u8; needed as usize];
    let read = unsafe {
        GetTokenInformation(
            token,
            TokenUser,
            Some(buffer.as_mut_ptr().cast()),
            needed,
            &raw mut needed,
        )
    };
    unsafe {
        let _ = CloseHandle(token);
    }
    read.ok()?;
    // SAFETY: Windows filled the buffer with a TOKEN_USER of the size it asked for.
    let user = unsafe { &*buffer.as_ptr().cast::<TOKEN_USER>() };
    let mut spelled = windows::core::PWSTR::null();
    unsafe { ConvertSidToStringSidW(user.User.Sid, &raw mut spelled) }.ok()?;
    let sid = unsafe { spelled.to_string() }.ok();
    unsafe {
        let _ = LocalFree(Some(HLOCAL(spelled.0.cast())));
    }
    sid
}

/// One line down a named pipe, opened for identification only: a server squatting the name
/// before the resident took it would otherwise be handed this process's token to impersonate.
/// True when the line was written; false when nobody answers at that name.
#[must_use]
pub fn write_line_to_pipe(path: &str, line: &str) -> bool {
    let wide: Vec<u16> = path.encode_utf16().chain(std::iter::once(0)).collect();
    let open = || unsafe {
        CreateFileW(
            windows::core::PCWSTR(wide.as_ptr()),
            GENERIC_WRITE.0,
            FILE_SHARE_NONE,
            None,
            OPEN_EXISTING,
            SECURITY_SQOS_PRESENT | SECURITY_IDENTIFICATION,
            None,
        )
    };
    let handle = match open() {
        Ok(handle) => handle,
        Err(busy) if busy.code() == ERROR_PIPE_BUSY.to_hresult() => {
            if !unsafe { WaitNamedPipeW(windows::core::PCWSTR(wide.as_ptr()), 2_000) }.as_bool() {
                return false;
            }
            match open() {
                Ok(handle) => handle,
                Err(_) => return false,
            }
        }
        Err(_) => return false,
    };
    let body = format!("{line}\n");
    let mut written = 0u32;
    let wrote = unsafe { WriteFile(handle, Some(body.as_bytes()), Some(&raw mut written), None) };
    // Closing right after the write is a race the resident loses: a client gone before the
    // server accepted reads as a dead connection, and the line goes with it. The flush waits
    // until the other end has read.
    unsafe {
        let _ = FlushFileBuffers(handle);
        let _ = CloseHandle(handle);
    }
    wrote.is_ok() && written as usize == body.len()
}

/// A window that is shown but never takes the activation: what is being typed keeps going where
/// it went.
pub fn never_activates(window: isize) {
    let hwnd = HWND(window as *mut std::ffi::c_void);
    let style = unsafe { GetWindowLongPtrW(hwnd, GWL_EXSTYLE) };
    unsafe {
        SetWindowLongPtrW(hwnd, GWL_EXSTYLE, style | WS_EX_NOACTIVATE.0 as isize);
    }
}

/// Windows refuses `SetForegroundWindow` to a process that is not already in
/// front. Attaching to the input queue of the window that is lifts the refusal;
/// this is the handshake every launcher performs.
pub fn take_the_keyboard(window: isize) {
    let hwnd = HWND(window as *mut std::ffi::c_void);
    let front = unsafe { GetForegroundWindow() };
    let ours = unsafe { GetCurrentThreadId() };
    let theirs = unsafe { GetWindowThreadProcessId(front, None) };

    if theirs != 0 && theirs != ours {
        let _ = unsafe { AttachThreadInput(ours, theirs, true) };
        let _ = unsafe { SetForegroundWindow(hwnd) };
        let _ = unsafe { SetFocus(Some(hwnd)) };
        let _ = unsafe { AttachThreadInput(ours, theirs, false) };
    } else {
        let _ = unsafe { SetForegroundWindow(hwnd) };
        let _ = unsafe { SetFocus(Some(hwnd)) };
    }
}

/// Asked at the moment of the click rather than remembered from a key event: a modifier held
/// down while the pointer does the work may never produce one.
#[must_use]
pub fn shift_is_down() -> bool {
    // The high bit is "down now"; the low one is the toggle, which for Shift is Caps Lock.
    (unsafe { GetKeyState(i32::from(VK_SHIFT.0)) } & (1 << 15)) != 0
}

/// Windows only lets the window that already holds the foreground give it away. The browser is
/// usually running already, so the copy we start just forwards the URL and dies — and the window
/// that does have it is a process that was never granted the right to come forward.
pub fn let_whoever_opens_next_come_forward() {
    let _ = unsafe { AllowSetForegroundWindow(ASFW_ANY) };
}

/// The clipboard is a global the whole desktop shares: it has to be opened,
/// emptied and closed, and the buffer handed over stops being ours the moment
/// `SetClipboardData` accepts it.
pub fn copy_text(text: &str) -> bool {
    let wide: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
    let bytes = std::mem::size_of_val(wide.as_slice());

    unsafe {
        if OpenClipboard(None).is_err() {
            return false;
        }
        let _ = EmptyClipboard();

        let Ok(handle) = GlobalAlloc(GMEM_MOVEABLE, bytes) else {
            let _ = CloseClipboard();
            return false;
        };
        let target = GlobalLock(handle);
        if target.is_null() {
            let _ = GlobalFree(Some(handle));
            let _ = CloseClipboard();
            return false;
        }
        std::ptr::copy_nonoverlapping(wide.as_ptr(), target.cast::<u16>(), wide.len());
        let _ = GlobalUnlock(handle);

        let given = SetClipboardData(CF_UNICODETEXT.0.into(), Some(HANDLE(handle.0)));
        if given.is_err() {
            let _ = GlobalFree(Some(handle));
            let _ = CloseClipboard();
            return false;
        }
        let _ = CloseClipboard();
        true
    }
}

/// Room for a long path: a program installed past `MAX_PATH` answered with nothing, and the
/// picker then knew no origin for a click made in it.
const LONG_PATH: usize = 32_768;

fn image_path(process: HANDLE) -> Option<String> {
    let mut buffer = vec![0u16; LONG_PATH];
    let mut size = buffer.len() as u32;
    let ok = unsafe {
        QueryFullProcessImageNameW(
            process,
            PROCESS_NAME_WIN32,
            windows::core::PWSTR(buffer.as_mut_ptr()),
            &mut size,
        )
    };
    ok.ok()?;
    String::from_utf16(&buffer[..size as usize]).ok()
}

/// The executable owning the foreground window, which is the application the
/// link was clicked in. Must be read before the picker is shown, or the answer
/// is the picker itself.
#[must_use]
pub fn foreground_process_path() -> Option<String> {
    let window = unsafe { GetForegroundWindow() };
    if window.0.is_null() {
        return None;
    }

    let mut pid = 0u32;
    unsafe { GetWindowThreadProcessId(window, Some(&mut pid)) };
    if pid == 0 {
        return None;
    }

    let process = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) }.ok()?;
    let path = image_path(process);
    unsafe {
        let _ = CloseHandle(process);
    }
    path
}

/// Rules key off a stable name, not a full path that changes with every update.
#[must_use]
pub fn source_app() -> Option<String> {
    let path = foreground_process_path()?;
    let stem = Path::new(&path).file_stem()?.to_str()?.to_ascii_lowercase();
    (!stem.is_empty()).then_some(stem)
}

/// The pixels behind a bitmap, as RGBA. Windows hands them back bottom-up and
/// in BGRA, so both are undone here rather than by every caller.
fn bitmap_pixels(handle: HBITMAP) -> Option<(u32, u32, Vec<u8>)> {
    let mut bitmap = BITMAP::default();
    let wrote = unsafe {
        GetObjectW(
            handle.into(),
            i32::try_from(size_of::<BITMAP>()).ok()?,
            Some(std::ptr::from_mut(&mut bitmap).cast()),
        )
    };
    if wrote == 0 || bitmap.bmWidth <= 0 || bitmap.bmHeight <= 0 {
        return None;
    }

    let width = u32::try_from(bitmap.bmWidth).ok()?;
    let height = u32::try_from(bitmap.bmHeight).ok()?;
    let count = width.checked_mul(height)?.checked_mul(4)?;
    let mut header = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: u32::try_from(size_of::<BITMAPINFOHEADER>()).ok()?,
            biWidth: bitmap.bmWidth,
            // Negative height asks for top-down rows; DIBs are bottom-up otherwise.
            biHeight: -bitmap.bmHeight,
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0,
            ..Default::default()
        },
        ..Default::default()
    };

    let mut pixels = vec![0u8; count as usize];
    let screen = unsafe { GetDC(None) };
    if screen.is_invalid() {
        return None;
    }
    let rows = unsafe {
        GetDIBits(
            screen,
            handle,
            0,
            height,
            Some(pixels.as_mut_ptr().cast()),
            &mut header,
            DIB_RGB_COLORS,
        )
    };
    unsafe {
        ReleaseDC(None, screen);
    }
    if rows == 0 {
        return None;
    }

    let (quads, _) = pixels.as_chunks_mut::<4>();
    for px in quads {
        px.swap(0, 2);
    }
    Some((width, height, pixels))
}

fn icon_pixels(icon: HICON) -> Option<(u32, u32, Vec<u8>)> {
    let mut info = ICONINFO::default();
    unsafe { GetIconInfo(icon, &mut info) }.ok()?;
    let pixels = bitmap_pixels(info.hbmColor);
    unsafe {
        let _ = DeleteObject(info.hbmColor.into());
        let _ = DeleteObject(info.hbmMask.into());
    }
    pixels
}

/// Asks the shell for the icon at a chosen size, which reaches the high
/// resolution variants inside the executable. `SHGetFileInfoW` only ever returns
/// the 32 px one, and shrinking that into a list row loses the detail.
fn shell_image(path: &Path, side: i32) -> Option<(u32, u32, Vec<u8>)> {
    use windows::Win32::Foundation::SIZE;
    use windows::Win32::System::Com::{COINIT_APARTMENTTHREADED, CoInitializeEx, CoUninitialize};
    use windows::Win32::UI::Shell::{
        IShellItemImageFactory, SHCreateItemFromParsingName, SIIGBF_BIGGERSIZEOK,
    };

    let mut wide: Vec<u16> = path.as_os_str().encode_wide().collect();
    wide.push(0);

    unsafe {
        let started = CoInitializeEx(None, COINIT_APARTMENTTHREADED).is_ok();
        let factory: Option<IShellItemImageFactory> =
            SHCreateItemFromParsingName(windows::core::PCWSTR(wide.as_ptr()), None).ok();
        let out = factory
            .and_then(|f| {
                f.GetImage(SIZE { cx: side, cy: side }, SIIGBF_BIGGERSIZEOK)
                    .ok()
            })
            .and_then(|handle| {
                let pixels = bitmap_pixels(handle);
                let _ = DeleteObject(handle.into());
                pixels
            });
        if started {
            CoUninitialize();
        }
        out
    }
}

/// The icon Explorer shows for this file, at the side asked for, as RGBA pixels.
/// `None` when the path is unreachable or carries no icon — a network drive, a
/// stripped binary.
#[must_use]
pub fn file_icon(path: &Path, side: i32) -> Option<(u32, u32, Vec<u8>)> {
    if let Some(found) = shell_image(path, side) {
        return Some(found);
    }

    let mut wide: Vec<u16> = path.as_os_str().encode_wide().collect();
    wide.push(0);

    let mut info = SHFILEINFOW::default();
    let got = unsafe {
        SHGetFileInfoW(
            windows::core::PCWSTR(wide.as_ptr()),
            windows::Win32::Storage::FileSystem::FILE_FLAGS_AND_ATTRIBUTES(0),
            Some(&mut info),
            u32::try_from(size_of::<SHFILEINFOW>()).ok()?,
            SHGFI_ICON | SHGFI_LARGEICON,
        )
    };
    if got == 0 || info.hIcon.is_invalid() {
        return None;
    }
    let pixels = icon_pixels(info.hIcon);
    unsafe {
        let _ = DestroyIcon(info.hIcon);
    }
    pixels
}

#[cfg(test)]
mod keys {
    use super::digit_behind;

    /// Slint reports the character the layout produced, so with Shift held a
    /// digit never arrives as a digit. Asking Windows which key it was is what
    /// makes Shift-to-open-privately work on any keyboard.
    #[test]
    fn a_plain_digit_is_itself() {
        for (typed, expected) in [('1', 1), ('5', 5), ('9', 9)] {
            assert_eq!(digit_behind(typed), Some(expected), "{typed}");
        }
    }

    #[test]
    fn a_letter_is_not_a_destination() {
        for typed in ['a', 'z', ' ', '0'] {
            assert_eq!(digit_behind(typed), None, "{typed}");
        }
    }

    /// Whatever this machine's layout puts on Shift+1 must resolve back to 1.
    #[test]
    fn the_shifted_face_of_a_digit_resolves_to_that_digit() {
        let shifted = ['!', '"', '@', '#', '$', '%', '&', '/', '(', ')', '='];
        let resolved: Vec<_> = shifted.iter().filter_map(|c| digit_behind(*c)).collect();
        assert!(
            !resolved.is_empty(),
            "no shifted digit resolved on this layout: the guard would be dead"
        );
        assert!(
            resolved.iter().all(|d| (1..=9).contains(d)),
            "resolved outside 1..9: {resolved:?}"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_foreground_process_is_named_by_its_stem() {
        if let Some(app) = source_app() {
            assert!(!app.is_empty());
            assert!(!app.contains('\\'));
            assert!(!app.ends_with(".exe"));
            assert_eq!(app, app.to_ascii_lowercase());
        }
    }

    /// The pipe's descriptor names this account by SID; an account that cannot be named leaves
    /// the pipe with the default descriptor, readable by every account on the machine.
    #[test]
    fn the_account_this_runs_as_can_be_named() {
        let sid = super::current_user_sid().expect("a token to read");
        assert!(sid.starts_with("S-1-5-"), "{sid}");
    }

    #[test]
    fn a_line_to_a_pipe_nobody_holds_is_refused_rather_than_waited_for() {
        assert!(!super::write_line_to_pipe(
            r"\\.\pipe\linkunbound-nobody-holds-this",
            "https://example.test/"
        ));
    }

    #[test]
    fn telling_the_shell_about_a_change_does_not_panic() {
        notify_associations_changed();
    }
}
