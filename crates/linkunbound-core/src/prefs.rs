use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Theme {
    #[default]
    System,
    Light,
    Dark,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Locale {
    #[default]
    System,
    Spanish,
    English,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Preferences {
    #[serde(default)]
    pub schema_version: u32,
    #[serde(default)]
    pub theme: Theme,
    #[serde(default)]
    pub locale: Locale,
    /// `None` means the user turned it off, which is not the same as never
    /// having chosen: an absent field falls back to the default combination.
    #[serde(default = "default_shortcut")]
    pub shortcut: Option<String>,
    #[serde(default)]
    pub hide_tray: bool,
    #[serde(default = "yes")]
    pub notify_on_rule: bool,
}

fn default_shortcut() -> Option<String> {
    Some("Alt+Shift+L".to_owned())
}

const fn yes() -> bool {
    true
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            schema_version: 0,
            theme: Theme::default(),
            locale: Locale::default(),
            shortcut: default_shortcut(),
            hide_tray: false,
            notify_on_rule: true,
        }
    }
}

impl Preferences {
    /// 1.x kept the theme in a file of its own holding nothing but the word.
    #[must_use]
    pub fn with_legacy_theme(mut self, raw: &str) -> Self {
        self.theme = match raw.trim().to_ascii_lowercase().as_str() {
            "dark" => Theme::Dark,
            "light" => Theme::Light,
            _ => Theme::System,
        };
        self
    }

    /// Hiding the tray with no shortcut left would leave no way back in.
    #[must_use]
    pub fn reachable(&self) -> bool {
        !self.hide_tray || self.shortcut.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::{Locale, Preferences, Theme};

    #[test]
    fn a_fresh_install_follows_the_system_and_claims_the_default_shortcut() {
        let p = Preferences::default();
        assert_eq!(p.theme, Theme::System);
        assert_eq!(p.locale, Locale::System);
        assert_eq!(p.shortcut.as_deref(), Some("Alt+Shift+L"));
        assert!(p.notify_on_rule);
    }

    #[test]
    fn the_theme_1_x_left_in_its_own_file_is_carried_over() {
        assert_eq!(
            Preferences::default().with_legacy_theme("dark").theme,
            Theme::Dark
        );
        assert_eq!(
            Preferences::default().with_legacy_theme("Light\n").theme,
            Theme::Light
        );
        assert_eq!(
            Preferences::default().with_legacy_theme("nonsense").theme,
            Theme::System
        );
    }

    /// A file written before the field existed must not read as "turned off".
    #[test]
    fn a_file_without_a_shortcut_falls_back_instead_of_disabling_it() {
        let p: Preferences = serde_json::from_str("{}").unwrap();
        assert_eq!(p.shortcut.as_deref(), Some("Alt+Shift+L"));
        assert!(p.notify_on_rule);
    }

    #[test]
    fn turning_the_shortcut_off_is_kept_as_off() {
        let p: Preferences = serde_json::from_str(r#"{"shortcut": null}"#).unwrap();
        assert!(p.shortcut.is_none());
    }

    #[test]
    fn hiding_the_tray_without_a_shortcut_leaves_no_way_back_in() {
        let hidden = Preferences {
            hide_tray: true,
            shortcut: None,
            ..Preferences::default()
        };
        assert!(!hidden.reachable());

        let with_key = Preferences {
            hide_tray: true,
            ..Preferences::default()
        };
        assert!(with_key.reachable());
    }
}
