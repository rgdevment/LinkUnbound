use linkunbound_core::Theme;
use tauri::{AppHandle, Manager, Runtime};

pub const SETTINGS: &str = "settings";

/// The title bar is the system's, and it only follows the page when told which way.
#[must_use]
pub fn dressed_as(theme: Theme) -> Option<tauri::Theme> {
    match theme {
        Theme::System => None,
        Theme::Light => Some(tauri::Theme::Light),
        Theme::Dark => Some(tauri::Theme::Dark),
    }
}

pub fn repaint<R: Runtime>(app: &AppHandle<R>, theme: Theme) {
    if let Some(window) = app.get_webview_window(SETTINGS) {
        let _ = window.set_theme(dressed_as(theme));
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

    let built = tauri::WebviewWindowBuilder::new(app, SETTINGS, tauri::WebviewUrl::default())
        .title("LinkUnbound")
        .inner_size(780.0, 560.0)
        .min_inner_size(640.0, 480.0)
        .resizable(true)
        .theme(dressed_as(theme))
        .build();
    if let Ok(window) = built {
        let _ = window.set_focus();
    }
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
