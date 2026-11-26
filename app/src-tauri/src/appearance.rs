use linkunbound_core::{Density, PickerPosition, PickerStyle, Preferences, TextSize, Theme};
use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use crate::{shell, store};

/// Only these fields, so a stale copy here never undoes what another page saved.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Appearance {
    pub theme: Theme,
    pub high_contrast: bool,
    pub text_size: TextSize,
    pub picker_style: PickerStyle,
    pub density: Density,
    pub picker_position: PickerPosition,
}

impl Appearance {
    #[must_use]
    pub const fn of(prefs: &Preferences) -> Self {
        Self {
            theme: prefs.theme,
            high_contrast: prefs.high_contrast,
            text_size: prefs.text_size,
            picker_style: prefs.picker_style,
            density: prefs.density,
            picker_position: prefs.picker_position,
        }
    }

    pub const fn onto(self, prefs: &mut Preferences) {
        prefs.theme = self.theme;
        prefs.high_contrast = self.high_contrast;
        prefs.text_size = self.text_size;
        prefs.picker_style = self.picker_style;
        prefs.density = self.density;
        prefs.picker_position = self.picker_position;
    }
}

#[tauri::command]
pub fn appearance_get() -> Appearance {
    Appearance::of(&store().prefs())
}

#[tauri::command]
pub fn appearance_set(app: AppHandle, appearance: Appearance) -> Result<Appearance, String> {
    store()
        .edit_prefs(|prefs| {
            appearance.onto(prefs);
            true
        })
        .map_err(|e| e.to_string())?;
    shell::repaint(&app, appearance.theme);
    Ok(appearance)
}

/// Back to what a fresh install draws.
#[tauri::command]
pub fn appearance_reset(app: AppHandle) -> Result<Appearance, String> {
    appearance_set(app, Appearance::of(&Preferences::default()))
}

#[cfg(test)]
mod tests {
    use linkunbound_core::{Density, PickerPosition, PickerStyle, Preferences, TextSize, Theme};

    use super::Appearance;

    #[test]
    fn it_changes_only_what_the_page_shows() {
        let mut prefs = Preferences {
            shortcut: None,
            looks_for_updates: false,
            ..Preferences::default()
        };
        Appearance {
            theme: Theme::Dark,
            high_contrast: true,
            text_size: TextSize::Large,
            picker_style: PickerStyle::Classic,
            density: Density::Comfortable,
            picker_position: PickerPosition::Center,
        }
        .onto(&mut prefs);

        assert_eq!(prefs.theme, Theme::Dark);
        assert!(prefs.high_contrast);
        assert_eq!(prefs.text_size, TextSize::Large);
        assert_eq!(prefs.picker_style, PickerStyle::Classic);
        assert_eq!(prefs.density, Density::Comfortable);
        assert_eq!(prefs.picker_position, PickerPosition::Center);
        assert_eq!(prefs.shortcut, None);
        assert!(!prefs.looks_for_updates);
    }

    #[test]
    fn it_reads_back_what_it_wrote() {
        let mut prefs = Preferences::default();
        let asked = Appearance {
            theme: Theme::Light,
            high_contrast: true,
            text_size: TextSize::Small,
            picker_style: PickerStyle::Sheet,
            density: Density::Compact,
            picker_position: PickerPosition::Pointer,
        };
        asked.onto(&mut prefs);
        assert_eq!(Appearance::of(&prefs), asked);
    }

    #[test]
    fn it_speaks_the_names_the_page_sends() {
        let sent = serde_json::json!({
            "theme": "system",
            "high_contrast": true,
            "text_size": "large",
            "picker_style": "sheet",
            "density": "comfortable",
            "picker_position": "center",
        });
        let got: Appearance = serde_json::from_value(sent.clone()).unwrap();

        assert_eq!(got.theme, Theme::System);
        assert!(got.high_contrast);
        assert_eq!(got.text_size, TextSize::Large);
        assert_eq!(got.picker_style, PickerStyle::Sheet);
        assert_eq!(got.density, Density::Comfortable);
        assert_eq!(got.picker_position, PickerPosition::Center);
        assert_eq!(serde_json::to_value(got).unwrap(), sent);
    }

    #[test]
    fn it_refuses_a_name_the_page_does_not_know() {
        let sent = serde_json::json!({
            "theme": "system", "high_contrast": false, "text_size": "huge",
            "picker_style": "bear", "density": "normal", "picker_position": "pointer",
        });
        assert!(serde_json::from_value::<Appearance>(sent).is_err());
    }

    #[test]
    fn the_defaults_are_what_a_fresh_install_draws() {
        let fresh = Appearance::of(&Preferences::default());
        assert_eq!(fresh.theme, Theme::System);
        assert!(!fresh.high_contrast);
        assert_eq!(fresh.text_size, TextSize::Normal);
        assert_eq!(fresh.picker_style, PickerStyle::Bear);
        assert_eq!(fresh.density, Density::Normal);
        assert_eq!(fresh.picker_position, PickerPosition::Pointer);
    }
}
