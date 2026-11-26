//! «System» is the family the platform writes its own dialogs in.

/// What each platform drew with before a font could be chosen, and still does when asking fails.
pub const FALLBACK: &str = if cfg!(windows) {
    "Segoe UI"
} else if cfg!(target_os = "macos") {
    ".SF NS"
} else {
    ""
};

/// Icon fonts that do not always say so: their letters are pictures.
const SYMBOLS: [&str; 5] = [
    "Marlett",
    "Segoe MDL2 Assets",
    "Segoe Fluent Icons",
    "Webdings",
    "Wingdings",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Family {
    /// The English name, the one saved and handed to the renderer.
    pub name: String,
    /// The name in the person's language, the one the list shows.
    pub local: String,
    pub symbol: bool,
}

/// macOS hides its own families behind a leading dot.
#[must_use]
pub fn offerable(family: &Family) -> bool {
    let name = family.name.to_lowercase();
    !family.symbol
        && !name.is_empty()
        && !name.starts_with('.')
        && !SYMBOLS.iter().any(|one| {
            let one = one.to_lowercase();
            name == one || name.starts_with(&format!("{one} "))
        })
}

/// Every installed family the list offers, once each, in the order the person reads them.
#[must_use]
pub fn offered(mut families: Vec<Family>) -> Vec<Family> {
    families.retain(offerable);
    families.sort_by_cached_key(|one| {
        (
            one.local.to_lowercase(),
            one.name.to_lowercase(),
            one.name.clone(),
        )
    });
    families.dedup_by(|one, other| one.name.eq_ignore_ascii_case(&other.name));
    families
}

/// A `FontSubstitutes` key or value may carry a charset after a comma, as in `Helv,0`.
#[must_use]
pub fn substituted(face: &str, entries: &[(String, String)]) -> String {
    let bare = |said: &str| said.split(',').next().unwrap_or(said).trim().to_owned();
    entries
        .iter()
        .find(|(key, _)| key.trim().eq_ignore_ascii_case(face))
        .or_else(|| {
            entries
                .iter()
                .find(|(key, _)| bare(key).eq_ignore_ascii_case(face))
        })
        .map(|(_, value)| bare(value))
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| face.to_owned())
}

/// The family to draw with: the one asked for while it is installed, else the system's.
pub fn worn(
    asked: Option<&str>,
    installed: impl Fn(&str) -> bool,
    system: impl FnOnce() -> String,
) -> String {
    match asked {
        Some(family) if installed(family) => family.to_owned(),
        _ => system(),
    }
}

/// The name shown beside «System»: macOS hides the family it draws with behind a dot.
#[must_use]
pub fn shown(family: &str) -> &str {
    if family.starts_with('.') {
        "SF Pro"
    } else {
        family
    }
}

#[cfg(test)]
mod tests {
    use super::{Family, offerable, offered, shown, substituted, worn};

    fn family(name: &str, local: &str) -> Family {
        Family {
            name: name.to_owned(),
            local: local.to_owned(),
            symbol: false,
        }
    }

    #[test]
    fn pictures_and_hidden_families_are_not_offered_for_text() {
        assert!(offerable(&family("Segoe UI", "Segoe UI")));
        assert!(offerable(&family("Wingdingsby Sans", "Wingdingsby Sans")));
        for name in ["Marlett", "Wingdings 3", "segoe fluent icons", ".SF NS", ""] {
            assert!(!offerable(&family(name, name)), "{name}");
        }
        let symbol = Family {
            symbol: true,
            ..family("Symbol", "Symbol")
        };
        assert!(!offerable(&symbol));
    }

    #[test]
    fn the_list_is_sorted_by_the_shown_name_and_says_each_family_once() {
        let list = offered(vec![
            family("Yu Gothic", "游ゴシック"),
            family("Arial", "Arial"),
            family("calibri", "calibri"),
            family("Calibri", "Calibri"),
            family("Webdings", "Webdings"),
        ]);
        let names: Vec<&str> = list.iter().map(|one| one.name.as_str()).collect();
        assert_eq!(names, ["Arial", "Calibri", "Yu Gothic"]);
    }

    #[test]
    fn a_substitute_replaces_the_face_with_or_without_its_charset() {
        let entries = vec![
            ("MS Shell Dlg".to_owned(), "Tahoma".to_owned()),
            ("Helv,0".to_owned(), "MS Sans Serif,0".to_owned()),
            ("Empty".to_owned(), " ".to_owned()),
        ];
        assert_eq!(substituted("ms shell dlg", &entries), "Tahoma");
        assert_eq!(substituted("Helv", &entries), "MS Sans Serif");
        assert_eq!(substituted("Empty", &entries), "Empty");
        assert_eq!(substituted("Segoe UI", &entries), "Segoe UI");
    }

    #[test]
    fn a_chosen_family_is_worn_only_while_it_is_installed() {
        let installed = |name: &str| name == "Calibri";
        let system = || "Segoe UI".to_owned();
        assert_eq!(worn(Some("Calibri"), installed, system), "Calibri");
        assert_eq!(worn(Some("Gone Sans"), installed, system), "Segoe UI");
        assert_eq!(worn(None, installed, system), "Segoe UI");
    }

    #[test]
    fn a_list_of_nothing_offers_nothing_and_equal_local_names_keep_one_family() {
        assert!(offered(Vec::new()).is_empty());
        let list = offered(vec![family("Beta", "Same"), family("Alpha", "Same")]);
        let names: Vec<&str> = list.iter().map(|one| one.name.as_str()).collect();
        assert_eq!(names, ["Alpha", "Beta"]);
    }

    #[test]
    fn a_substitute_is_looked_for_by_the_whole_key_before_the_bare_one() {
        let entries = vec![
            ("Helv,0".to_owned(), "Wrong".to_owned()),
            ("Helv".to_owned(), "Right".to_owned()),
        ];
        assert_eq!(substituted("Helv", &entries), "Right");
        assert_eq!(substituted("Helv", &[]), "Helv");
    }

    #[test]
    fn the_system_family_is_only_asked_for_when_nothing_installed_was_chosen() {
        let installed = |_: &str| true;
        let never = || panic!("the system family was asked for");
        assert_eq!(worn(Some("Calibri"), installed, never), "Calibri");
    }

    #[test]
    fn the_hidden_mac_family_is_shown_by_its_public_name() {
        assert_eq!(shown(".SF NS"), "SF Pro");
        assert_eq!(shown("Segoe UI"), "Segoe UI");
    }
}
