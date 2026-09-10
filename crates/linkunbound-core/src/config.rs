use serde::{Deserialize, Serialize};

use crate::browser::Browser;
use crate::private::private_flag_for;
use crate::rule::{Rule, RuleSet, Scope, Target};

pub const SCHEMA_VERSION: u32 = 2;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BrowserConfig {
    #[serde(default)]
    pub schema_version: u32,
    #[serde(default)]
    pub browsers: Vec<Browser>,
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("the file is not valid JSON: {0}")]
    Malformed(#[from] serde_json::Error),
    /// Reading it would drop the fields this version does not know, and the next
    /// save would delete them from disk.
    #[error("written by a newer version ({found} > {SCHEMA_VERSION})")]
    TooNew { found: u32 },
}

#[derive(Debug, Deserialize)]
struct LegacyRule {
    domain: String,
    #[serde(rename = "browserId")]
    browser_id: String,
    #[serde(rename = "sourceApp")]
    source_app: Option<String>,
    #[serde(default)]
    private: bool,
}

#[derive(Debug, Deserialize)]
struct LegacyBrowser {
    id: String,
    name: String,
    #[serde(rename = "executablePath")]
    executable_path: String,
    #[serde(default, rename = "extraArgs")]
    extra_args: Vec<String>,
    #[serde(default, rename = "privateArgs")]
    private_args: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct LegacyBrowserConfig {
    #[serde(default)]
    browsers: Vec<LegacyBrowser>,
}

/// `Site` keeps the reach 1.x had: `Host` would narrow every migrated rule, and
/// reducing the domain to its registrable form would widen one saved for a subdomain.
fn migrate_rule(index: usize, legacy: LegacyRule) -> Rule {
    let scope = if legacy.domain == "*" {
        Scope::Any
    } else {
        Scope::Site(legacy.domain.to_ascii_lowercase())
    };
    Rule {
        id: format!("migrated-{index}"),
        scope,
        source_app: legacy.source_app.filter(|s| !s.is_empty()),
        target: Target {
            browser_id: legacy.browser_id,
            profile_id: None,
        },
        private: legacy.private,
    }
}

/// Reads either schema. A 1.x rules file is a bare array; a 2.x one is an object
/// carrying its version, which the 1.x file never had and could not gain.
pub fn read_rules(raw: &str) -> Result<RuleSet, ConfigError> {
    let value: serde_json::Value = serde_json::from_str(raw)?;
    if value.is_object() {
        let found = version_of(&value);
        if found > SCHEMA_VERSION {
            return Err(ConfigError::TooNew { found });
        }
        return Ok(serde_json::from_value(value)?);
    }
    let legacy: Vec<LegacyRule> = serde_json::from_value(value)?;
    Ok(RuleSet {
        schema_version: SCHEMA_VERSION,
        rules: legacy
            .into_iter()
            .enumerate()
            .map(|(i, r)| migrate_rule(i, r))
            .collect(),
    })
}

fn version_of(value: &serde_json::Value) -> u32 {
    value
        .get("schema_version")
        .and_then(serde_json::Value::as_u64)
        .and_then(|v| u32::try_from(v).ok())
        .unwrap_or(0)
}

pub fn read_browsers(raw: &str) -> Result<BrowserConfig, ConfigError> {
    let value: serde_json::Value = serde_json::from_str(raw)?;
    let found = version_of(&value);
    if found > SCHEMA_VERSION {
        return Err(ConfigError::TooNew { found });
    }
    if found == SCHEMA_VERSION {
        return Ok(serde_json::from_value(value)?);
    }

    let legacy: LegacyBrowserConfig = serde_json::from_value(value)?;
    Ok(BrowserConfig {
        schema_version: SCHEMA_VERSION,
        browsers: legacy
            .browsers
            .into_iter()
            .map(|b| Browser {
                id: b.id,
                name: b.name,
                profiles: Vec::new(),
                // 1.x stored null to mean "derive it from the executable",
                // not "this browser has none".
                private_flag: b.private_args.map_or_else(
                    || private_flag_for(&b.executable_path).map(str::to_owned),
                    |a| a.into_iter().next(),
                ),
                exe: b.executable_path,
                extra_args: b.extra_args,
            })
            .collect(),
    })
}

/// The writer stamps the version; trusting the caller means a set built with
/// `default()` lands on disk claiming version zero.
pub fn write_rules(rules: &RuleSet) -> Result<String, ConfigError> {
    let sealed = RuleSet {
        schema_version: SCHEMA_VERSION,
        rules: rules.rules.clone(),
    };
    Ok(serde_json::to_string_pretty(&sealed)?)
}

pub fn write_browsers(config: &BrowserConfig) -> Result<String, ConfigError> {
    let sealed = BrowserConfig {
        schema_version: SCHEMA_VERSION,
        browsers: config.browsers.clone(),
    };
    Ok(serde_json::to_string_pretty(&sealed)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    const LEGACY_RULES: &str = r#"[
        {"domain": "github.com", "browserId": "firefox", "private": false},
        {"domain": "*", "browserId": "brave", "sourceApp": "slack", "private": true},
        {"domain": "GitLab.test", "browserId": "chrome", "sourceApp": null}
    ]"#;

    #[test]
    fn a_1_x_rules_file_is_read_without_asking_the_user_anything() {
        let set = read_rules(LEGACY_RULES).unwrap();
        assert_eq!(set.schema_version, SCHEMA_VERSION);
        assert_eq!(set.rules.len(), 3);
        assert_eq!(set.rules[0].scope, Scope::Site("github.com".to_owned()));
        assert_eq!(set.rules[1].scope, Scope::Any);
        assert_eq!(set.rules[1].source_app.as_deref(), Some("slack"));
        assert!(set.rules[1].private);
        assert_eq!(set.rules[2].scope, Scope::Site("gitlab.test".to_owned()));
        assert!(set.rules[2].source_app.is_none());
    }

    #[test]
    fn a_migrated_rule_still_covers_the_subdomains_1_x_covered() {
        let set = read_rules(LEGACY_RULES).unwrap();
        let any = "https://example.test/";
        assert!(set.resolve(any, "github.com", None).is_some());
        assert!(set.resolve(any, "gist.github.com", None).is_some());
        assert!(set.resolve(any, "docs.gitlab.test", None).is_some());
        assert!(set.resolve(any, "notgithub.com", None).is_none());
    }

    #[test]
    fn a_2_x_file_round_trips_unchanged() {
        let set = read_rules(LEGACY_RULES).unwrap();
        let written = write_rules(&set).unwrap();
        let again = read_rules(&written).unwrap();
        assert_eq!(again.rules.len(), set.rules.len());
        assert_eq!(again.rules[1].scope, Scope::Any);
        assert_eq!(again.schema_version, SCHEMA_VERSION);
    }

    #[test]
    fn a_1_x_browser_list_keeps_its_private_switch() {
        let raw = r#"{"schema_version": "1.0", "browsers": [
            {"id": "ff", "name": "Firefox", "executablePath": "C:/ff.exe",
             "iconPath": "", "extraArgs": [], "privateArgs": ["-private-window"]}
        ]}"#;
        let config = read_browsers(raw).unwrap();
        assert_eq!(config.schema_version, SCHEMA_VERSION);
        assert_eq!(
            config.browsers[0].private_flag.as_deref(),
            Some("-private-window")
        );
        assert!(config.browsers[0].profiles.is_empty());
    }

    #[test]
    fn a_browser_with_no_private_switch_is_marked_as_lacking_one() {
        let raw = r#"{"browsers": [
            {"id": "safari", "name": "Safari", "executablePath": "/Safari.app", "extraArgs": []}
        ]}"#;
        let config = read_browsers(raw).unwrap();
        assert!(!config.browsers[0].supports_private());
    }

    #[test]
    fn a_broken_file_reports_itself_instead_of_losing_the_rules() {
        assert!(read_rules("{ not json").is_err());
        assert!(read_browsers("").is_err());
    }
}
