use std::cmp::Reverse;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum HostPattern {
    Any,
    Exact(String),
    Suffix(String),
}

impl HostPattern {
    pub fn matches(&self, host: &str) -> bool {
        match self {
            Self::Any => true,
            Self::Exact(d) => host == d,
            Self::Suffix(d) => host == d || host.ends_with(&format!(".{d}")),
        }
    }

    fn specificity(&self) -> u32 {
        match self {
            Self::Any => 0,
            Self::Suffix(d) => 1_000 + d.len() as u32,
            Self::Exact(d) => 100_000 + d.len() as u32,
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
    pub host: HostPattern,
    #[serde(default)]
    pub source_app: Option<String>,
    pub target: Target,
    #[serde(default)]
    pub private: bool,
}

impl Rule {
    fn specificity(&self) -> u32 {
        let origin = if self.source_app.is_some() {
            1_000_000
        } else {
            0
        };
        origin + self.host.specificity()
    }

    fn applies(&self, host: &str, source_app: Option<&str>) -> bool {
        if !self.host.matches(host) {
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
    /// Replaces the rule covering the same host and origin instead of appending.
    /// Without this, choosing "always here" a second time for the same site adds
    /// a rule that never wins and the app appears to ignore the request.
    pub fn upsert(&mut self, rule: Rule) {
        match self
            .rules
            .iter_mut()
            .find(|r| r.host == rule.host && r.source_app == rule.source_app)
        {
            Some(existing) => *existing = rule,
            None => self.rules.push(rule),
        }
    }

    /// `Reverse` on the index keeps the first of equally specific rules: the list
    /// the user ordered is the list that decides, and `max_by_key` would take the last.
    pub fn resolve(&self, host: &str, source_app: Option<&str>) -> Option<&Rule> {
        self.rules
            .iter()
            .enumerate()
            .filter(|(_, r)| r.applies(host, source_app))
            .max_by_key(|(i, r)| (r.specificity(), Reverse(*i)))
            .map(|(_, r)| r)
    }
}

#[cfg(test)]
mod tests {
    use super::{HostPattern, Rule, RuleSet, Target};

    fn rule(id: &str, host: HostPattern, source_app: Option<&str>, browser: &str) -> Rule {
        Rule {
            id: id.to_owned(),
            host,
            source_app: source_app.map(str::to_owned),
            target: Target {
                browser_id: browser.to_owned(),
                profile_id: None,
            },
            private: false,
        }
    }

    #[test]
    fn a_suffix_covers_the_domain_and_every_subdomain() {
        let p = HostPattern::Suffix("github.com".to_owned());
        assert!(p.matches("github.com"));
        assert!(p.matches("gist.github.com"));
        assert!(!p.matches("notgithub.com"));
        assert!(!p.matches("github.com.evil.test"));
    }

    #[test]
    fn an_exact_host_beats_the_suffix_that_also_covers_it() {
        let set = RuleSet {
            schema_version: 1,
            rules: vec![
                rule(
                    "wide",
                    HostPattern::Suffix("github.com".to_owned()),
                    None,
                    "firefox",
                ),
                rule(
                    "narrow",
                    HostPattern::Exact("gist.github.com".to_owned()),
                    None,
                    "chrome",
                ),
            ],
        };
        assert_eq!(set.resolve("gist.github.com", None).unwrap().id, "narrow");
        assert_eq!(set.resolve("github.com", None).unwrap().id, "wide");
    }

    #[test]
    fn naming_the_origin_outranks_any_host_precision() {
        let set = RuleSet {
            schema_version: 1,
            rules: vec![
                rule(
                    "host",
                    HostPattern::Exact("github.com".to_owned()),
                    None,
                    "firefox",
                ),
                rule("origin", HostPattern::Any, Some("slack"), "brave"),
            ],
        };
        assert_eq!(
            set.resolve("github.com", Some("slack")).unwrap().id,
            "origin"
        );
        assert_eq!(set.resolve("github.com", None).unwrap().id, "host");
    }

    #[test]
    fn choosing_always_here_twice_changes_the_answer_instead_of_being_ignored() {
        let mut set = RuleSet {
            schema_version: 2,
            rules: vec![rule(
                "first",
                HostPattern::Suffix("github.com".to_owned()),
                None,
                "firefox",
            )],
        };
        set.upsert(rule(
            "second",
            HostPattern::Suffix("github.com".to_owned()),
            None,
            "chrome",
        ));
        assert_eq!(set.rules.len(), 1);
        assert_eq!(
            set.resolve("github.com", None).unwrap().target.browser_id,
            "chrome"
        );
    }

    #[test]
    fn a_rule_for_a_different_origin_is_a_different_rule() {
        let mut set = RuleSet {
            schema_version: 2,
            rules: vec![rule(
                "anywhere",
                HostPattern::Suffix("github.com".to_owned()),
                None,
                "firefox",
            )],
        };
        set.upsert(rule(
            "from-slack",
            HostPattern::Suffix("github.com".to_owned()),
            Some("slack"),
            "brave",
        ));
        assert_eq!(set.rules.len(), 2);
    }

    #[test]
    fn equally_specific_rules_are_decided_by_the_order_the_user_sees() {
        let set = RuleSet {
            schema_version: 1,
            rules: vec![
                rule(
                    "first",
                    HostPattern::Exact("github.com".to_owned()),
                    None,
                    "firefox",
                ),
                rule(
                    "second",
                    HostPattern::Exact("github.com".to_owned()),
                    None,
                    "chrome",
                ),
            ],
        };
        assert_eq!(set.resolve("github.com", None).unwrap().id, "first");
    }

    #[test]
    fn a_rule_bound_to_an_origin_never_fires_without_one() {
        let set = RuleSet {
            schema_version: 1,
            rules: vec![rule("origin", HostPattern::Any, Some("slack"), "brave")],
        };
        assert!(set.resolve("github.com", None).is_none());
        assert!(set.resolve("github.com", Some("SLACK")).is_some());
    }
}
