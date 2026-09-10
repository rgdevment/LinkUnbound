//! The one place in the workspace where `unsafe` is allowed. Every call here is
//! a Win32 entry point with no safe wrapper; the `Unsafe stays where it was
//! audited` step of `rules.yml` names this file and fails on any other.
#![allow(unsafe_code)]

use std::os::windows::ffi::OsStrExt;
use std::path::Path;

use windows::Win32::Foundation::{CloseHandle, HANDLE, MAX_PATH};
use windows::Win32::Graphics::Gdi::{
    BI_RGB, BITMAP, BITMAPINFO, BITMAPINFOHEADER, DIB_RGB_COLORS, DeleteObject, GetDC, GetDIBits,
    GetObjectW, HBITMAP, ReleaseDC,
};
use windows::Win32::System::Threading::{
    OpenProcess, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION, QueryFullProcessImageNameW,
};
use windows::Win32::UI::Shell::{SHCNE_ASSOCCHANGED, SHCNF_IDLIST, SHChangeNotify};
use windows::Win32::UI::Shell::{SHFILEINFOW, SHGFI_ICON, SHGFI_LARGEICON, SHGetFileInfoW};
use windows::Win32::UI::WindowsAndMessaging::{DestroyIcon, GetIconInfo, HICON, ICONINFO};
use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};

/// Tells the shell the association keys changed. Without it Explorer keeps
/// serving the previous default until something else invalidates its cache.
pub fn notify_associations_changed() {
    unsafe {
        SHChangeNotify(SHCNE_ASSOCCHANGED, SHCNF_IDLIST, None, None);
    }
}

fn image_path(process: HANDLE) -> Option<String> {
    let mut buffer = [0u16; MAX_PATH as usize];
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

    #[test]
    fn telling_the_shell_about_a_change_does_not_panic() {
        notify_associations_changed();
    }
}
