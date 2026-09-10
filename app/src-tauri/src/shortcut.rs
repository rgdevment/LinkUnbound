use tauri::{AppHandle, Runtime};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

/// Returns what was actually claimed. `None` means the user turned it off, or
/// that something else already holds the combination — the plugin refuses it
/// rather than stealing it, and the settings screen has to say so.
pub fn install<R: Runtime>(app: &AppHandle<R>, wanted: Option<&str>) -> Option<String> {
    let manager = app.global_shortcut();
    let _ = manager.unregister_all();

    let combination = wanted?;
    let parsed = combination.parse::<Shortcut>().ok()?;
    manager
        .on_shortcut(parsed, |app, _shortcut, event| {
            if event.state() == ShortcutState::Pressed {
                crate::shell::open_settings(app);
            }
        })
        .ok()?;
    Some(combination.to_owned())
}

#[cfg(test)]
mod tests {
    use linkunbound_core::Preferences;
    use tauri_plugin_global_shortcut::Shortcut;

    /// The default lives in the core, which cannot ask the plugin whether the
    /// combination it ships is one the system will accept.
    #[test]
    fn the_default_the_core_ships_is_one_the_system_can_parse() {
        let shipped = Preferences::default()
            .shortcut
            .expect("a fresh install claims one");
        assert!(shipped.parse::<Shortcut>().is_ok());
    }

    /// What the settings screen sends comes from a key capture, so it has to
    /// survive the same parser the registration uses.
    #[test]
    fn the_combinations_a_capture_can_produce_all_parse() {
        for combination in [
            "Alt+Shift+L",
            "Control+Alt+B",
            "Control+Shift+Space",
            "Super+L",
            "Control+Alt+Digit1",
            "F9",
        ] {
            assert!(
                combination.parse::<Shortcut>().is_ok(),
                "{combination} should parse"
            );
        }
    }

    #[test]
    fn nonsense_is_refused_instead_of_claiming_something_else() {
        assert!("".parse::<Shortcut>().is_err());
        assert!("Shift".parse::<Shortcut>().is_err());
    }
}
