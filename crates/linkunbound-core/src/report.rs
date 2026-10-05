use crate::{Preferences, RuleSet, Scope, Strings};

/// A report is meant to be pasted into a public issue, so nothing that names a
/// place the user has been may survive it: the host stays, the rest goes.
#[must_use]
pub fn redact(url: &str, instead: &str) -> String {
    // Parsed, not split on `/`: a browser reads `\` as a separator too, and a path written with
    // backslashes rode along whole as part of the host.
    let Ok(parsed) = url::Url::parse(url) else {
        return instead.to_owned();
    };
    let Some(host) = parsed.host_str() else {
        return instead.to_owned();
    };
    let at = match parsed.port() {
        Some(port) => format!("{}://{host}:{port}", parsed.scheme()),
        None => format!("{}://{host}", parsed.scheme()),
    };
    let goes_further =
        parsed.path() != "/" || parsed.query().is_some() || parsed.fragment().is_some();
    if goes_further {
        format!("{at}/…")
    } else {
        at
    }
}

fn describe(scope: &Scope, words: &Strings) -> String {
    match scope {
        Scope::Any => words.report_any_link.to_owned(),
        Scope::Url(u) => Strings::fill(words.report_the_link, &redact(u, words.report_redacted)),
        Scope::Host(h) => Strings::fill(words.report_the_host, h),
        Scope::Site(d) => Strings::fill(words.report_the_site, d),
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
    words: &Strings,
    errors: &[String],
) -> String {
    let mut out = format!("LinkUnbound {version}\n\n## {}\n", words.report_system);
    for (key, value) in system {
        out.push_str(&format!("- {key}: {value}\n"));
    }

    out.push_str(&format!(
        "\n## {}\n- {}: {:?}\n- {}: {:?}\n- {}: {}\n- {}: {}\n",
        words.report_prefs,
        words.report_theme,
        prefs.theme,
        words.report_locale,
        prefs.locale,
        words.report_shortcut,
        prefs.shortcut.as_deref().unwrap_or(words.report_nothing),
        words.report_notify,
        prefs.notify_on_rule,
    ));

    out.push_str(&format!(
        "\n## {} ({})\n",
        words.report_rules,
        rules.rules.len()
    ));
    for rule in &rules.rules {
        let origin = rule
            .source_app
            .as_ref()
            .map_or(String::new(), |a| Strings::fill(words.report_from, a));
        let private = if rule.private {
            words.report_private
        } else {
            ""
        };
        out.push_str(&format!(
            "- {}{origin} → {}{private}\n",
            describe(&rule.scope, words),
            rule.target.browser_id,
        ));
    }

    out.push_str(&format!(
        "
## {} ({})
",
        words.report_errors,
        errors.len()
    ));
    for line in errors {
        out.push_str(&format!(
            "- {line}
"
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{diagnostics, redact};
    use crate::{Language, Preferences, Rule, RuleSet, Scope, Target};

    const GONE: &str = "[redactado]";

    #[test]
    fn a_link_keeps_its_host_and_loses_everywhere_it_leads() {
        assert_eq!(
            redact("https://intranet.corp/informes/2026/sueldos?id=44", GONE),
            "https://intranet.corp/…"
        );
        assert_eq!(redact("https://example.test", GONE), "https://example.test");
    }

    /// Credentials in the authority would ride along with the host otherwise.
    #[test]
    fn a_password_in_the_url_never_reaches_the_report() {
        let out = redact("https://ana:hunter2@intranet.corp/x", GONE);
        assert!(!out.contains("hunter2"));
        assert!(!out.contains("ana"));
        assert!(out.contains("intranet.corp"));
    }

    #[test]
    fn a_path_written_with_backslashes_is_cut_like_any_other() {
        let out = redact(r"https://intranet.corp\reports\salaries", GONE);
        assert_eq!(out, "https://intranet.corp/…");
        assert_eq!(
            redact("https://intranet.corp:8443/x", GONE),
            "https://intranet.corp:8443/…"
        );
        assert_eq!(redact("https://a.test/?q=1", GONE), "https://a.test/…");
    }

    #[test]
    fn something_that_is_not_a_link_is_dropped_whole() {
        assert_eq!(redact("C:/Users/Ana/secreto.pdf", GONE), GONE);
        assert_eq!(redact("https://", GONE), GONE);
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
        let out = diagnostics(
            "2.0.0",
            &[],
            &rules,
            &Preferences::default(),
            &Language::Spanish.strings(),
            &[],
        );
        assert!(out.contains("mail.corp"));
        assert!(!out.contains("token=abc"));
        assert!(!out.contains("inbox"));
        assert!(out.contains("desde teams"));
        assert!(out.contains("en privado"));
    }

    #[test]
    fn the_report_is_written_in_the_language_that_was_chosen() {
        let rules = RuleSet {
            schema_version: 2,
            rules: vec![Rule {
                id: "site:x".to_owned(),
                scope: Scope::Site("example.test".to_owned()),
                source_app: Some("teams".to_owned()),
                target: Target {
                    browser_id: "firefox".to_owned(),
                    profile_id: None,
                },
                private: true,
            }],
        };
        let out = diagnostics(
            "2.0.0",
            &[],
            &rules,
            &Preferences::default(),
            &Language::English.strings(),
            &[],
        );
        assert!(out.contains("## System"), "{out}");
        assert!(out.contains("## Preferences"));
        assert!(out.contains("## Rules (1)"));
        assert!(out.contains("the site example.test from teams"));
        assert!(out.contains("privately"));
        assert!(!out.contains("desde"));
        assert!(!out.contains("Sistema"));
    }

    #[test]
    fn the_report_carries_the_system_facts_it_was_given() {
        let facts = vec![("navegador predeterminado".to_owned(), "no".to_owned())];
        let out = diagnostics(
            "2.0.0",
            &facts,
            &RuleSet::default(),
            &Preferences::default(),
            &Language::Spanish.strings(),
            &[],
        );
        assert!(out.contains("LinkUnbound 2.0.0"));
        assert!(out.contains("- navegador predeterminado: no"));
        assert!(out.contains("## Reglas (0)"));
        assert!(out.contains("## Errores recientes (0)"));
    }

    #[test]
    fn the_report_ends_with_the_errors_the_journal_kept() {
        let errors =
            vec!["2026-10-04T23:00:00Z [launch] firefox: the browser would not start".to_owned()];
        let out = diagnostics(
            "2.0.0",
            &[],
            &RuleSet::default(),
            &Preferences::default(),
            &Language::English.strings(),
            &errors,
        );
        assert!(
            out.ends_with(
                "## Recent errors (1)
- 2026-10-04T23:00:00Z [launch] firefox: the browser would not start
"
            ),
            "{out}"
        );
    }
}
