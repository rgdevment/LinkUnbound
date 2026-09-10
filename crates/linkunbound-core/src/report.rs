use crate::{Preferences, RuleSet, Scope};

/// A report is meant to be pasted into a public issue, so nothing that names a
/// place the user has been may survive it: the host stays, the rest goes.
#[must_use]
pub fn redact(url: &str) -> String {
    let Some((scheme, rest)) = url.split_once("://") else {
        return "[redactado]".to_owned();
    };
    let host = rest
        .split(['/', '?', '#'])
        .next()
        .unwrap_or_default()
        .rsplit('@')
        .next()
        .unwrap_or_default();
    if host.is_empty() {
        return "[redactado]".to_owned();
    }
    if rest.len() > host.len() {
        return format!("{scheme}://{host}/…");
    }
    format!("{scheme}://{host}")
}

fn describe(scope: &Scope) -> String {
    match scope {
        Scope::Any => "cualquier enlace".to_owned(),
        Scope::Url(u) => format!("el enlace {}", redact(u)),
        Scope::Host(h) => format!("el host {h}"),
        Scope::Site(d) => format!("el sitio {d}"),
    }
}

/// Everything a maintainer needs to answer "why does it not open my links",
/// and nothing that would embarrass the person who sends it.
#[must_use]
pub fn diagnostics(
    version: &str,
    system: &[(String, String)],
    rules: &RuleSet,
    prefs: &Preferences,
) -> String {
    let mut out = format!("LinkUnbound {version}\n\n## Sistema\n");
    for (key, value) in system {
        out.push_str(&format!("- {key}: {value}\n"));
    }

    out.push_str(&format!(
        "\n## Preferencias\n- tema: {:?}\n- idioma: {:?}\n- atajo: {}\n- bandeja oculta: {}\n- avisa al aplicar una regla: {}\n",
        prefs.theme,
        prefs.locale,
        prefs.shortcut.as_deref().unwrap_or("ninguno"),
        prefs.hide_tray,
        prefs.notify_on_rule,
    ));

    out.push_str(&format!("\n## Reglas ({})\n", rules.rules.len()));
    for rule in &rules.rules {
        let origin = rule
            .source_app
            .as_ref()
            .map_or(String::new(), |a| format!(" desde {a}"));
        let private = if rule.private { " en privado" } else { "" };
        out.push_str(&format!(
            "- {}{origin} → {}{private}\n",
            describe(&rule.scope),
            rule.target.browser_id,
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{diagnostics, redact};
    use crate::{Preferences, Rule, RuleSet, Scope, Target};

    #[test]
    fn a_link_keeps_its_host_and_loses_everywhere_it_leads() {
        assert_eq!(
            redact("https://intranet.corp/informes/2026/sueldos?id=44"),
            "https://intranet.corp/…"
        );
        assert_eq!(redact("https://example.test"), "https://example.test");
    }

    /// Credentials in the authority would ride along with the host otherwise.
    #[test]
    fn a_password_in_the_url_never_reaches_the_report() {
        let out = redact("https://ana:hunter2@intranet.corp/x");
        assert!(!out.contains("hunter2"));
        assert!(!out.contains("ana"));
        assert!(out.contains("intranet.corp"));
    }

    #[test]
    fn something_that_is_not_a_link_is_dropped_whole() {
        assert_eq!(redact("C:/Users/Ana/secreto.pdf"), "[redactado]");
        assert_eq!(redact("https://"), "[redactado]");
    }

    #[test]
    fn the_report_names_the_rules_without_naming_where_they_lead() {
        let rules = RuleSet {
            schema_version: 2,
            rules: vec![Rule {
                id: "url:x".to_owned(),
                scope: Scope::Url("https://mail.corp/inbox/42?token=abc".to_owned()),
                source_app: Some("teams".to_owned()),
                target: Target {
                    browser_id: "firefox".to_owned(),
                    profile_id: None,
                },
                private: true,
            }],
        };
        let out = diagnostics("2.0.0", &[], &rules, &Preferences::default());
        assert!(out.contains("mail.corp"));
        assert!(!out.contains("token=abc"));
        assert!(!out.contains("inbox"));
        assert!(out.contains("desde teams"));
        assert!(out.contains("en privado"));
    }

    #[test]
    fn the_report_carries_the_system_facts_it_was_given() {
        let facts = vec![("navegador predeterminado".to_owned(), "no".to_owned())];
        let out = diagnostics(
            "2.0.0",
            &facts,
            &RuleSet::default(),
            &Preferences::default(),
        );
        assert!(out.contains("LinkUnbound 2.0.0"));
        assert!(out.contains("- navegador predeterminado: no"));
        assert!(out.contains("## Reglas (0)"));
    }
}
