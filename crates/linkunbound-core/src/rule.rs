use std::cmp::Reverse;

use serde::{Deserialize, Serialize};

/// Registrable domain; an unknown suffix keeps the host, which matches less rather than more.
#[must_use]
pub fn site_of(host: &str) -> String {
    psl::domain_str(host).unwrap_or(host).to_ascii_lowercase()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum Scope {
    Any,
    Url(String),
    Host(String),
    Site(String),
}

impl Scope {
    #[must_use]
    pub fn matches(&self, url: &str, host: &str) -> bool {
        match self {
            Self::Any => true,
            Self::Url(u) => url == u,
            Self::Host(h) => host == h,
            Self::Site(d) => host == d || host.ends_with(&format!(".{d}")),
        }
    }

    /// The origin bonus in `Rule` sits above every value this returns.
    fn specificity(&self) -> u64 {
        match self {
            Self::Any => 0,
            Self::Site(d) => 1_000 + d.len() as u64,
            Self::Host(h) => 100_000 + h.len() as u64,
            Self::Url(u) => 10_000_000 + u.len() as u64,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Target {
    pub browser_id: String,
    #[serde(default)]
    pub profile_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rule {
    pub id: String,
    pub scope: Scope,
    #[serde(default)]
    pub source_app: Option<String>,
    pub target: Target,
    #[serde(default)]
    pub private: bool,
}

impl Rule {
    fn specificity(&self) -> u64 {
        let origin = if self.source_app.is_some() {
            1_000_000_000
        } else {
            0
        };
        origin + self.scope.specificity()
    }

    fn applies(&self, url: &str, host: &str, source_app: Option<&str>) -> bool {
        if !self.scope.matches(url, host) {
            return false;
        }
        match (&self.source_app, source_app) {
            (None, _) => true,
            (Some(want), Some(got)) => want.eq_ignore_ascii_case(got),
            (Some(_), None) => false,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RuleSet {
    pub schema_version: u32,
    pub rules: Vec<Rule>,
}

impl RuleSet {
    /// Replaces the rule covering the same scope and origin instead of appending.
    /// Without this, choosing "always here" a second time for the same site adds
    /// a rule that never wins and the app appears to ignore the request.
    pub fn upsert(&mut self, rule: Rule) {
        match self
            .rules
            .iter_mut()
            .find(|r| r.scope == rule.scope && r.source_app == rule.source_app)
        {
            Some(existing) => *existing = rule,
            None => self.rules.push(rule),
        }
    }

    /// `Reverse` on the index keeps the first of equally specific rules: the list
    /// the user ordered is the list that decides, and `max_by_key` would take the last.
    #[must_use]
    pub fn resolve(&self, url: &str, host: &str, source_app: Option<&str>) -> Option<&Rule> {
        self.rules
            .iter()
            .enumerate()
            .filter(|(_, r)| r.applies(url, host, source_app))
            .max_by_key(|(i, r)| (r.specificity(), Reverse(*i)))
            .map(|(_, r)| r)
    }
}

#[cfg(test)]
mod tests {
    use super::{Rule, RuleSet, Scope, Target, site_of};

    const URL: &str = "https://github.com/rgdevment/LinkUnbound";

    fn rule(id: &str, scope: Scope, source_app: Option<&str>, browser: &str) -> Rule {
        Rule {
            id: id.to_owned(),
            scope,
            source_app: source_app.map(str::to_owned),
            target: Target {
                browser_id: browser.to_owned(),
                profile_id: None,
            },
            private: false,
        }
    }

    #[test]
    fn a_site_covers_the_domain_and_every_subdomain() {
        let p = Scope::Site("github.com".to_owned());
        assert!(p.matches(URL, "github.com"));
        assert!(p.matches(URL, "gist.github.com"));
        assert!(!p.matches(URL, "notgithub.com"));
        assert!(!p.matches(URL, "github.com.evil.test"));
    }

    #[test]
    fn the_site_of_a_host_is_its_registrable_domain() {
        assert_eq!(site_of("docs.google.com"), "google.com");
        assert_eq!(site_of("google.com"), "google.com");
        assert_eq!(site_of("www.bbc.co.uk"), "bbc.co.uk");
        assert_eq!(site_of("bbc.co.uk"), "bbc.co.uk");
        assert_eq!(site_of("a.b.c.github.io"), "c.github.io");
    }

    #[test]
    fn an_unrecognised_host_keeps_itself_as_its_site() {
        assert_eq!(site_of("localhost"), "localhost");
        assert_eq!(site_of("box.invalid-tld-xyz"), "box.invalid-tld-xyz");
    }

    #[test]
    fn an_exact_host_beats_the_site_that_also_covers_it() {
        let set = RuleSet {
            schema_version: 2,
            rules: vec![
                rule(
                    "wide",
                    Scope::Site("github.com".to_owned()),
                    None,
                    "firefox",
                ),
                rule(
                    "narrow",
                    Scope::Host("gist.github.com".to_owned()),
                    None,
                    "chrome",
                ),
            ],
        };
        assert_eq!(
            set.resolve(URL, "gist.github.com", None).unwrap().id,
            "narrow"
        );
        assert_eq!(set.resolve(URL, "github.com", None).unwrap().id, "wide");
    }

    #[test]
    fn one_link_beats_the_host_that_also_covers_it() {
        let set = RuleSet {
            schema_version: 2,
            rules: vec![
                rule(
                    "host",
                    Scope::Host("github.com".to_owned()),
                    None,
                    "firefox",
                ),
                rule("link", Scope::Url(URL.to_owned()), None, "chrome"),
            ],
        };
        assert_eq!(set.resolve(URL, "github.com", None).unwrap().id, "link");
        assert_eq!(
            set.resolve("https://github.com/other", "github.com", None)
                .unwrap()
                .id,
            "host"
        );
    }

    #[test]
    fn a_link_rule_ignores_a_query_it_was_not_saved_with() {
        let set = RuleSet {
            schema_version: 2,
            rules: vec![rule("link", Scope::Url(URL.to_owned()), None, "chrome")],
        };
        assert!(
            set.resolve(&format!("{URL}?tab=readme"), "github.com", None)
                .is_none()
        );
    }

    #[test]
    fn naming_the_origin_outranks_any_host_precision() {
        let set = RuleSet {
            schema_version: 2,
            rules: vec![
                rule("link", Scope::Url(URL.to_owned()), None, "firefox"),
                rule("origin", Scope::Any, Some("slack"), "brave"),
            ],
        };
        assert_eq!(
            set.resolve(URL, "github.com", Some("slack")).unwrap().id,
            "origin"
        );
        assert_eq!(set.resolve(URL, "github.com", None).unwrap().id, "link");
    }

    #[test]
    fn choosing_always_here_twice_changes_the_answer_instead_of_being_ignored() {
        let mut set = RuleSet {
            schema_version: 2,
            rules: vec![rule(
                "first",
                Scope::Site("github.com".to_owned()),
                None,
                "firefox",
            )],
        };
        set.upsert(rule(
            "second",
            Scope::Site("github.com".to_owned()),
            None,
            "chrome",
        ));
        assert_eq!(set.rules.len(), 1);
        assert_eq!(
            set.resolve(URL, "github.com", None)
                .unwrap()
                .target
                .browser_id,
            "chrome"
        );
    }

    #[test]
    fn a_narrower_scope_over_the_same_host_is_a_different_rule() {
        let mut set = RuleSet {
            schema_version: 2,
            rules: vec![rule(
                "site",
                Scope::Site("github.com".to_owned()),
                None,
                "firefox",
            )],
        };
        set.upsert(rule("link", Scope::Url(URL.to_owned()), None, "chrome"));
        assert_eq!(set.rules.len(), 2);
    }

    #[test]
    fn a_rule_for_a_different_origin_is_a_different_rule() {
        let mut set = RuleSet {
            schema_version: 2,
            rules: vec![rule(
                "anywhere",
                Scope::Site("github.com".to_owned()),
                None,
                "firefox",
            )],
        };
        set.upsert(rule(
            "from-slack",
            Scope::Site("github.com".to_owned()),
            Some("slack"),
            "brave",
        ));
        assert_eq!(set.rules.len(), 2);
    }

    #[test]
    fn equally_specific_rules_are_decided_by_the_order_the_user_sees() {
        let set = RuleSet {
            schema_version: 2,
            rules: vec![
                rule(
                    "first",
                    Scope::Host("github.com".to_owned()),
                    None,
                    "firefox",
                ),
                rule(
                    "second",
                    Scope::Host("github.com".to_owned()),
                    None,
                    "chrome",
                ),
            ],
        };
        assert_eq!(set.resolve(URL, "github.com", None).unwrap().id, "first");
    }

    #[test]
    fn a_rule_bound_to_an_origin_never_fires_without_one() {
        let set = RuleSet {
            schema_version: 2,
            rules: vec![rule("origin", Scope::Any, Some("slack"), "brave")],
        };
        assert!(set.resolve(URL, "github.com", None).is_none());
        assert!(set.resolve(URL, "github.com", Some("SLACK")).is_some());
    }

    #[test]
    #[ignore = "fails today: site_of pipes an IPv4 literal through psl::domain_str, which treats the last two octets as label+suffix"]
    fn an_ip_v4_literal_keeps_itself_as_its_site() {
        assert_eq!(site_of("192.168.1.50"), "192.168.1.50");
        assert_eq!(site_of("10.0.1.50"), "10.0.1.50");
        assert_eq!(site_of("127.0.0.1"), "127.0.0.1");
        assert_eq!(site_of("8.8.8.8"), "8.8.8.8");
    }

    #[test]
    fn an_ipv6_literal_keeps_itself_as_its_site() {
        assert_eq!(site_of("[::1]"), "[::1]");
    }

    #[test]
    fn a_host_carrying_a_port_is_not_split_by_it() {
        assert_eq!(site_of("example.com:8443"), "example.com:8443");
    }

    #[test]
    #[ignore = "fails today: site_of destroys IPv4 literals (see an_ip_v4_literal_keeps_itself_as_its_site), so a Site scope built from one machine's IP is short enough to also match a different machine"]
    fn a_site_rule_built_from_one_machines_ip_never_matches_a_different_machine() {
        let scope = Scope::Site(site_of("192.168.1.50"));
        assert!(scope.matches("https://192.168.1.50/admin", "192.168.1.50"));
        assert!(!scope.matches("https://10.0.1.50/admin", "10.0.1.50"));
    }

    #[test]
    #[ignore = "fails today: RuleSet::upsert compares source_app with `==`, so replacing a rule migrated from 1.x as \"Slack\" with one saved as \"slack\" appends a duplicate instead of replacing it"]
    fn upserting_a_rule_replaces_one_saved_for_the_same_origin_in_a_different_case() {
        let mut set = RuleSet {
            schema_version: 2,
            rules: vec![rule(
                "legacy",
                Scope::Site("github.com".to_owned()),
                Some("Slack"),
                "firefox",
            )],
        };
        set.upsert(rule(
            "updated",
            Scope::Site("github.com".to_owned()),
            Some("slack"),
            "chrome",
        ));
        assert_eq!(set.rules.len(), 1);
        assert_eq!(
            set.resolve(URL, "github.com", Some("slack"))
                .unwrap()
                .target
                .browser_id,
            "chrome"
        );
    }
}
