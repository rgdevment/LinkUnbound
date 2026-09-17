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
    #[serde(default, rename = "isCustom")]
    is_custom: bool,
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
    // Entry by entry, as 1.x read it: one line somebody edited by hand cost every other rule
    // until settings reset the file.
    let legacy: Vec<serde_json::Value> = serde_json::from_value(value)?;
    Ok(RuleSet {
        schema_version: SCHEMA_VERSION,
        rules: legacy
            .into_iter()
            .enumerate()
            .filter_map(|(i, entry)| {
                serde_json::from_value::<LegacyRule>(entry)
                    .ok()
                    .map(|r| migrate_rule(i, r))
            })
            .collect(),
    })
}

/// A bare array, or an object that never learned to say its version: what the 1.x line wrote.
pub fn is_the_old_shape(raw: &str) -> bool {
    serde_json::from_str::<serde_json::Value>(raw)
        .is_ok_and(|value| value.is_array() || (value.is_object() && version_of(&value) == 0))
}

fn version_of(value: &serde_json::Value) -> u32 {
    value
        .get("schema_version")
        .and_then(serde_json::Value::as_u64)
        .and_then(|v| u32::try_from(v).ok())
        .unwrap_or(0)
}

/// The file is disarmed on the way in, at the one place it is read: this app is
/// what the shell runs for a link, so whatever the file says is what gets
/// launched. Anything past this point can be trusted to launch.
pub fn read_browsers(raw: &str) -> Result<BrowserConfig, ConfigError> {
    let value: serde_json::Value = serde_json::from_str(raw)?;
    let found = version_of(&value);
    if found > SCHEMA_VERSION {
        return Err(ConfigError::TooNew { found });
    }
    if found == SCHEMA_VERSION {
        let mut config: BrowserConfig = serde_json::from_value(value)?;
        for browser in &mut config.browsers {
            crate::hostile::disarm(browser);
        }
        return Ok(config);
    }

    let legacy: LegacyBrowserConfig = serde_json::from_value(value)?;
    let mut config = BrowserConfig {
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
                icon_path: None,
                custom: b.is_custom,
                hidden: false,
                extra_args: b.extra_args,
            })
            .collect(),
    };
    for browser in &mut config.browsers {
        crate::hostile::disarm(browser);
    }
    Ok(config)
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

    /// What is written has to be what can be read back, and it has to carry the schema: a file
    /// that says nothing about its version is one a later reader has to guess at.
    #[test]
    fn what_is_written_reads_back_as_the_same_browsers() {
        let mut config = BrowserConfig::default();
        config.browsers.push(crate::Browser {
            id: "chrome".to_owned(),
            name: "Google Chrome".to_owned(),
            exe: r"C:\Program Files\chrome.exe".to_owned(),
            profiles: Vec::new(),
            extra_args: vec!["--new-window".to_owned()],
            private_flag: Some("--incognito".to_owned()),
            icon_path: None,
            custom: true,
            hidden: true,
        });

        let body = write_browsers(&config).expect("written");
        assert!(
            body.contains("\"schema_version\""),
            "the file has to say what it is: {body}"
        );

        let read = read_browsers(&body).expect("read back");
        assert_eq!(read.browsers.len(), 1);
        assert_eq!(read.browsers[0].name, "Google Chrome");
        assert_eq!(read.browsers[0].extra_args, vec!["--new-window".to_owned()]);
        assert!(read.browsers[0].hidden, "hidden survives the trip");
        assert!(read.browsers[0].custom);
    }
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
    fn one_broken_1_x_entry_does_not_discard_the_rest() {
        let raw = r#"[
            {"domain": "a.test", "browserId": "ff", "private": false},
            {"domain": 12, "browserId": null},
            "not even an object",
            {"domain": "b.test", "browserId": "ch", "private": true}
        ]"#;
        let rules = read_rules(raw).unwrap();
        assert_eq!(rules.rules.len(), 2);
        assert_eq!(
            rules.rules[1].id, "migrated-3",
            "the index is the file's, not the count's"
        );
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

    /// The 1.x file is the one a person actually brings, and it was never disarmed: a browser
    /// somebody planted in it launched whatever it named until settings happened to save.
    #[test]
    fn a_1_x_browser_list_is_disarmed_on_the_way_in_like_any_other() {
        let raw = r#"{"schema_version": "1.0", "browsers": [
            {"id": "x", "name": "X", "executablePath": "\\\\evil\\share\\x.exe",
             "iconPath": "", "extraArgs": ["--gpu-launcher=calc.exe", "--new-window"],
             "privateArgs": null, "isCustom": true}
        ]}"#;
        let config = read_browsers(raw).unwrap();
        assert_eq!(config.browsers[0].exe, "", "a remote executable is dropped");
        assert_eq!(
            config.browsers[0].extra_args,
            vec!["--new-window".to_owned()]
        );
    }

    #[test]
    fn the_old_shape_is_told_apart_from_what_this_writes() {
        assert!(is_the_old_shape("[]"));
        assert!(is_the_old_shape(
            r#"[{"domain": "a.test", "browserId": "ff"}]"#
        ));
        assert!(is_the_old_shape(
            r#"{"schema_version": "1.0", "browsers": []}"#
        ));
        assert!(is_the_old_shape(r#"{"browsers": []}"#));
        assert!(!is_the_old_shape(r#"{"schema_version": 2, "rules": []}"#));
        assert!(!is_the_old_shape("{ not json"));
    }

    #[test]
    fn a_broken_file_reports_itself_instead_of_losing_the_rules() {
        assert!(read_rules("{ not json").is_err());
        assert!(read_browsers("").is_err());
    }

    #[test]
    fn a_rules_file_written_by_a_newer_schema_is_refused_instead_of_dropping_fields() {
        let err = read_rules(r#"{"schema_version": 99, "rules": []}"#).unwrap_err();
        assert!(matches!(err, ConfigError::TooNew { found: 99 }));
    }

    #[test]
    fn a_browsers_file_written_by_a_newer_schema_is_refused_instead_of_dropping_fields() {
        let err = read_browsers(r#"{"schema_version": 99, "browsers": []}"#).unwrap_err();
        assert!(matches!(err, ConfigError::TooNew { found: 99 }));
    }
}
