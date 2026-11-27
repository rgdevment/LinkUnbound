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

/// The picker's look: rows under the link, the way 1.x users know it, the bear's cards without
/// the bear, a row of icons, or the bear asking where the link goes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PickerStyle {
    Classic,
    Sheet,
    Compact,
    #[default]
    Bear,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PickerPosition {
    #[default]
    Pointer,
    Center,
}

/// How large the picker draws its words, as a factor over the sizes it was designed at.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextSize {
    Small,
    #[default]
    Normal,
    Large,
}

impl TextSize {
    #[must_use]
    pub const fn zoom(self) -> f32 {
        match self {
            Self::Small => 0.87,
            Self::Normal => 1.0,
            Self::Large => 1.13,
        }
    }
}

/// How tall each row and card of the picker is; the compact look, all icons, has none.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Density {
    Compact,
    #[default]
    Normal,
    Comfortable,
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
    #[serde(default = "yes")]
    pub notify_on_rule: bool,
    /// Carried from 1.x so somebody who read the Edge notice once is not told again.
    #[serde(default)]
    pub edge_warning_dismissed: bool,
    #[serde(default)]
    pub picker_style: PickerStyle,
    #[serde(default)]
    pub picker_position: PickerPosition,
    #[serde(default)]
    pub text_size: TextSize,
    #[serde(default)]
    pub density: Density,
    /// Darker greys, visible edges, heavier letters and an opaque sheet whatever the desktop.
    #[serde(default)]
    pub high_contrast: bool,
    /// English name, as renderers know it; absent or `system` keeps the look from before.
    #[serde(default)]
    pub font: Option<String>,
    /// Whether the resident asks about releases on its own, every few hours; off, only the
    /// settings window ever asks, and only while it is open.
    #[serde(default = "yes")]
    pub looks_for_updates: bool,
    /// `None` until the person answers, and while it is the picker asks once a minor version.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updates_automatically: Option<bool>,
    /// The version the question was last put away in; it waits for the next minor one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updates_asked_in: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub here_since: Option<u64>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub asked_for_a_star: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub news_seen: Option<String>,
    /// Closing the welcome counts too: it is offered again from settings, never pushed twice.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub welcomed: bool,
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
            notify_on_rule: true,
            edge_warning_dismissed: false,
            picker_style: PickerStyle::default(),
            picker_position: PickerPosition::default(),
            text_size: TextSize::default(),
            density: Density::default(),
            high_contrast: false,
            font: None,
            looks_for_updates: true,
            updates_automatically: None,
            updates_asked_in: None,
            here_since: None,
            asked_for_a_star: false,
            news_seen: None,
            welcomed: false,
        }
    }
}

impl Preferences {
    /// The family the person chose, or `None` for the system's.
    #[must_use]
    pub fn chosen_font(&self) -> Option<&str> {
        self.font
            .as_deref()
            .map(str::trim)
            .filter(|family| !family.is_empty() && !family.eq_ignore_ascii_case("system"))
    }

    /// The bear became everybody's picker once, whatever was chosen before; the move is written
    /// down so a style picked afterwards stands. Returns the style left behind, when it was not
    /// already the bear, so the person can be told how to get it back.
    pub fn meets_the_bear(&mut self) -> Option<PickerStyle> {
        if self.schema_version >= 1 {
            return None;
        }
        self.schema_version = 1;
        let before = std::mem::replace(&mut self.picker_style, PickerStyle::Bear);
        (before != PickerStyle::Bear).then_some(before)
    }

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
            "edge_warning_dismissed" => self.edge_warning_dismissed = said == "1",
            "global_hotkey" => self.shortcut = shortcut_of(said),
            _ => {}
        }
        self
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
    use super::{Density, Locale, PickerPosition, PickerStyle, Preferences, TextSize, Theme};

    #[test]
    fn preferences_written_before_the_update_answer_read_it_as_unanswered() {
        let read: Preferences =
            serde_json::from_str(r#"{"looks_for_updates": true}"#).expect("old file");
        assert_eq!(read.updates_automatically, None);
        assert_eq!(read.updates_asked_in, None);
    }

    #[test]
    fn an_unanswered_update_question_is_not_written_out() {
        let written = serde_json::to_string(&Preferences::default()).expect("written");
        assert!(!written.contains("updates_automatically"));
        assert!(!written.contains("updates_asked_in"));
    }

    #[test]
    fn an_update_answer_survives_a_round_trip() {
        let given = Preferences {
            updates_automatically: Some(false),
            updates_asked_in: Some("2.11.0".to_owned()),
            ..Preferences::default()
        };
        let read: Preferences =
            serde_json::from_str(&serde_json::to_string(&given).expect("written")).expect("read");
        assert_eq!(read, given);
    }

    #[test]
    fn a_fresh_install_follows_the_system_and_claims_the_default_shortcut() {
        let p = Preferences::default();
        assert_eq!(p.theme, Theme::System);
        assert_eq!(p.locale, Locale::System);
        assert_eq!(p.shortcut.as_deref(), Some("Alt+Shift+L"));
        assert!(p.notify_on_rule);
    }

    /// Everything 1.x kept beside the theme was being dropped on the way in: the language went
    /// back to the system's and the shortcut was gone.
    #[test]
    fn the_rest_of_what_1_x_chose_is_carried_over_too() {
        let said = Preferences::default()
            .with_legacy("locale", "es\n")
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

    /// The bear is what a copy gets until somebody asks for another look. The words in the file
    /// are the ones settings shows, so a hand edit reads back.
    #[test]
    fn the_picker_is_the_bear_unless_another_look_was_asked_for() {
        let fresh: Preferences = serde_json::from_str("{}").unwrap();
        assert_eq!(fresh.picker_style, PickerStyle::Bear);

        let asked: Preferences = serde_json::from_str(r#"{"picker_style": "sheet"}"#).unwrap();
        assert_eq!(asked.picker_style, PickerStyle::Sheet);

        let written = serde_json::to_string(&asked).unwrap();
        assert!(written.contains(r#""picker_style":"sheet""#), "{written}");
    }

    #[test]
    fn the_picker_opens_by_the_pointer_unless_the_center_was_asked_for() {
        let fresh: Preferences = serde_json::from_str("{}").unwrap();
        assert_eq!(fresh.picker_position, PickerPosition::Pointer);

        let asked: Preferences = serde_json::from_str(r#"{"picker_position": "center"}"#).unwrap();
        assert_eq!(asked.picker_position, PickerPosition::Center);

        let written = serde_json::to_string(&asked).unwrap();
        assert!(
            written.contains(r#""picker_position":"center""#),
            "{written}"
        );
    }

    /// Everybody moves to the bear once; after that, what they pick stays picked.
    #[test]
    fn the_bear_comes_once_and_a_later_choice_stands() {
        let mut kept: Preferences = serde_json::from_str(r#"{"picker_style": "sheet"}"#).unwrap();
        assert_eq!(kept.meets_the_bear(), Some(PickerStyle::Sheet));
        assert_eq!(kept.picker_style, PickerStyle::Bear);

        kept.picker_style = PickerStyle::Classic;
        assert_eq!(kept.meets_the_bear(), None);
        assert_eq!(kept.picker_style, PickerStyle::Classic);

        let mut fresh = Preferences::default();
        assert_eq!(fresh.meets_the_bear(), None, "nothing to tell a new copy");
        assert_eq!(fresh.schema_version, 1);
    }

    #[test]
    fn turning_the_shortcut_off_is_kept_as_off() {
        let p: Preferences = serde_json::from_str(r#"{"shortcut": null}"#).unwrap();
        assert!(p.shortcut.is_none());
    }

    #[test]
    fn a_file_from_before_text_size_density_and_contrast_reads_as_ordinary() {
        let fresh: Preferences = serde_json::from_str("{}").unwrap();
        assert_eq!(fresh.text_size, TextSize::Normal);
        assert_eq!(fresh.density, Density::Normal);
        assert!(!fresh.high_contrast);

        let old: Preferences =
            serde_json::from_str(r#"{"theme": "dark", "picker_style": "sheet"}"#).unwrap();
        assert_eq!(old.theme, Theme::Dark);
        assert_eq!(old.picker_style, PickerStyle::Sheet);
        assert_eq!(old.text_size, TextSize::Normal);
        assert_eq!(old.density, Density::Normal);
        assert!(!old.high_contrast);
        assert_eq!(Preferences::default().text_size, TextSize::Normal);
    }

    #[test]
    fn text_size_density_and_contrast_are_written_in_the_words_settings_shows() {
        let asked: Preferences = serde_json::from_str(
            r#"{"text_size": "large", "density": "comfortable", "high_contrast": true}"#,
        )
        .unwrap();
        assert_eq!(asked.text_size, TextSize::Large);
        assert_eq!(asked.density, Density::Comfortable);
        assert!(asked.high_contrast);

        let written = serde_json::to_string(&asked).unwrap();
        assert!(written.contains(r#""text_size":"large""#), "{written}");
        assert!(written.contains(r#""density":"comfortable""#), "{written}");
        assert!(written.contains(r#""high_contrast":true"#), "{written}");

        let back: Preferences = serde_json::from_str(&written).unwrap();
        assert_eq!(back, asked);
    }

    #[test]
    fn a_file_from_before_the_font_reads_as_the_system_font() {
        let old: Preferences = serde_json::from_str(r#"{"theme": "dark"}"#).unwrap();
        assert_eq!(old.font, None);
        assert_eq!(old.chosen_font(), None);
        assert_eq!(Preferences::default().chosen_font(), None);
    }

    #[test]
    fn system_empty_and_blank_fonts_all_mean_the_system_font() {
        for said in ["system", "SYSTEM", "System", "", "   "] {
            let p: Preferences = serde_json::from_str(&format!(r#"{{"font": "{said}"}}"#)).unwrap();
            assert_eq!(p.chosen_font(), None, "{said:?}");
        }
        let null: Preferences = serde_json::from_str(r#"{"font": null}"#).unwrap();
        assert_eq!(null.chosen_font(), None);
    }

    #[test]
    fn a_chosen_font_is_kept_trimmed_and_written_back_out() {
        let p: Preferences = serde_json::from_str(r#"{"font": "  Calibri "}"#).unwrap();
        assert_eq!(p.chosen_font(), Some("Calibri"));

        let written = serde_json::to_string(&p).unwrap();
        assert!(written.contains(r#""font":"  Calibri ""#), "{written}");
        let back: Preferences = serde_json::from_str(&written).unwrap();
        assert_eq!(back, p);
    }

    #[test]
    fn a_family_that_only_starts_with_system_is_still_a_family() {
        let p: Preferences = serde_json::from_str(r#"{"font": "System Mono"}"#).unwrap();
        assert_eq!(p.chosen_font(), Some("System Mono"));
    }

    #[test]
    fn the_text_sizes_scale_around_the_size_it_was_designed_at() {
        assert_eq!(TextSize::Small.zoom(), 0.87);
        assert_eq!(TextSize::Normal.zoom(), 1.0);
        assert_eq!(TextSize::Large.zoom(), 1.13);
    }
}
