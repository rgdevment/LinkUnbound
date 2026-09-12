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
                ..found.clone()
            });
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
    pub fn duplicated(&self, id: String) -> Self {
        Self {
            id,
            name: format!("{} (copia)", self.name),
            custom: true,
            hidden: false,
            profiles: Vec::new(),
            ..self.clone()
        }
    }

    pub fn profile(&self, id: &str) -> Option<&Profile> {
        self.profiles.iter().find(|p| p.id == id)
    }

    pub fn supports_private(&self) -> bool {
        self.private_flag.is_some()
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
    use super::{Browser, merge};

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
        }
    }

    /// An update moves the executable, so a saved path must never win.
    #[test]
    fn detection_decides_the_path_and_the_saved_entry_decides_the_rest() {
        let detected = vec![browser("chrome", "C:/New/chrome.exe")];
        let saved = vec![Browser {
            hidden: true,
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
        let copy = original.duplicated("custom-1".to_owned());
        assert_eq!(copy.id, "custom-1");
        assert_eq!(copy.exe, "C:/c.exe");
        assert_eq!(copy.private_flag.as_deref(), Some("--incognito"));
        assert!(copy.custom);
        assert!(copy.profiles.is_empty());
        assert!(copy.name.contains("copia"));
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
