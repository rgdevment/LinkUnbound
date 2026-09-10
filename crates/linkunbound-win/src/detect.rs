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
            if !found.iter().any(|b| b.id == browser.id) {
                found.push(browser);
            }
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
    let Ok(state) = serde_json::from_str::<serde_json::Value>(&raw) else {
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
    fn neither_internet_explorer_nor_ourselves_count_as_destinations() {
        assert!(!is_destination(
            "IEXPLORE.EXE",
            r"C:\Program Files\Internet Explorer\iexplore.exe"
        ));
        assert!(!is_destination("LinkUnbound", r"C:\Apps\linkunbound.exe"));
        assert!(is_destination(
            "firefox-308046b0af4a39cb",
            r"C:firefox.exe"
        ));
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
