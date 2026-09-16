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

#[cfg(test)]
mod tests {
    use super::{id_for, profiles_in};

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
