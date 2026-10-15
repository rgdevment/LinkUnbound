use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum LaunchRefused {
    #[error("the profile {0} is no longer there")]
    ProfileGone(String),
    #[error("this browser cannot open a private window")]
    PrivateImpossible,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Profile {
    pub id: String,
    pub name: String,
    pub args: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Browser {
    pub id: String,
    pub name: String,
    pub exe: String,
    #[serde(default)]
    pub profiles: Vec<Profile>,
    #[serde(default)]
    pub extra_args: Vec<String>,
    /// Declared at detection, never guessed: browsers treat an unknown switch as a URL.
    #[serde(default)]
    pub private_flag: Option<String>,
    /// Where to read the icon from when it is not the executable itself.
    #[serde(default)]
    pub icon_path: Option<String>,
    /// Added by hand, so the registry cannot confirm or refresh it.
    #[serde(default)]
    pub custom: bool,
    /// Kept out of the picker without being forgotten.
    #[serde(default)]
    pub hidden: bool,
    /// The key the person gave each row in the picker, by profile id, or by "" for a browser with
    /// none. Kept here rather than on the profile, which every scan rebuilds.
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub keys: std::collections::BTreeMap<String, char>,
}

/// A key a person may give a row of the picker: a digit from 1 or a letter, in either case.
#[must_use]
pub fn usable_key(key: char) -> Option<char> {
    let key = key.to_ascii_uppercase();
    (key.is_ascii_uppercase() || ('1'..='9').contains(&key)).then_some(key)
}

/// The keys the rows answer to unless the person chose one: the digits, then the three rows of
/// letters under them.
pub const KEYS: &str = "123456789QWERTYASDFGHZXCVBN";

/// The key each row shows, from the ones chosen in order of the rows: a chosen key stands unless
/// an earlier row holds it already, the others take the first of the defaults nobody took, and
/// past the defaults a row has none and opens with the pointer.
#[must_use]
pub fn assigned_keys(chosen: impl IntoIterator<Item = Option<char>>) -> Vec<Option<char>> {
    let mut given: Vec<Option<char>> = Vec::new();
    for key in chosen {
        let key = key.and_then(usable_key);
        given.push(key.filter(|k| !given.contains(&Some(*k))));
    }
    let spare: Vec<char> = KEYS
        .chars()
        .filter(|k| !given.contains(&Some(*k)))
        .collect();
    let mut spare = spare.into_iter();
    given
        .into_iter()
        .map(|key| key.or_else(|| spare.next()))
        .collect()
}

/// The keys the picker gives each browser's rows, by profile id or "" as in [`Browser::keys`]: the
/// hidden ones are not in the picker and have none.
#[must_use]
pub fn picker_keys(browsers: &[Browser]) -> Vec<std::collections::BTreeMap<String, char>> {
    let rows: Vec<(usize, String, Option<char>)> = browsers
        .iter()
        .enumerate()
        .filter(|(_, b)| !b.hidden)
        .flat_map(|(at, b)| {
            b.rows().into_iter().map(move |profile| {
                let id = profile.map(|p| p.id.as_str());
                (at, id.unwrap_or_default().to_owned(), b.key_for(id))
            })
        })
        .collect();
    let keys = assigned_keys(rows.iter().map(|(_, _, key)| *key));
    let mut out = vec![std::collections::BTreeMap::new(); browsers.len()];
    for ((at, row, _), key) in rows.into_iter().zip(keys) {
        if let Some(key) = key {
            out[at].insert(row, key);
        }
    }
    out
}

/// What the registry finds is the truth about paths and profiles; what the user
/// saved is the truth about order, hiding and their own entries. A saved entry
/// that detection no longer sees was uninstalled and goes.
#[must_use]
pub fn merge(detected: Vec<Browser>, saved: &[Browser]) -> Vec<Browser> {
    let mut out: Vec<Browser> = Vec::with_capacity(detected.len() + saved.len());
    for kept in saved {
        if kept.custom {
            out.push(kept.clone());
        } else if let Some(found) = detected.iter().find(|d| d.id == kept.id) {
            // What the user edited wins; what detection owns is the path, the
            // profiles and whether the browser can open privately at all.
            out.push(Browser {
                hidden: kept.hidden,
                name: kept.name.clone(),
                extra_args: kept.extra_args.clone(),
                private_flag: kept.private_flag.clone(),
                icon_path: kept.icon_path.clone(),
                keys: kept.keys.clone(),
                ..found.clone()
            });
            if let Some(last) = out.last_mut() {
                last.forget_gone_keys();
            }
        }
    }
    for found in detected {
        if !out.iter().any(|b| b.id == found.id) {
            out.push(found);
        }
    }
    out
}

impl Browser {
    /// Where the icon comes from: the override when set, the executable otherwise.
    #[must_use]
    pub fn icon_source(&self) -> &str {
        self.icon_path.as_deref().unwrap_or(&self.exe)
    }

    /// A copy the user can change without touching the original, which is how
    /// one browser becomes two profiles of the same binary.
    #[must_use]
    pub fn duplicated(&self, id: String, named: &str) -> Self {
        Self {
            id,
            name: named.to_owned(),
            custom: true,
            hidden: false,
            keys: Default::default(),
            profiles: Vec::new(),
            ..self.clone()
        }
    }

    /// The rows it puts in the picker: one per profile, or the browser itself when it has none.
    #[must_use]
    pub fn rows(&self) -> Vec<Option<&Profile>> {
        if self.profiles.is_empty() {
            vec![None]
        } else {
            self.profiles.iter().map(Some).collect()
        }
    }

    /// The key the person gave the row of this profile, or of the browser when it has none.
    #[must_use]
    pub fn key_for(&self, profile: Option<&str>) -> Option<char> {
        self.keys
            .get(profile.unwrap_or_default())
            .copied()
            .and_then(usable_key)
    }

    /// A scan can take a profile away, or give profiles to a browser that had none: a key left on
    /// a row nobody sees would still count as taken, with no box in settings to clear it.
    fn forget_gone_keys(&mut self) {
        let live: Vec<String> = self
            .rows()
            .iter()
            .map(|p| p.map(|p| p.id.clone()).unwrap_or_default())
            .collect();
        self.keys.retain(|row, _| live.contains(row));
    }

    pub fn profile(&self, id: &str) -> Option<&Profile> {
        self.profiles.iter().find(|p| p.id == id)
    }

    pub fn supports_private(&self) -> bool {
        self.private_flag
            .as_deref()
            .is_some_and(|f| !f.trim().is_empty())
    }

    /// Never degrades quietly: a rule asking for a profile that is gone, or for
    /// a private window a browser cannot open, is a promise this cannot keep,
    /// and opening the link anyway sends it somewhere the user did not choose.
    pub fn launch(
        &self,
        profile: Option<&str>,
        private: bool,
        url: &str,
    ) -> Result<Vec<String>, LaunchRefused> {
        let mut args = self.extra_args.clone();

        if let Some(id) = profile {
            let known = self
                .profile(id)
                .ok_or_else(|| LaunchRefused::ProfileGone(id.to_owned()))?;
            args.extend(known.args.iter().cloned());
        }

        if private {
            let flag = self
                .private_flag
                .as_ref()
                .ok_or(LaunchRefused::PrivateImpossible)?;
            args.push(flag.clone());
        }

        args.push(url.to_owned());
        Ok(args)
    }
}

#[cfg(test)]
mod tests {

    /// A copy is made to be used, so it starts visible even when what it was copied from is
    /// hidden — otherwise duplicating a hidden browser produces something that cannot be found.
    #[test]
    fn a_copy_starts_visible_whatever_it_was_copied_from() {
        let mut original = Browser {
            id: "chrome".to_owned(),
            name: "Google Chrome".to_owned(),
            exe: "chrome.exe".to_owned(),
            profiles: Vec::new(),
            extra_args: Vec::new(),
            private_flag: Some("--incognito".to_owned()),
            icon_path: None,
            custom: false,
            hidden: true,
            keys: Default::default(),
        };

        assert!(
            !original.duplicated("custom-1".to_owned(), "x").hidden,
            "a copy of a hidden one is still there to be used"
        );

        original.hidden = false;
        assert!(!original.duplicated("custom-2".to_owned(), "x").hidden);
    }

    /// The picker lights the private icon on the strength of this, and the keyboard acts on it.
    #[test]
    fn private_is_offered_only_where_there_is_a_flag_to_do_it_with() {
        let mut browser = Browser {
            id: "chrome".to_owned(),
            name: "Google Chrome".to_owned(),
            exe: "chrome.exe".to_owned(),
            profiles: Vec::new(),
            extra_args: Vec::new(),
            private_flag: Some("--incognito".to_owned()),
            icon_path: None,
            custom: false,
            hidden: false,
            keys: Default::default(),
        };
        assert!(browser.supports_private());

        browser.private_flag = None;
        assert!(!browser.supports_private());

        browser.private_flag = Some(String::new());
        assert!(
            !browser.supports_private(),
            "an empty flag is a field somebody left blank, not a way to open a private window"
        );
    }
    use super::*;

    fn chrome() -> Browser {
        Browser {
            id: "chrome".to_owned(),
            name: "Chrome".to_owned(),
            exe: "chrome.exe".to_owned(),
            profiles: vec![Profile {
                id: "Profile 2".to_owned(),
                name: "Work".to_owned(),
                args: vec!["--profile-directory=Profile 2".to_owned()],
            }],
            private_flag: Some("--incognito".to_owned()),
            extra_args: Vec::new(),
            icon_path: None,
            custom: false,
            hidden: false,
            keys: Default::default(),
        }
    }

    fn safari() -> Browser {
        Browser {
            id: "safari".to_owned(),
            name: "Safari".to_owned(),
            exe: "Safari.app".to_owned(),
            profiles: Vec::new(),
            private_flag: None,
            extra_args: Vec::new(),
            icon_path: None,
            custom: false,
            hidden: false,
            keys: Default::default(),
        }
    }

    #[test]
    fn a_known_profile_and_a_private_window_compose_as_asked() {
        let args = chrome()
            .launch(Some("Profile 2"), true, "https://intranet.test")
            .unwrap();
        assert_eq!(
            args,
            [
                "--profile-directory=Profile 2",
                "--incognito",
                "https://intranet.test"
            ]
        );
    }

    #[test]
    fn a_profile_that_vanished_refuses_instead_of_opening_the_default_one() {
        let refused = chrome()
            .launch(Some("Profile 7"), false, "https://intranet.test")
            .unwrap_err();
        assert_eq!(refused, LaunchRefused::ProfileGone("Profile 7".to_owned()));
    }

    #[test]
    fn a_browser_that_cannot_go_private_refuses_instead_of_opening_in_the_open() {
        let refused = safari()
            .launch(None, true, "https://bank.test")
            .unwrap_err();
        assert_eq!(refused, LaunchRefused::PrivateImpossible);
    }

    #[test]
    fn asking_for_nothing_special_still_works() {
        let args = safari()
            .launch(None, false, "https://example.test")
            .unwrap();
        assert_eq!(args, ["https://example.test"]);
    }
}

#[cfg(test)]
mod merging {
    use super::{Browser, merge, usable_key};

    fn browser(id: &str, exe: &str) -> Browser {
        Browser {
            id: id.to_owned(),
            name: id.to_owned(),
            exe: exe.to_owned(),
            profiles: Vec::new(),
            extra_args: Vec::new(),
            private_flag: None,
            icon_path: None,
            custom: false,
            hidden: false,
            keys: Default::default(),
        }
    }

    /// An update moves the executable, so a saved path must never win.
    #[test]
    fn detection_decides_the_path_and_the_saved_entry_decides_the_rest() {
        let detected = vec![browser("chrome", "C:/New/chrome.exe")];
        let saved = vec![Browser {
            hidden: true,
            keys: Default::default(),
            extra_args: vec!["--foo".to_owned()],
            ..browser("chrome", "C:/Old/chrome.exe")
        }];
        let out = merge(detected, &saved);
        assert_eq!(out[0].exe, "C:/New/chrome.exe");
        assert!(out[0].hidden);
        assert_eq!(out[0].extra_args, ["--foo"]);
    }

    #[test]
    fn a_browser_added_by_hand_survives_without_the_registry() {
        let mine = Browser {
            custom: true,
            ..browser("mine", "C:/Apps/odd.exe")
        };
        let out = merge(Vec::new(), &[mine]);
        assert_eq!(out.len(), 1);
        assert!(out[0].custom);
    }

    #[test]
    fn a_detected_browser_that_vanished_is_dropped() {
        let out = merge(Vec::new(), &[browser("gone", "C:/Gone/x.exe")]);
        assert!(out.is_empty());
    }

    #[test]
    fn something_newly_installed_shows_up_at_the_end() {
        let saved = vec![browser("chrome", "C:/C/chrome.exe")];
        let detected = vec![
            browser("firefox", "C:/F/firefox.exe"),
            browser("chrome", "C:/C/chrome.exe"),
        ];
        let ids: Vec<String> = merge(detected, &saved).into_iter().map(|b| b.id).collect();
        assert_eq!(ids, ["chrome", "firefox"]);
    }

    #[test]
    fn a_custom_icon_survives_a_path_that_detection_moved() {
        let detected = vec![browser("chrome", "C:/New/chrome.exe")];
        let saved = vec![Browser {
            icon_path: Some("C:/mine/icon.ico".to_owned()),
            ..browser("chrome", "C:/Old/chrome.exe")
        }];
        let out = merge(detected, &saved);
        assert_eq!(out[0].icon_source(), "C:/mine/icon.ico");
    }

    #[test]
    fn without_an_override_the_icon_comes_from_the_executable() {
        assert_eq!(browser("chrome", "C:/c.exe").icon_source(), "C:/c.exe");
    }

    /// The copy has to be editable, so it is the user's even when the original
    /// came from the registry, and it starts with no profiles of its own.
    #[test]
    fn a_duplicate_belongs_to_the_user_and_keeps_the_binary() {
        let original = Browser {
            private_flag: Some("--incognito".to_owned()),
            profiles: vec![super::Profile {
                id: "Default".to_owned(),
                name: "Personal".to_owned(),
                args: Vec::new(),
            }],
            ..browser("chrome", "C:/c.exe")
        };
        let copy = original.duplicated("custom-1".to_owned(), "Original (copy)");
        assert_eq!(copy.id, "custom-1");
        assert_eq!(copy.exe, "C:/c.exe");
        assert_eq!(copy.private_flag.as_deref(), Some("--incognito"));
        assert!(copy.custom);
        assert!(copy.profiles.is_empty());
        assert_eq!(
            copy.name, "Original (copy)",
            "the name is the caller's, worded in its language"
        );
    }

    /// The keys are the person's, like the name: a rescan rebuilds the profiles but not them, and
    /// a copy starts with none, or it would fight its original for the same key.
    #[test]
    fn the_keys_survive_a_scan_and_stay_off_a_copy() {
        let detected = vec![browser("chrome", "C:/real/chrome.exe")];
        let mut mine = browser("chrome", "C:/real/chrome.exe");
        mine.keys.insert("Default".to_owned(), 'W');
        mine.keys.insert(String::new(), 'F');
        mine.keys.insert("Gone".to_owned(), 'G');
        let mut detected = detected;
        detected[0].profiles = vec![super::Profile {
            id: "Default".to_owned(),
            name: "Personal".to_owned(),
            args: Vec::new(),
        }];
        let merged = merge(detected, &[mine]);
        assert_eq!(merged[0].key_for(Some("Default")), Some('W'));
        assert_eq!(
            merged[0].keys.len(),
            1,
            "the rows the scan took away let go of their keys"
        );
        assert_eq!(merged[0].key_for(None), None);
        assert!(
            merged[0]
                .duplicated("custom-1".to_owned(), "x")
                .keys
                .is_empty()
        );
    }

    /// Settings shows the key each row gets on its own, so it has to be the picker's: the chosen
    /// ones stand, the rest take the free defaults in order, and a hidden browser gets none.
    #[test]
    fn every_row_shown_gets_the_key_the_picker_gives_it() {
        let mut chrome = browser("chrome", "chrome.exe");
        chrome.profiles = ["Default", "Work"]
            .map(|id| super::Profile {
                id: id.to_owned(),
                name: id.to_owned(),
                args: Vec::new(),
            })
            .to_vec();
        chrome.keys.insert("Work".to_owned(), '1');
        let mut hidden = browser("edge", "edge.exe");
        hidden.hidden = true;
        let firefox = browser("firefox", "firefox.exe");

        let keys = super::picker_keys(&[chrome, hidden, firefox]);
        assert_eq!(keys[0].get("Default"), Some(&'2'));
        assert_eq!(keys[0].get("Work"), Some(&'1'));
        assert!(keys[1].is_empty(), "hidden, so not in the picker");
        assert_eq!(keys[2].get(""), Some(&'3'));
    }

    #[test]
    fn only_a_digit_from_one_or_a_letter_is_a_key() {
        assert_eq!(usable_key('w'), Some('W'));
        assert_eq!(usable_key('7'), Some('7'));
        assert_eq!(usable_key('0'), None);
        assert_eq!(usable_key('ñ'), None);
        assert_eq!(usable_key(' '), None);
    }

    /// Settings lets a detected browser be renamed and given a private switch.
    /// Handing those fields back to detection on read undoes the edit in the
    /// same render that saved it.
    #[test]
    fn renaming_a_detected_browser_survives_the_next_merge() {
        let detected = vec![Browser {
            private_flag: Some("--incognito".to_owned()),
            ..browser("chrome", "C:/real/chrome.exe")
        }];
        let saved = vec![Browser {
            name: "Chrome del trabajo".to_owned(),
            private_flag: Some("--muy-privado".to_owned()),
            ..browser("chrome", "C:/stale/chrome.exe")
        }];

        let merged = merge(detected, &saved);
        assert_eq!(merged[0].name, "Chrome del trabajo", "the edit wins");
        assert_eq!(merged[0].private_flag.as_deref(), Some("--muy-privado"));
        assert_eq!(
            merged[0].exe, "C:/real/chrome.exe",
            "but the path stays detection's to own"
        );
    }

    #[test]
    fn the_saved_order_is_the_order_that_comes_back() {
        let saved = vec![browser("b", "C:/b.exe"), browser("a", "C:/a.exe")];
        let detected = vec![browser("a", "C:/a.exe"), browser("b", "C:/b.exe")];
        let ids: Vec<String> = merge(detected, &saved).into_iter().map(|b| b.id).collect();
        assert_eq!(ids, ["b", "a"]);
    }
}
