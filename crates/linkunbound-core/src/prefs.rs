use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Theme {
    System,
    Light,
    #[default]
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

/// The picker's look: rows under the link, the way 1.x users know it, or a sheet of tiles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PickerStyle {
    #[default]
    Classic,
    Sheet,
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
    /// Carried from 1.x so somebody who read the Edge notice once is not told again.
    #[serde(default)]
    pub edge_warning_dismissed: bool,
    #[serde(default)]
    pub picker_style: PickerStyle,
    /// Whether the resident asks about releases on its own, every few hours; off, only the
    /// settings window ever asks, and only while it is open.
    #[serde(default = "yes")]
    pub looks_for_updates: bool,
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
            edge_warning_dismissed: false,
            picker_style: PickerStyle::default(),
            looks_for_updates: true,
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

    /// 1.x kept each of these in a file of its own, holding one word or the single character
    /// `1`. Their absence is what the person chose as much as their presence, so a missing file
    /// means the default rather than nothing.
    #[must_use]
    pub fn with_legacy(mut self, named: &str, raw: &str) -> Self {
        let said = raw.trim();
        match named {
            "theme" => return self.with_legacy_theme(said),
            "locale" => {
                self.locale = match said.to_ascii_lowercase().as_str() {
                    "es" => Locale::Spanish,
                    "en" => Locale::English,
                    _ => Locale::System,
                };
            }
            "hide_tray" => self.hide_tray = said == "1",
            "edge_warning_dismissed" => self.edge_warning_dismissed = said == "1",
            "global_hotkey" => self.shortcut = shortcut_of(said),
            _ => {}
        }
        self
    }

    /// Hiding the tray with no shortcut left would leave no way back in.
    #[must_use]
    pub fn reachable(&self) -> bool {
        !self.hide_tray || self.shortcut.is_some()
    }
}

/// 1.x wrote `ctrl+shift+l`; this reads `Ctrl+Shift+L`. Same keys, different spelling, and a
/// shortcut that does not parse is one the person quietly loses.
fn shortcut_of(said: &str) -> Option<String> {
    if said.is_empty() {
        return None;
    }
    let spelled = said
        .split('+')
        .map(|part| {
            let part = part.trim();
            match part.to_ascii_lowercase().as_str() {
                "ctrl" | "control" => "Ctrl".to_owned(),
                "alt" => "Alt".to_owned(),
                "shift" => "Shift".to_owned(),
                "win" | "super" | "meta" | "cmd" => "Super".to_owned(),
                _ => part.to_ascii_uppercase(),
            }
        })
        .collect::<Vec<_>>()
        .join("+");
    (!spelled.is_empty()).then_some(spelled)
}

#[cfg(test)]
mod tests {
    use super::{Locale, PickerStyle, Preferences, Theme};

    #[test]
    fn a_fresh_install_is_dark_and_claims_the_default_shortcut() {
        let p = Preferences::default();
        assert_eq!(p.theme, Theme::Dark);
        assert_eq!(p.locale, Locale::System);
        assert_eq!(p.shortcut.as_deref(), Some("Alt+Shift+L"));
        assert!(p.notify_on_rule);
    }

    /// Everything 1.x kept beside the theme was being dropped on the way in: the language went
    /// back to the system's, the tray came back, and the shortcut was gone.
    #[test]
    fn the_rest_of_what_1_x_chose_is_carried_over_too() {
        let said = Preferences::default()
            .with_legacy("locale", "es\n")
            .with_legacy("hide_tray", "1")
            .with_legacy("edge_warning_dismissed", "1")
            .with_legacy("global_hotkey", "ctrl+shift+l");

        assert_eq!(said.locale, Locale::Spanish);
        assert_eq!(
            Preferences::default().with_legacy("locale", "en").locale,
            Locale::English
        );
        assert_eq!(
            Preferences::default().with_legacy("locale", "fr").locale,
            Locale::System,
            "a language this one does not speak follows the system"
        );
        assert!(said.hide_tray);
        assert!(said.edge_warning_dismissed);
        assert_eq!(said.shortcut.as_deref(), Some("Ctrl+Shift+L"));
    }

    /// 1.x spelled its shortcut in lower case and this reads it capitalised. A shortcut that does
    /// not parse is one the person loses without being told.
    #[test]
    fn a_shortcut_1_x_wrote_is_spelled_the_way_this_one_reads() {
        for (was, now) in [
            ("ctrl+shift+l", "Ctrl+Shift+L"),
            ("CTRL+ALT+K", "Ctrl+Alt+K"),
            ("control+shift+l", "Ctrl+Shift+L"),
            ("win+l", "Super+L"),
            ("alt+space", "Alt+SPACE"),
        ] {
            assert_eq!(
                Preferences::default()
                    .with_legacy("global_hotkey", was)
                    .shortcut
                    .as_deref(),
                Some(now),
                "{was}"
            );
        }
    }

    /// An empty file is somebody who turned the shortcut off, which is not the same as never
    /// having had one.
    #[test]
    fn a_shortcut_turned_off_stays_off() {
        assert!(
            Preferences::default()
                .with_legacy("global_hotkey", "  ")
                .shortcut
                .is_none()
        );
    }

    /// A file 1.x never wrote means the default, not an empty value.
    #[test]
    fn a_name_nobody_knows_changes_nothing() {
        let said = Preferences::default().with_legacy("something-else", "1");
        assert_eq!(said, Preferences::default());
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
        assert!(
            p.looks_for_updates,
            "a copy from before the switch keeps hearing about releases"
        );
        let quiet: Preferences = serde_json::from_str(r#"{"looks_for_updates": false}"#).unwrap();
        assert!(!quiet.looks_for_updates);
    }

    /// The rows are what a copy gets until somebody asks for the tiles, and a file from before
    /// the choice existed must read the same way. The words in the file are the ones settings
    /// shows, so a hand edit reads back.
    #[test]
    fn the_picker_is_the_classic_one_unless_the_sheet_was_asked_for() {
        let fresh: Preferences = serde_json::from_str("{}").unwrap();
        assert_eq!(fresh.picker_style, PickerStyle::Classic);

        let asked: Preferences = serde_json::from_str(r#"{"picker_style": "sheet"}"#).unwrap();
        assert_eq!(asked.picker_style, PickerStyle::Sheet);

        let written = serde_json::to_string(&asked).unwrap();
        assert!(written.contains(r#""picker_style":"sheet""#), "{written}");
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
