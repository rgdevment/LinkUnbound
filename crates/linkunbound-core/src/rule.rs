use std::cmp::Reverse;

use serde::{Deserialize, Serialize};

/// `psl` would read the last two octets as label plus suffix: `192.168.1.50`
/// becomes `1.50`, which then matches other machines.
#[must_use]
pub fn is_address(host: &str) -> bool {
    host.starts_with('[') || host.parse::<std::net::IpAddr>().is_ok()
}

/// Registrable domain; an unknown suffix keeps the host, which matches less rather than more.
#[must_use]
pub fn site_of(host: &str) -> String {
    if is_address(host) {
        return host.to_ascii_lowercase();
    }
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
            Self::Site(d) => host == d || host.strip_suffix(d).is_some_and(|p| p.ends_with('.')),
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

fn same_origin(a: Option<&str>, b: Option<&str>) -> bool {
    match (a, b) {
        (None, None) => true,
        (Some(a), Some(b)) => a.eq_ignore_ascii_case(b),
        _ => false,
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
    /// What `upsert` already treats as the same rule, so the id cannot name two.
    #[must_use]
    pub fn identity(&self) -> String {
        let scope = match &self.scope {
            Scope::Any => "any".to_owned(),
            Scope::Url(u) => format!("url:{u}"),
            Scope::Host(h) => format!("host:{h}"),
            Scope::Site(d) => format!("site:{d}"),
        };
        match &self.source_app {
            Some(app) => format!("{scope}@{}", app.to_ascii_lowercase()),
            None => scope,
        }
    }

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
    pub fn upsert(&mut self, mut rule: Rule) {
        rule.id = rule.identity();
        match self.rules.iter_mut().find(|r| {
            r.scope == rule.scope
                && same_origin(r.source_app.as_deref(), rule.source_app.as_deref())
        }) {
            Some(existing) => *existing = rule,
            None => self.rules.push(rule),
        }
    }

    /// The order the user sees is the order that decides, so moving a rule is
    /// how a tie gets broken.
    pub fn remove(&mut self, id: &str) -> bool {
        let before = self.rules.len();
        self.rules.retain(|r| r.id != id);
        before != self.rules.len()
    }

    pub fn reorder(&mut self, ids: &[String]) {
        let mut moved: Vec<Rule> = Vec::with_capacity(self.rules.len());
        for id in ids {
            if let Some(at) = self.rules.iter().position(|r| &r.id == id) {
                moved.push(self.rules.remove(at));
            }
        }
        moved.append(&mut self.rules);
        self.rules = moved;
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
    fn two_rules_that_upsert_would_merge_cannot_hold_different_ids() {
        let a = rule(
            "x",
            Scope::Site("github.com".to_owned()),
            Some("Slack"),
            "chrome",
        );
        let b = rule(
            "y",
            Scope::Site("github.com".to_owned()),
            Some("slack"),
            "firefox",
        );
        assert_eq!(a.identity(), b.identity());

        let c = rule("z", Scope::Host("github.com".to_owned()), None, "chrome");
        assert_ne!(a.identity(), c.identity());
    }

    #[test]
    fn saving_a_rule_names_it_after_what_it_covers() {
        let mut set = RuleSet::default();
        set.upsert(rule(
            "ignored",
            Scope::Site("github.com".to_owned()),
            None,
            "chrome",
        ));
        assert_eq!(set.rules[0].id, "site:github.com");
    }

    #[test]
    fn removing_a_rule_takes_only_that_one() {
        let mut set = RuleSet::default();
        set.upsert(rule(
            "a",
            Scope::Site("github.com".to_owned()),
            None,
            "chrome",
        ));
        set.upsert(rule("b", Scope::Url(URL.to_owned()), None, "firefox"));
        assert!(set.remove("site:github.com"));
        assert!(!set.remove("site:github.com"));
        assert_eq!(set.rules.len(), 1);
    }

    /// Between equally specific rules the first wins, so reordering is the only
    /// way the user can change which one answers.
    #[test]
    fn reordering_decides_which_of_two_equal_rules_answers() {
        let mut set = RuleSet::default();
        set.upsert(rule(
            "a",
            Scope::Host("github.com".to_owned()),
            None,
            "firefox",
        ));
        set.upsert(rule(
            "b",
            Scope::Host("github.com".to_owned()),
            Some("slack"),
            "chrome",
        ));
        let ids: Vec<String> = set.rules.iter().rev().map(|r| r.id.clone()).collect();
        set.reorder(&ids);
        assert_eq!(set.rules[0].target.browser_id, "chrome");
    }

    /// An id the caller no longer has must not drop the rule it names.
    #[test]
    fn reordering_with_a_stale_id_keeps_every_rule() {
        let mut set = RuleSet::default();
        set.upsert(rule(
            "a",
            Scope::Site("github.com".to_owned()),
            None,
            "chrome",
        ));
        set.upsert(rule("b", Scope::Url(URL.to_owned()), None, "firefox"));
        set.reorder(&["site:github.com".to_owned(), "gone".to_owned()]);
        assert_eq!(set.rules.len(), 2);
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
    fn a_site_rule_built_from_one_machines_ip_never_matches_a_different_machine() {
        let scope = Scope::Site(site_of("192.168.1.50"));
        assert!(scope.matches("https://192.168.1.50/admin", "192.168.1.50"));
        assert!(!scope.matches("https://10.0.1.50/admin", "10.0.1.50"));
    }

    #[test]
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
