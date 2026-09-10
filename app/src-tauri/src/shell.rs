use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, Runtime};

pub const SETTINGS: &str = "settings";
pub const PICKER: &str = "picker";

/// Built on demand: an idle webview costs tens of megabytes and this window is opened rarely.
pub fn open_settings<R: Runtime>(app: &AppHandle<R>) {
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
        .build();
    if let Ok(window) = built {
        let _ = window.set_focus();
    }
}

pub fn hide_picker<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window(PICKER) {
        let _ = window.hide();
    }
}

/// The tray is the only proof the app is running: it lives in the background
/// with no window of its own, so without it there is no way to reach settings
/// or to tell it is alive at all.
pub fn install_tray<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    let settings = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let menu = Menu::with_items(app, &[&settings, &separator, &quit])?;

    TrayIconBuilder::with_id("main")
        .icon(app.default_window_icon().cloned().ok_or_else(|| {
            tauri::Error::AssetNotFound("the bundled window icon is missing".to_owned())
        })?)
        .tooltip("LinkUnbound")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "settings" => open_settings(app),
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                open_settings(tray.app_handle());
            }
        })
        .build(app)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{PICKER, SETTINGS};

    /// The label is the contract with `tauri.conf.json`; a typo means a window
    /// that silently never opens.
    #[test]
    fn the_picker_is_declared_under_the_label_the_code_asks_for() {
        let config = include_str!("../tauri.conf.json");
        assert!(config.contains(&format!("\"label\": \"{PICKER}\"")));
    }

    /// Declaring it would pre-create its webview at startup, which is the tens
    /// of megabytes `open_settings` exists to avoid.
    #[test]
    fn settings_is_not_declared_so_it_stays_built_on_demand() {
        let config = include_str!("../tauri.conf.json");
        assert!(!config.contains(&format!("\"label\": \"{SETTINGS}\"")));
    }
}
