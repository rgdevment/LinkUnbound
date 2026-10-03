use crate::browser::Profile;

/// The 1.x identifier scheme, kept byte for byte: rules migrated from it store
/// this string, and an id that does not match points a rule at nothing.
#[must_use]
pub fn id_for(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    for ch in name.to_ascii_lowercase().chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch);
        } else if !out.ends_with('-') {
            out.push('-');
        }
    }
    out.trim_matches('-').to_owned()
}

/// Chromium keeps the human names of its profiles in `Local State`; the
/// directory names it launches with are the keys of that map.
#[must_use]
pub fn profiles_in(local_state: &str) -> Vec<Profile> {
    let Ok(state) = serde_json::from_str::<serde_json::Value>(local_state) else {
        return Vec::new();
    };
    let Some(cache) = state
        .get("profile")
        .and_then(|p| p.get("info_cache"))
        .and_then(serde_json::Value::as_object)
    else {
        return Vec::new();
    };

    let mut profiles: Vec<Profile> = cache
        .iter()
        .map(|(dir_name, info)| {
            let label = info
                .get("name")
                .and_then(serde_json::Value::as_str)
                .filter(|n| !n.is_empty())
                .unwrap_or(dir_name);
            Profile {
                id: dir_name.clone(),
                name: label.to_owned(),
                args: vec![format!("--profile-directory={dir_name}")],
            }
        })
        .collect();
    profiles.sort_by_key(|p| p.name.to_lowercase());
    profiles
}

/// Where each Chromium build keeps `Local State`, below Application Support on a Mac and below
/// `%LOCALAPPDATA%` on Windows. Every channel keeps its own: reading stable's for a Beta offers
/// profiles it does not have, and launching one creates it empty. Most specific first.
struct Chromium {
    bundle: &'static str,
    mac: &'static str,
    windows: Option<&'static str>,
}

const CHROMIUM: [Chromium; 19] = [
    Chromium {
        bundle: "google chrome canary",
        mac: "Google/Chrome Canary",
        windows: Some(r"Google\Chrome SxS"),
    },
    Chromium {
        bundle: "google chrome beta",
        mac: "Google/Chrome Beta",
        windows: Some(r"Google\Chrome Beta"),
    },
    Chromium {
        bundle: "google chrome dev",
        mac: "Google/Chrome Dev",
        windows: Some(r"Google\Chrome Dev"),
    },
    Chromium {
        bundle: "google chrome for testing",
        mac: "Google/Chrome for Testing",
        windows: None,
    },
    Chromium {
        bundle: "google chrome",
        mac: "Google/Chrome",
        windows: Some(r"Google\Chrome"),
    },
    Chromium {
        bundle: "chromium",
        mac: "Chromium",
        windows: Some("Chromium"),
    },
    Chromium {
        bundle: "chrome",
        mac: "Google/Chrome",
        windows: None,
    },
    Chromium {
        bundle: "microsoft edge canary",
        mac: "Microsoft Edge Canary",
        windows: Some(r"Microsoft\Edge SxS"),
    },
    Chromium {
        bundle: "microsoft edge beta",
        mac: "Microsoft Edge Beta",
        windows: Some(r"Microsoft\Edge Beta"),
    },
    Chromium {
        bundle: "microsoft edge dev",
        mac: "Microsoft Edge Dev",
        windows: Some(r"Microsoft\Edge Dev"),
    },
    Chromium {
        bundle: "microsoft edge",
        mac: "Microsoft Edge",
        windows: Some(r"Microsoft\Edge"),
    },
    Chromium {
        bundle: "brave browser nightly",
        mac: "BraveSoftware/Brave-Browser-Nightly",
        windows: Some(r"BraveSoftware\Brave-Browser-Nightly"),
    },
    Chromium {
        bundle: "brave browser dev",
        mac: "BraveSoftware/Brave-Browser-Dev",
        windows: Some(r"BraveSoftware\Brave-Browser-Dev"),
    },
    Chromium {
        bundle: "brave browser beta",
        mac: "BraveSoftware/Brave-Browser-Beta",
        windows: Some(r"BraveSoftware\Brave-Browser-Beta"),
    },
    Chromium {
        bundle: "brave browser",
        mac: "BraveSoftware/Brave-Browser",
        windows: Some(r"BraveSoftware\Brave-Browser"),
    },
    Chromium {
        bundle: "vivaldi",
        mac: "Vivaldi",
        windows: Some("Vivaldi"),
    },
    Chromium {
        bundle: "arc",
        mac: "Arc/User Data",
        windows: None,
    },
    Chromium {
        bundle: "opera gx",
        mac: "com.operasoftware.OperaGX",
        windows: None,
    },
    Chromium {
        bundle: "opera",
        mac: "com.operasoftware.Opera",
        windows: None,
    },
];

/// Below Application Support, for a bundle named this. Matched on the bundle's name, never on
/// the folder it sits in: a browser under `/Users/marc/Applications` is not Arc.
#[must_use]
pub fn chromium_home_on_mac(bundle_name: &str) -> Option<&'static str> {
    let name = bundle_name.to_ascii_lowercase();
    let name = name.strip_suffix(".app").unwrap_or(&name);
    CHROMIUM
        .iter()
        .find(|c| {
            if c.bundle == "arc" {
                name == "arc"
            } else {
                name.contains(c.bundle)
            }
        })
        .map(|c| c.mac)
}

/// Below `%LOCALAPPDATA%`, with `User Data` still to add, for a browser installed at `exe`.
/// Chromium installs into `<vendor>\<channel>\Application`, whichever root it went under.
#[must_use]
pub fn chromium_home_on_windows(exe: &str) -> Option<&'static str> {
    let exe = exe.replace('/', "\\").to_ascii_lowercase();
    CHROMIUM.iter().find_map(|c| {
        let home = c.windows?;
        exe.contains(&format!("\\{}\\application\\", home.to_ascii_lowercase()))
            .then_some(home)
    })
}

#[cfg(test)]
mod tests {
    use super::{chromium_home_on_mac, chromium_home_on_windows, id_for, profiles_in};

    #[test]
    fn every_chromium_channel_on_windows_reads_its_own_profiles() {
        for (exe, home) in [
            (
                r"C:\Program Files\Google\Chrome\Application\chrome.exe",
                Some(r"Google\Chrome"),
            ),
            (
                r"C:\Program Files\Google\Chrome Beta\Application\chrome.exe",
                Some(r"Google\Chrome Beta"),
            ),
            (
                r"C:\Program Files\Google\Chrome Dev\Application\chrome.exe",
                Some(r"Google\Chrome Dev"),
            ),
            (
                r"C:\Users\a\AppData\Local\Google\Chrome SxS\Application\chrome.exe",
                Some(r"Google\Chrome SxS"),
            ),
            (
                r"C:\Users\a\AppData\Local\Chromium\Application\chrome.exe",
                Some("Chromium"),
            ),
            (
                r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe",
                Some(r"Microsoft\Edge"),
            ),
            (
                r"C:\Program Files (x86)\Microsoft\Edge Beta\Application\msedge.exe",
                Some(r"Microsoft\Edge Beta"),
            ),
            (
                r"C:\Program Files (x86)\Microsoft\Edge Dev\Application\msedge.exe",
                Some(r"Microsoft\Edge Dev"),
            ),
            (
                r"C:\Users\a\AppData\Local\Microsoft\Edge SxS\Application\msedge.exe",
                Some(r"Microsoft\Edge SxS"),
            ),
            (
                r"C:\Program Files\BraveSoftware\Brave-Browser\Application\brave.exe",
                Some(r"BraveSoftware\Brave-Browser"),
            ),
            (
                r"C:\Program Files\BraveSoftware\Brave-Browser-Beta\Application\brave.exe",
                Some(r"BraveSoftware\Brave-Browser-Beta"),
            ),
            (
                r"C:\Users\a\AppData\Local\Vivaldi\Application\vivaldi.exe",
                Some("Vivaldi"),
            ),
            (r"C:\Program Files\Mozilla Firefox\firefox.exe", None),
            (r"D:\Portable\chrome\chrome.exe", None),
        ] {
            assert_eq!(chromium_home_on_windows(exe), home, "{exe}");
        }
    }

    #[test]
    fn every_chromium_channel_on_a_mac_reads_its_own_profiles() {
        for (bundle, home) in [
            ("Google Chrome.app", Some("Google/Chrome")),
            ("Google Chrome Canary.app", Some("Google/Chrome Canary")),
            ("Microsoft Edge Beta.app", Some("Microsoft Edge Beta")),
            ("Microsoft Edge Dev.app", Some("Microsoft Edge Dev")),
            ("Microsoft Edge Canary.app", Some("Microsoft Edge Canary")),
            (
                "Brave Browser Beta.app",
                Some("BraveSoftware/Brave-Browser-Beta"),
            ),
            (
                "Brave Browser Nightly.app",
                Some("BraveSoftware/Brave-Browser-Nightly"),
            ),
            (
                "Brave Browser Dev.app",
                Some("BraveSoftware/Brave-Browser-Dev"),
            ),
            (
                "Google Chrome for Testing.app",
                Some("Google/Chrome for Testing"),
            ),
            ("Chrome.app", Some("Google/Chrome")),
            ("Chromium.app", Some("Chromium")),
            ("Arc.app", Some("Arc/User Data")),
            ("Archive Utility.app", None),
            ("Firefox.app", None),
        ] {
            assert_eq!(chromium_home_on_mac(bundle), home, "{bundle}");
        }
    }

    /// The directory name is what the browser is launched with; the name in `Local State` is
    /// only what the person reads. A profile whose name was never set has an empty one there,
    /// and an empty label leaves a nameless row in the picker.
    #[test]
    fn a_profile_with_no_name_of_its_own_is_read_by_its_directory() {
        let profiles = profiles_in(
            r#"{"profile":{"info_cache":{
                "Default":{"name":"Personal"},
                "Profile 2":{"name":""},
                "Profile 3":{}
            }}}"#,
        );

        assert_eq!(profiles.len(), 3);
        let named: Vec<&str> = profiles.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(
            named,
            vec!["Personal", "Profile 2", "Profile 3"],
            "sorted by what the person reads"
        );

        let personal = &profiles[0];
        assert_eq!(personal.id, "Default");
        assert_eq!(
            personal.args,
            vec!["--profile-directory=Default".to_owned()],
            "the directory launches it, whatever it is called"
        );
    }

    /// Chromium rewrites this file on every run and a half-written one is what a crash leaves
    /// behind. Nothing there means no profiles, not no browser.
    #[test]
    fn a_local_state_that_says_nothing_useful_yields_no_profiles() {
        assert!(profiles_in("{ not json at all").is_empty());
        assert!(profiles_in("{}").is_empty());
        assert!(profiles_in(r#"{"profile":{}}"#).is_empty());
        assert!(profiles_in(r#"{"profile":{"info_cache":[]}}"#).is_empty());
    }

    #[test]
    fn identifiers_match_the_ones_1_x_wrote_into_its_rules() {
        assert_eq!(id_for("Google Chrome"), "google-chrome");
        assert_eq!(
            id_for("Vivaldi.XZTTKVPR7OU6S6FGTKA6S5FOHM"),
            "vivaldi-xzttkvpr7ou6s6fgtka6s5fohm"
        );
        assert_eq!(
            id_for("firefox-308046b0af4a39cb"),
            "firefox-308046b0af4a39cb"
        );
        assert_eq!(id_for("Brave"), "brave");
    }

    /// The same scheme reaches both systems: what Windows applies to a registry key, macOS
    /// applies to a bundle identifier, and 1.x wrote exactly these into its rules there.
    #[test]
    fn a_bundle_identifier_yields_the_id_1_x_wrote_on_a_mac() {
        assert_eq!(id_for("com.apple.Safari"), "com-apple-safari");
        assert_eq!(id_for("org.mozilla.firefox"), "org-mozilla-firefox");
        assert_eq!(id_for("com.vivaldi.Vivaldi"), "com-vivaldi-vivaldi");
        assert_eq!(id_for("com.google.Chrome"), "com-google-chrome");
    }
}
