//! The one place in the workspace where `unsafe` is allowed. Every call here is
//! a Win32 entry point with no safe wrapper; `rules.yml` fails the build if the
//! allowance appears in any other file.
#![allow(unsafe_code)]

use std::path::Path;

use windows::Win32::Foundation::{CloseHandle, HANDLE, MAX_PATH};
use windows::Win32::System::Threading::{
    OpenProcess, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION, QueryFullProcessImageNameW,
};
use windows::Win32::UI::Shell::{SHCNE_ASSOCCHANGED, SHCNF_IDLIST, SHChangeNotify};
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
