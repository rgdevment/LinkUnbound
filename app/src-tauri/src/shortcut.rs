use tauri::{AppHandle, Runtime};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

/// Until settings can change it, the app claims one combination. It is not
/// registered when something else already holds it: stealing a shortcut from
/// another application is worse than not having one.
pub const DEFAULT: &str = "Alt+Shift+L";

pub fn install<R: Runtime>(app: &AppHandle<R>) {
    let Ok(shortcut) = DEFAULT.parse::<Shortcut>() else {
        return;
    };
    let manager = app.global_shortcut();
    if manager.is_registered(shortcut) {
        return;
    }
    let _ = manager.on_shortcut(shortcut, |app, _shortcut, event| {
        if event.state() == ShortcutState::Pressed {
            crate::shell::open_settings(app);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::DEFAULT;
    use tauri_plugin_global_shortcut::Shortcut;

    #[test]
    fn the_default_combination_is_one_the_system_can_parse() {
        assert!(DEFAULT.parse::<Shortcut>().is_ok());
    }
}
