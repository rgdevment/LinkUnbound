use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, Runtime};

pub const SETTINGS: &str = "settings";
pub const PICKER: &str = "picker";

/// What the shortcut and the tray both do: there is one window to bring up, and
/// it is the one the user configures things in.
pub fn open_settings<R: Runtime>(app: &AppHandle<R>) {
    let Some(window) = app.get_webview_window(SETTINGS) else {
        return;
    };
    let _ = window.show();
    let _ = window.unminimize();
    let _ = window.set_focus();
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

    /// The labels are the contract with `tauri.conf.json`; a typo here means a
    /// window that silently never opens.
    #[test]
    fn the_window_labels_match_the_ones_that_are_declared() {
        let config = include_str!("../tauri.conf.json");
        assert!(config.contains(&format!("\"label\": \"{SETTINGS}\"")));
        assert!(config.contains(&format!("\"label\": \"{PICKER}\"")));
    }
}
