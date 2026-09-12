use crate::browser::LaunchRefused;
use crate::launch::LaunchError;
use crate::prefs::Locale;

macro_rules! catalogue {
    ($($key:ident: $es:literal | $en:literal,)*) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub struct Strings {
            $(pub $key: &'static str,)*
        }

        const SPANISH: Strings = Strings { $($key: $es,)* };
        const ENGLISH: Strings = Strings { $($key: $en,)* };

        impl Strings {
            #[must_use]
            pub fn pairs(&self) -> Vec<(&'static str, &'static str)> {
                vec![$((stringify!($key), self.$key),)*]
            }
        }
    };
}

catalogue! {
    tray_settings: "Ajustes" | "Settings",
    tray_quit: "Salir" | "Exit",
    picker_from: "Desde {}" | "From {}",
    picker_remember: "Recordar esta elección" | "Remember this choice",
    picker_private: "privada" | "private",
    reach_once: "Solo esta vez" | "Just this time",
    reach_url: "Esta URL" | "This URL",
    reach_subdomain: "Subdominio" | "Subdomain",
    reach_site: "Todo el sitio" | "The whole site",
    reach_from_app: "Desde {}" | "From {}",
    wrapper_unresolved: "Enlace envuelto: el destino real se conocerá al abrirlo" | "Wrapped link: the real destination is known once it opens",
    no_browsers: "Ningún navegador que ofrecer. Añade uno en Ajustes." | "No browser to offer. Add one in Settings.",
    notice_opened: "Abierto en {}" | "Opened in {}",
    notice_by_rule: "Una regla decidió por {}" | "A rule decided for {}",
    notice_undo: "Deshacer" | "Undo",
    fail_not_remembered: "No se pudo guardar la regla" | "The rule could not be saved",
    fail_unknown_browser: "Ese navegador ya no está configurado" | "That browser is no longer configured",
    fail_profile_gone: "El perfil {} ya no existe" | "The profile {} is no longer there",
    fail_private_impossible: "Este navegador no puede abrir una ventana privada" | "This browser cannot open a private window",
    fail_spawn: "El navegador no arrancó" | "The browser would not start",
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    Spanish,
    English,
}

impl Language {
    #[must_use]
    pub const fn strings(self) -> Strings {
        match self {
            Self::Spanish => SPANISH,
            Self::English => ENGLISH,
        }
    }

    #[must_use]
    pub fn of_tag(tag: Option<&str>) -> Self {
        match tag {
            Some(tag) if tag.to_ascii_lowercase().starts_with("es") => Self::Spanish,
            _ => Self::English,
        }
    }

    #[must_use]
    pub fn chosen(locale: Locale) -> Self {
        match locale {
            Locale::Spanish => Self::Spanish,
            Locale::English => Self::English,
            Locale::System => Self::of_tag(sys_locale::get_locale().as_deref()),
        }
    }
}

impl Strings {
    #[must_use]
    pub fn fill(template: &str, value: &str) -> String {
        template.replace("{}", value)
    }

    /// The `Display` of these stays English for the log; the picker cannot.
    #[must_use]
    pub fn on_failure(&self, failure: &LaunchError) -> String {
        match failure {
            LaunchError::UnknownBrowser(_) => self.fail_unknown_browser.to_owned(),
            LaunchError::Refused(LaunchRefused::ProfileGone(profile)) => {
                Self::fill(self.fail_profile_gone, profile)
            }
            LaunchError::Refused(LaunchRefused::PrivateImpossible) => {
                self.fail_private_impossible.to_owned()
            }
            LaunchError::Spawn(_) => self.fail_spawn.to_owned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ENGLISH, Language, SPANISH, Strings};
    use crate::browser::LaunchRefused;
    use crate::launch::LaunchError;
    use crate::prefs::Locale;

    #[test]
    fn an_explicit_choice_is_obeyed_and_the_rest_is_asked_of_the_system() {
        assert_eq!(Language::chosen(Locale::Spanish), Language::Spanish);
        assert_eq!(Language::chosen(Locale::English), Language::English);
    }

    #[test]
    fn only_a_spanish_system_gets_spanish() {
        assert_eq!(Language::of_tag(Some("es-CL")), Language::Spanish);
        assert_eq!(Language::of_tag(Some("es")), Language::Spanish);
        assert_eq!(Language::of_tag(Some("en-US")), Language::English);
        assert_eq!(Language::of_tag(Some("pt-BR")), Language::English);
        assert_eq!(Language::of_tag(None), Language::English);
    }

    #[test]
    fn a_placeholder_present_in_one_language_is_present_in_both() {
        for ((key, spanish), (_, english)) in SPANISH.pairs().into_iter().zip(ENGLISH.pairs()) {
            assert_eq!(
                spanish.contains("{}"),
                english.contains("{}"),
                "{key} disagrees about its placeholder"
            );
        }
    }

    /// Words that are the same in both languages, and would read as an
    /// oversight rather than as a translation.
    const SHARED: [&str; 2] = ["picker_private", "tray_settings"];

    /// A string left as the Spanish one ships green otherwise: it is not blank,
    /// it keeps its placeholders, and no test compares the two languages.
    #[test]
    fn an_english_string_is_not_just_the_spanish_one() {
        for ((key, spanish), (_, english)) in SPANISH.pairs().into_iter().zip(ENGLISH.pairs()) {
            if SHARED.contains(&key) {
                continue;
            }
            assert_ne!(
                spanish, english,
                "{key} reads the same in both languages: untranslated, or a shared word to list"
            );
        }
    }

    /// The reaches sit side by side in one row, so a pair swapped in one
    /// language alone points the user at the wrong scope without looking odd.
    #[test]
    fn each_reach_says_the_same_thing_in_both_languages() {
        for (spanish, english, widening) in [
            (SPANISH.reach_url, ENGLISH.reach_url, "url"),
            (
                SPANISH.reach_subdomain,
                ENGLISH.reach_subdomain,
                "subdomain",
            ),
            (SPANISH.reach_site, ENGLISH.reach_site, "site"),
        ] {
            let es = spanish.to_lowercase();
            let en = english.to_lowercase();
            let agreed = match widening {
                "url" => es.contains("url") && en.contains("url"),
                "subdomain" => es.contains("subdominio") && en.contains("subdomain"),
                _ => es.contains("sitio") && en.contains("site"),
            };
            assert!(
                agreed,
                "the {widening} reach disagrees: {spanish} / {english}"
            );
        }
    }

    #[test]
    fn nothing_ships_blank_or_untranslated() {
        for ((key, spanish), (_, english)) in SPANISH.pairs().into_iter().zip(ENGLISH.pairs()) {
            assert!(!spanish.trim().is_empty(), "{key} is blank in Spanish");
            assert!(!english.trim().is_empty(), "{key} is blank in English");
        }
    }

    #[test]
    fn a_failure_is_reported_in_the_users_language_and_keeps_the_detail() {
        let gone = LaunchError::Refused(LaunchRefused::ProfileGone("Trabajo".to_owned()));
        assert_eq!(SPANISH.on_failure(&gone), "El perfil Trabajo ya no existe");
        assert_eq!(
            ENGLISH.on_failure(&gone),
            "The profile Trabajo is no longer there"
        );
    }

    #[test]
    fn filling_a_template_puts_the_value_where_the_marker_was() {
        assert_eq!(
            Strings::fill(SPANISH.notice_opened, "Chrome"),
            "Abierto en Chrome"
        );
        assert_eq!(
            Strings::fill(ENGLISH.notice_by_rule, "github.com"),
            "A rule decided for github.com"
        );
    }
}
