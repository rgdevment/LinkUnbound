//! The installed families settings offers for the picker and itself, «System» first.

use linkunbound_core::fonts::{Family, offered, shown};
use linkunbound_shell::font;
use serde::Serialize;

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct Offered {
    /// The English name, the one saved in the preferences.
    name: String,
    /// The name in the person's language.
    label: String,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct Fonts {
    /// The name shown beside «System», which is what an absent choice draws with.
    system: String,
    families: Vec<Offered>,
}

fn listed(system: &str, families: Vec<Family>) -> Fonts {
    let families = offered(families);
    let system = families
        .iter()
        .find(|one| one.name.eq_ignore_ascii_case(system))
        .map_or_else(|| shown(system).to_owned(), |one| one.local.clone());
    Fonts {
        system,
        families: families
            .into_iter()
            .map(|one| Offered {
                name: one.name,
                label: one.local,
            })
            .collect(),
    }
}

/// Async: asking for every installed family takes long enough to stall the window otherwise.
#[tauri::command(async)]
pub fn fonts_list() -> Fonts {
    listed(&font::system(), font::families())
}

#[cfg(test)]
mod tests {
    use super::{Offered, fonts_list, listed};
    use linkunbound_core::fonts::Family;

    fn family(name: &str, local: &str, symbol: bool) -> Family {
        Family {
            name: name.to_owned(),
            local: local.to_owned(),
            symbol,
        }
    }

    #[test]
    fn the_list_offers_text_families_by_their_local_name_and_saves_the_english_one() {
        let fonts = listed(
            "Segoe UI",
            vec![
                family("Yu Gothic", "游ゴシック", false),
                family("Segoe UI", "Segoe UI", false),
                family("Marlett", "Marlett", true),
            ],
        );
        assert_eq!(fonts.system, "Segoe UI");
        assert_eq!(
            fonts.families,
            [
                Offered {
                    name: "Segoe UI".to_owned(),
                    label: "Segoe UI".to_owned(),
                },
                Offered {
                    name: "Yu Gothic".to_owned(),
                    label: "游ゴシック".to_owned(),
                },
            ]
        );
    }

    #[test]
    fn system_is_named_even_when_it_is_hidden_from_the_list() {
        assert_eq!(listed(".SF NS", Vec::new()).system, "SF Pro");
        assert_eq!(listed("Tahoma", Vec::new()).system, "Tahoma");
    }

    #[cfg(windows)]
    #[test]
    fn this_windows_names_its_system_family_and_offers_no_pictures() {
        let fonts = fonts_list();
        assert!(fonts.families.iter().any(|one| one.label == fonts.system));
        assert!(fonts.families.iter().any(|one| one.name == "Arial"));
        assert!(!fonts.families.iter().any(|one| one.name == "Marlett"));
    }
}
