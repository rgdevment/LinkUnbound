use std::sync::atomic::{AtomicBool, Ordering};

use linkunbound_core::Theme;
use tauri::{AppHandle, Manager, Runtime};

pub const SETTINGS: &str = "settings";
pub const WELCOME: &str = "welcome";

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
    for label in [SETTINGS, WELCOME] {
        if let Some(window) = app.get_webview_window(label) {
            let _ = window.set_theme(dressed_as(theme));
            let _ = window.set_background_color(ground(theme));
            dress_natively(&window, theme);
        }
    }
}

/// The window is looked up again on the main thread rather than carried over
/// as a pointer: closed in between, the pointer would name freed memory.
#[cfg(target_os = "macos")]
fn dress_natively<R: Runtime>(window: &tauri::WebviewWindow<R>, theme: Theme) {
    let dark = dressed_as(theme).map(|t| t == tauri::Theme::Dark);
    let app = window.app_handle().clone();
    let label = window.label().to_owned();
    let _ = window.run_on_main_thread(move || {
        if let Some(handle) = app
            .get_webview_window(&label)
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
/// The page the command line asks settings to open on, when it asks for one.
#[must_use]
pub fn page_in(args: &[String]) -> Option<&'static str> {
    args.iter()
        .any(|a| a == linkunbound_core::BROWSERS)
        .then_some("browsers")
}

pub fn open_settings<R: Runtime>(app: &AppHandle<R>, theme: Theme, page: Option<&str>) {
    if let Some(window) = app.get_webview_window(SETTINGS) {
        if let Some(page) = page {
            let _ = window.eval(format!("location.hash = '{page}'"));
        }
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
        return;
    }
    if BUILDING.swap(true, Ordering::AcqRel) {
        return;
    }
    let _building = Building;
    build_settings(app, theme, page);
}

const LONG_ENOUGH_TO_PAINT: std::time::Duration = std::time::Duration::from_millis(1500);

fn build_settings<R: Runtime>(app: &AppHandle<R>, theme: Theme, page: Option<&str>) {
    let url = page.map_or_else(tauri::WebviewUrl::default, |page| {
        tauri::WebviewUrl::App(format!("index.html#{page}").into())
    });
    let builder = hidden(app, SETTINGS, url, theme)
        .inner_size(780.0, 560.0)
        .min_inner_size(640.0, 480.0)
        .resizable(true);
    if let Ok(window) = builder.build() {
        dressed(&window, theme);
    }
}

/// Over settings, the first time it opens while links still go elsewhere, and again whenever
/// the person asks for it there.
pub fn open_welcome<R: Runtime>(app: &AppHandle<R>, theme: Theme) {
    if let Some(window) = app.get_webview_window(WELCOME) {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
        return;
    }
    let url = tauri::WebviewUrl::App("index.html#welcome".into());
    let builder = hidden(app, WELCOME, url, theme)
        .inner_size(600.0, 460.0)
        .resizable(false)
        .maximizable(false)
        .center();
    if let Ok(window) = builder.build() {
        dressed(&window, theme);
    }
}

/// Shown only once the page has painted, so the window never flashes blank.
fn hidden<'a, R: Runtime, M: Manager<R>>(
    app: &'a M,
    label: &str,
    url: tauri::WebviewUrl,
    theme: Theme,
) -> tauri::WebviewWindowBuilder<'a, R, M> {
    let builder = tauri::WebviewWindowBuilder::new(app, label, url)
        .title("LinkUnbound")
        .visible(false)
        .theme(dressed_as(theme));
    match ground(theme) {
        Some(color) => builder.background_color(color),
        None => builder,
    }
}

fn dressed<R: Runtime>(window: &tauri::WebviewWindow<R>, theme: Theme) {
    dress_natively(window, theme);
    drop_the_title_icon(window);
    let late = window.clone();
    std::thread::spawn(move || {
        std::thread::sleep(LONG_ENOUGH_TO_PAINT);
        reveal(&late);
    });
}

#[tauri::command]
pub fn settings_painted(window: tauri::WebviewWindow) {
    reveal(&window);
}

pub fn reveal<R: Runtime>(window: &tauri::WebviewWindow<R>) {
    if window.is_visible().unwrap_or(true) {
        return;
    }
    let _ = window.show();
    let _ = window.set_focus();
    // Settings may paint after the welcome, and must not cover it when it does.
    if window.label() == SETTINGS
        && let Some(welcome) = window.app_handle().get_webview_window(WELCOME)
        && welcome.is_visible().unwrap_or(false)
    {
        let _ = welcome.set_focus();
    }
}

#[cfg(test)]
mod tests {
    use super::{SETTINGS, dressed_as, page_in};

    #[test]
    fn the_picker_can_send_settings_to_the_browsers() {
        let said = |args: &[&str]| args.iter().map(|&a| a.to_owned()).collect::<Vec<_>>();
        assert_eq!(
            page_in(&said(&["settings.exe", "--browsers"])),
            Some("browsers")
        );
        assert_eq!(page_in(&said(&["settings.exe"])), None);
    }
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

/// The picker over a sample link, so a look can be seen before it is kept. A resident already
/// running takes the word over its socket; with none, one is started with it.
#[tauri::command]
pub fn picker_preview() {
    if linkunbound_shell::single::hand_over(linkunbound_core::PREVIEW) {
        return;
    }
    let Ok(here) = std::env::current_exe() else {
        return;
    };
    let _ = linkunbound_core::spawn_and_forget(
        std::process::Command::new(linkunbound_core::link_handler(&here))
            .arg(linkunbound_core::PREVIEW),
    );
}
