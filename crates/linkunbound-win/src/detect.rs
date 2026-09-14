use linkunbound_core::{Browser, Profile, private_flag_for};
use winreg::RegKey;
use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};

const START_MENU: &str = r"Software\Clients\StartMenuInternet";

/// Internet Explorer still lists itself on Windows 11 but only redirects to
/// Edge, and we register under this same key: neither is a destination.
const NOT_DESTINATIONS: [&str; 4] = [
    "iexplore",
    "iexplore.exe",
    "internet explorer",
    "linkunbound",
];

fn is_destination(key_name: &str, exe: &str) -> bool {
    let key = key_name.to_ascii_lowercase();
    let exe = exe.to_ascii_lowercase();
    !NOT_DESTINATIONS
        .iter()
        .any(|blocked| key == *blocked || exe.contains(blocked))
}

fn unquote(command: &str) -> String {
    let trimmed = command.trim();
    trimmed
        .strip_prefix('"')
        .and_then(|rest| rest.split_once('"').map(|(exe, _)| exe))
        .unwrap_or(trimmed)
        .to_owned()
}

/// The 1.x identifier scheme, kept byte for byte: rules migrated from it store
/// this string, and an id that does not match points a rule at nothing.
fn normalise_id(key_name: &str) -> String {
    let mut out = String::with_capacity(key_name.len());
    for ch in key_name.to_ascii_lowercase().chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch);
        } else if !out.ends_with('-') {
            out.push('-');
        }
    }
    out.trim_matches('-').to_owned()
}

/// The same browser is listed under both hives when it was installed for the machine and then
/// updated for the user, and the picker would offer it twice.
fn add_unseen(found: &mut Vec<Browser>, browser: Browser) {
    if !found.iter().any(|b| b.id == browser.id) {
        found.push(browser);
    }
}

fn read_entry(root: &RegKey, key_name: &str) -> Option<Browser> {
    let key = root.open_subkey(key_name).ok()?;
    let exe = key
        .open_subkey(r"shell\open\command")
        .ok()
        .and_then(|c| c.get_value::<String, _>("").ok())
        .map(|c| unquote(&c))?;
    if exe.is_empty() {
        return None;
    }
    if !is_destination(key_name, &exe) {
        return None;
    }
    let name = key
        .get_value::<String, _>("")
        .ok()
        .filter(|n| !n.is_empty())
        .unwrap_or_else(|| key_name.to_owned());

    Some(Browser {
        id: normalise_id(key_name),
        name,
        private_flag: private_flag_for(&exe).map(str::to_owned),
        profiles: chromium_profiles(&exe),
        exe,
        extra_args: Vec::new(),
        icon_path: None,
        custom: false,
        hidden: false,
    })
}

/// Both hives list browsers, and a machine-wide install and a per-user one can
/// name the same browser: the first entry seen wins.
#[must_use]
pub fn installed_browsers() -> Vec<Browser> {
    let mut found: Vec<Browser> = Vec::new();
    for hive in [HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE] {
        let Ok(root) = RegKey::predef(hive).open_subkey(START_MENU) else {
            continue;
        };
        for key_name in root.enum_keys().flatten() {
            let Some(browser) = read_entry(&root, &key_name) else {
                continue;
            };
            add_unseen(&mut found, browser);
        }
    }
    found.sort_by_key(|b| b.name.to_lowercase());
    found
}

fn user_data_dir(exe: &str) -> Option<std::path::PathBuf> {
    let local = std::env::var_os("LOCALAPPDATA")?;
    let needle = exe.to_ascii_lowercase();
    let suffix = if needle.contains("msedge") {
        r"Microsoft\Edge\User Data"
    } else if needle.contains("brave") {
        r"BraveSoftware\Brave-Browser\User Data"
    } else if needle.contains("vivaldi") {
        r"Vivaldi\User Data"
    } else if needle.contains("chrome") {
        r"Google\Chrome\User Data"
    } else {
        return None;
    };
    Some(std::path::Path::new(&local).join(suffix))
}

/// Chromium keeps the human names of its profiles in `Local State`; the
/// directory names it launches with are the keys of that map.
#[must_use]
pub fn chromium_profiles(exe: &str) -> Vec<Profile> {
    let Some(dir) = user_data_dir(exe) else {
        return Vec::new();
    };
    let Ok(raw) = std::fs::read_to_string(dir.join("Local State")) else {
        return Vec::new();
    };
    profiles_from(&raw)
}

fn profiles_from(raw: &str) -> Vec<Profile> {
    let Ok(state) = serde_json::from_str::<serde_json::Value>(raw) else {
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

#[cfg(test)]
mod tests {
    use super::*;

    /// We register under this same key, so without this the picker offers itself as a
    /// destination and a link handed to it comes straight back. Either half is reason enough:
    /// Internet Explorer matches by key name, and a copy of ours installed anywhere matches by
    /// path.
    #[test]
    fn neither_ourselves_nor_internet_explorer_is_a_destination() {
        assert!(is_destination(
            "Firefox",
            r"C:\Program Files\Mozilla Firefoxirefox.exe"
        ));
        assert!(is_destination(
            "Google Chrome",
            r"C:\Program Files\Google\Chrome\chrome.exe"
        ));
        assert!(is_destination("firefox-308046b0af4a39cb", r"C:firefox.exe"));

        assert!(
            !is_destination("IEXPLORE.EXE", r"C:\Program Files\Internet Explorer\ie.exe"),
            "the key name alone has to be enough"
        );
        assert!(
            !is_destination(
                "Some Browser",
                r"C:\Program Files\LinkUnbound\linkunbound-shell.exe"
            ),
            "the path alone has to be enough"
        );
    }

    /// A browser installed for the machine and then updated for the user is listed under both
    /// hives, and the picker would offer it twice.
    #[test]
    fn a_browser_listed_under_both_hives_is_offered_once() {
        let mut found = Vec::new();
        add_unseen(&mut found, named("chrome", "Google Chrome"));
        add_unseen(&mut found, named("chrome", "Google Chrome (user)"));
        add_unseen(&mut found, named("firefox", "Firefox"));

        assert_eq!(found.len(), 2);
        assert_eq!(found[0].name, "Google Chrome", "the first one seen stays");
        assert_eq!(found[1].id, "firefox");
    }

    /// The directory name is what the browser is launched with; the name in `Local State` is
    /// only what the person reads. A profile whose name was never set has an empty one there,
    /// and an empty label leaves a nameless row in the picker.
    #[test]
    fn a_profile_with_no_name_of_its_own_is_read_by_its_directory() {
        let profiles = profiles_from(
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
        assert!(profiles_from("{ not json at all").is_empty());
        assert!(profiles_from("{}").is_empty());
        assert!(profiles_from(r#"{"profile":{}}"#).is_empty());
        assert!(profiles_from(r#"{"profile":{"info_cache":[]}}"#).is_empty());
    }

    /// `read_entry` reads what Windows wrote, so it is given keys shaped the way Windows shapes
    /// them rather than a stand-in.
    #[test]
    fn an_entry_becomes_a_browser_and_a_useless_one_is_left_out() {
        let root_path = r"Software\LinkUnbound-test\detect";
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let _ = hkcu.delete_subkey_all(root_path);
        let (root, _) = hkcu.create_subkey(root_path).expect("a place to write");

        let write = |key_name: &str, display: &str, command: &str| {
            let (key, _) = root.create_subkey(key_name).expect("the entry");
            key.set_value("", &display).expect("the display name");
            let (cmd, _) = key
                .create_subkey(r"shell\open\command")
                .expect("the command");
            cmd.set_value("", &command).expect("the command line");
        };

        write(
            "Firefox",
            "Mozilla Firefox",
            r#""C:\Program Files\Mozilla Firefoxirefox.exe" -osint -url "%1""#,
        );
        write("Nameless", "", r#""C:\Apps\other.exe""#);
        write("IEXPLORE.EXE", "Internet Explorer", r#""C:\Apps\ie.exe""#);
        write("Broken", "No Command At All", "");

        let firefox = read_entry(&root, "Firefox").expect("a browser");
        assert_eq!(firefox.name, "Mozilla Firefox");
        assert_eq!(firefox.id, "firefox");
        assert_eq!(
            firefox.private_flag.as_deref(),
            Some("-private-window"),
            "the private switch comes from the executable, not the key"
        );

        assert_eq!(
            read_entry(&root, "Nameless").expect("still a browser").name,
            "Nameless",
            "an entry with no display name is read by its key"
        );
        assert!(
            read_entry(&root, "IEXPLORE.EXE").is_none(),
            "not a destination"
        );
        assert!(
            read_entry(&root, "Broken").is_none(),
            "an entry with no command cannot open anything"
        );
        assert!(read_entry(&root, "NotThere").is_none());

        let _ = hkcu.delete_subkey_all(root_path);
    }

    fn named(id: &str, name: &str) -> Browser {
        Browser {
            id: id.to_owned(),
            name: name.to_owned(),
            exe: "x.exe".to_owned(),
            profiles: Vec::new(),
            extra_args: Vec::new(),
            private_flag: None,
            icon_path: None,
            custom: false,
            hidden: false,
        }
    }

    #[test]
    fn a_quoted_command_yields_the_executable_alone() {
        assert_eq!(
            unquote("\"C:\\Program Files\\Mozilla Firefox\\firefox.exe\" -osint -url \"%1\""),
            r"C:\Program Files\Mozilla Firefox\firefox.exe"
        );
    }

    #[test]
    fn an_unquoted_command_survives_too() {
        assert_eq!(unquote(r"C:\ff\firefox.exe"), r"C:\ff\firefox.exe");
    }

    #[test]
    fn identifiers_match_the_ones_1_x_wrote_into_its_rules() {
        assert_eq!(normalise_id("Google Chrome"), "google-chrome");
        assert_eq!(
            normalise_id("Vivaldi.XZTTKVPR7OU6S6FGTKA6S5FOHM"),
            "vivaldi-xzttkvpr7ou6s6fgtka6s5fohm"
        );
        assert_eq!(
            normalise_id("firefox-308046b0af4a39cb"),
            "firefox-308046b0af4a39cb"
        );
        assert_eq!(normalise_id("Brave"), "brave");
    }

    #[test]
    fn a_browser_outside_the_chromium_families_reports_no_profiles() {
        assert!(chromium_profiles(r"C:\Program Files\Mozilla Firefox\firefox.exe").is_empty());
    }

    #[test]
    fn whatever_is_installed_carries_a_launchable_path() {
        for browser in installed_browsers() {
            assert!(!browser.exe.is_empty());
            assert!(!browser.id.is_empty());
            assert!(!browser.name.is_empty());
            for profile in &browser.profiles {
                assert!(
                    profile
                        .args
                        .iter()
                        .any(|a| a.starts_with("--profile-directory="))
                );
            }
        }
    }
}

#[cfg(test)]
mod probe {
    use super::installed_browsers;

    /// Run with `cargo test -p linkunbound-win -- --ignored --nocapture` to see
    /// what this machine actually has installed.
    #[test]
    #[ignore = "reports the local machine rather than asserting"]
    fn list_what_is_installed() {
        for browser in installed_browsers() {
            println!("{} [{}] -> {}", browser.name, browser.id, browser.exe);
            println!("  private: {:?}", browser.private_flag);
            for profile in &browser.profiles {
                println!("  profile: {} ({})", profile.name, profile.id);
            }
        }
    }
}
