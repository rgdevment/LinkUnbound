use std::sync::atomic::{AtomicBool, Ordering};

use linkunbound_core::Theme;
use tauri::{AppHandle, Manager, Runtime};

pub const SETTINGS: &str = "settings";

#[must_use]
pub fn dressed_as(theme: Theme) -> Option<tauri::Theme> {
    match theme {
        Theme::System => None,
        Theme::Light => Some(tauri::Theme::Light),
        Theme::Dark => Some(tauri::Theme::Dark),
    }
}

#[must_use]
pub fn ground(theme: Theme) -> Option<tauri::window::Color> {
    match dressed_as(theme)? {
        tauri::Theme::Dark => Some(tauri::window::Color(25, 26, 31, 255)),
        _ => Some(tauri::window::Color(255, 255, 255, 255)),
    }
}

pub fn repaint<R: Runtime>(app: &AppHandle<R>, theme: Theme) {
    if let Some(window) = app.get_webview_window(SETTINGS) {
        let _ = window.set_theme(dressed_as(theme));
        let _ = window.set_background_color(ground(theme));
        dress_natively(&window, theme);
    }
}

/// The window is looked up again on the main thread rather than carried over
/// as a pointer: closed in between, the pointer would name freed memory.
#[cfg(target_os = "macos")]
fn dress_natively<R: Runtime>(window: &tauri::WebviewWindow<R>, theme: Theme) {
    let dark = dressed_as(theme).map(|t| t == tauri::Theme::Dark);
    let app = window.app_handle().clone();
    let _ = window.run_on_main_thread(move || {
        if let Some(handle) = app
            .get_webview_window(SETTINGS)
            .and_then(|window| window.ns_window().ok())
        {
            linkunbound_mac::dress_window(handle as isize, dark);
        }
    });
}

#[cfg(not(target_os = "macos"))]
fn dress_natively<R: Runtime>(_window: &tauri::WebviewWindow<R>, _theme: Theme) {}

#[cfg(target_os = "windows")]
fn drop_the_title_icon<R: Runtime>(window: &tauri::WebviewWindow<R>) {
    if let Ok(hwnd) = window.hwnd() {
        linkunbound_win::strip_title_icon(hwnd.0 as isize);
    }
}

#[cfg(not(target_os = "windows"))]
fn drop_the_title_icon<R: Runtime>(_window: &tauri::WebviewWindow<R>) {}

/// Set while the webview is being built. A second instance's knock, or the global shortcut,
/// arrives inside that build's own message pump — the WebView2 environment is created by
/// pumping messages until it answers — and building again from in there leaves both sides
/// waiting on each other. That is what a double click on the tray icon did: a blank window
/// that never answered, and a second process stuck knocking on it.
static BUILDING: AtomicBool = AtomicBool::new(false);

/// Lets go however the build ends: a panic that unwinds through it would otherwise leave the
/// flag up and no click could ever open the window again.
struct Building;

impl Drop for Building {
    fn drop(&mut self) {
        BUILDING.store(false, Ordering::Release);
    }
}

/// Built on demand: an idle webview costs tens of megabytes and this window is
/// opened rarely. The resident owns the tray, the picker and the notice.
pub fn open_settings<R: Runtime>(app: &AppHandle<R>, theme: Theme) {
    if let Some(window) = app.get_webview_window(SETTINGS) {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
        return;
    }
    if BUILDING.swap(true, Ordering::AcqRel) {
        return;
    }
    let _building = Building;
    build_settings(app, theme);
}

const LONG_ENOUGH_TO_PAINT: std::time::Duration = std::time::Duration::from_millis(1500);

fn build_settings<R: Runtime>(app: &AppHandle<R>, theme: Theme) {
    let mut builder = tauri::WebviewWindowBuilder::new(app, SETTINGS, tauri::WebviewUrl::default())
        .title("LinkUnbound")
        .inner_size(780.0, 560.0)
        .min_inner_size(640.0, 480.0)
        .resizable(true)
        .visible(false)
        .theme(dressed_as(theme));
    if let Some(color) = ground(theme) {
        builder = builder.background_color(color);
    }
    let built = builder.build();
    if let Ok(window) = built {
        dress_natively(&window, theme);
        drop_the_title_icon(&window);
        let late = window.clone();
        std::thread::spawn(move || {
            std::thread::sleep(LONG_ENOUGH_TO_PAINT);
            reveal(&late);
        });
    }
}

pub fn reveal<R: Runtime>(window: &tauri::WebviewWindow<R>) {
    if window.is_visible().unwrap_or(true) {
        return;
    }
    let _ = window.show();
    let _ = window.set_focus();
}

#[cfg(test)]
mod tests {
    use super::{SETTINGS, dressed_as};
    use linkunbound_core::Theme;

    #[test]
    fn the_title_bar_is_told_the_theme_and_left_alone_for_the_system() {
        assert_eq!(dressed_as(Theme::Dark), Some(tauri::Theme::Dark));
        assert_eq!(dressed_as(Theme::Light), Some(tauri::Theme::Light));
        assert_eq!(dressed_as(Theme::System), None);
    }

    /// Declaring it would pre-create its webview at startup, which is the tens
    /// of megabytes `open_settings` exists to avoid.
    #[test]
    fn settings_is_not_declared_so_it_stays_built_on_demand() {
        let config = include_str!("../tauri.conf.json");
        assert!(!config.contains(&format!("\"label\": \"{SETTINGS}\"")));
    }

    /// The picker left for the resident; a window still declared here would be
    /// a second one nobody drives.
    /// Two tray clicks would otherwise start two processes, each reconciling the
    /// registry and each claiming the global shortcut.
    #[test]
    fn only_one_settings_process_may_exist() {
        let wiring = include_str!("lib.rs");
        assert!(wiring.contains("tauri_plugin_single_instance::init"));
    }

    #[test]
    fn no_window_is_declared_at_all_any_more() {
        let config = include_str!("../tauri.conf.json");
        assert!(!config.contains("\"label\":"));
    }
}
