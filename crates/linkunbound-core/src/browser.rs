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
}

impl Browser {
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
