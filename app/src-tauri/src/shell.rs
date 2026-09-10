use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, Runtime};

pub const SETTINGS: &str = "settings";
pub const PICKER: &str = "picker";
pub const NOTICE: &str = "notice";

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

/// Hiding it is only reachable with a shortcut left; the core refuses the
/// combination that would strand the user.
pub fn show_tray<R: Runtime>(app: &AppHandle<R>, visible: bool) {
    if let Some(tray) = app.tray_by_id("main") {
        let _ = tray.set_visible(visible);
    }
}

/// Built on demand and thrown away: it exists for a few seconds and must never
/// take focus from whatever the user is doing.
pub fn flash<R: Runtime>(app: &AppHandle<R>) {
    dismiss_notice(app);
    let built = tauri::WebviewWindowBuilder::new(app, NOTICE, tauri::WebviewUrl::default())
        .title("LinkUnbound")
        .inner_size(340.0, 92.0)
        .resizable(false)
        .decorations(false)
        .transparent(true)
        .always_on_top(true)
        .skip_taskbar(true)
        .focused(false)
        .visible(false)
        .build();
    let _ = built;
}

/// The corner farthest from where work happens, on the screen the pointer is on.
pub fn show_notice<R: Runtime>(app: &AppHandle<R>) {
    let Some(window) = app.get_webview_window(NOTICE) else {
        return;
    };
    if let Ok(Some(screen)) = window.current_monitor()
        && let Ok(size) = window.outer_size()
    {
        let area = screen.size();
        let at = screen.position();
        let x = at.x + (area.width as i32) - (size.width as i32) - 24;
        let y = at.y + (area.height as i32) - (size.height as i32) - 64;
        let _ = window.set_position(tauri::PhysicalPosition::new(x, y));
    }
    let _ = window.show();
}

pub fn dismiss_notice<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window(NOTICE) {
        let _ = window.close();
    }
}

pub fn hide_picker<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window(PICKER) {
        let _ = window.hide();
    }
}

/// The taskbar never recolours what it is given, so both variants ship and the
/// right one is chosen. A tray icon is a silhouette, not the app icon shrunk.
#[cfg(windows)]
fn tray_icon() -> Option<tauri::image::Image<'static>> {
    let bytes: &[u8] = if linkunbound_win::taskbar_is_light() {
        include_bytes!("../icons/tray-light-32.png")
    } else {
        include_bytes!("../icons/tray-dark-32.png")
    };
    tauri::image::Image::from_bytes(bytes).ok()
}

#[cfg(not(windows))]
fn tray_icon() -> Option<tauri::image::Image<'static>> {
    None
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
        .icon(
            tray_icon()
                .or_else(|| app.default_window_icon().cloned())
                .ok_or_else(|| {
                    tauri::Error::AssetNotFound("the bundled tray icon is missing".to_owned())
                })?,
        )
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
