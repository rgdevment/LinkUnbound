use std::fs;
use std::path::{Path, PathBuf};

use crate::config::{
    BrowserConfig, ConfigError, read_browsers, read_rules, write_browsers, write_rules,
};
use crate::rule::RuleSet;

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("could not read {path}: {source}")]
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("could not save {path}: {source}")]
    Write {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("{path}: {source}")]
    Content { path: PathBuf, source: ConfigError },
}

/// Writes through a sibling temporary file and renames over the target. A crash
/// or a full disk mid-write then leaves the previous rules intact rather than a
/// half-written file that reads as no rules at all.
fn save_atomically(path: &Path, body: &str) -> Result<(), StoreError> {
    let fail = |source| StoreError::Write {
        path: path.to_path_buf(),
        source,
    };

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(fail)?;
    }
    let staging = path.with_extension("tmp");
    fs::write(&staging, body).map_err(fail)?;
    fs::rename(&staging, path).map_err(fail)
}

fn load<T>(path: &Path, parse: impl Fn(&str) -> Result<T, ConfigError>) -> Result<T, StoreError>
where
    T: Default,
{
    let raw = match fs::read_to_string(path) {
        Ok(raw) => raw,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(T::default()),
        Err(source) => {
            return Err(StoreError::Read {
                path: path.to_path_buf(),
                source,
            });
        }
    };
    parse(&raw).map_err(|source| StoreError::Content {
        path: path.to_path_buf(),
        source,
    })
}

/// Where the 1.x line kept its files, so an upgrade finds them where they are.
#[derive(Debug, Clone)]
pub struct Store {
    dir: PathBuf,
}

impl Store {
    #[must_use]
    pub fn at(dir: impl Into<PathBuf>) -> Self {
        Self { dir: dir.into() }
    }

    fn rules_path(&self) -> PathBuf {
        self.dir.join("rules.json")
    }

    fn prefs_path(&self) -> PathBuf {
        self.dir.join("preferences.json")
    }

    fn browsers_path(&self) -> PathBuf {
        self.dir.join("browsers.json")
    }

    /// A missing file is an empty set, not an error: that is a first run.
    pub fn rules(&self) -> Result<RuleSet, StoreError> {
        load(&self.rules_path(), read_rules)
    }

    pub fn browsers(&self) -> Result<BrowserConfig, StoreError> {
        load(&self.browsers_path(), read_browsers)
    }

    pub fn save_rules(&self, rules: &RuleSet) -> Result<(), StoreError> {
        let body = write_rules(rules).map_err(|source| StoreError::Content {
            path: self.rules_path(),
            source,
        })?;
        save_atomically(&self.rules_path(), &body)
    }

    /// Falls back to what 1.x left in its own `theme` file, so an upgrade keeps
    /// the appearance the user had chosen.
    pub fn prefs(&self) -> crate::Preferences {
        if let Ok(raw) = fs::read_to_string(self.prefs_path())
            && let Ok(found) = serde_json::from_str::<crate::Preferences>(&raw)
        {
            return found;
        }
        let legacy = fs::read_to_string(self.dir.join("theme")).unwrap_or_default();
        crate::Preferences::default().with_legacy_theme(&legacy)
    }

    pub fn save_prefs(&self, prefs: &crate::Preferences) -> Result<(), StoreError> {
        let body = serde_json::to_string_pretty(prefs).map_err(|source| StoreError::Content {
            path: self.prefs_path(),
            source: crate::ConfigError::Malformed(source),
        })?;
        save_atomically(&self.prefs_path(), &body)
    }

    pub fn save_browsers(&self, config: &BrowserConfig) -> Result<(), StoreError> {
        let body = write_browsers(config).map_err(|source| StoreError::Content {
            path: self.browsers_path(),
            source,
        })?;
        save_atomically(&self.browsers_path(), &body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::SCHEMA_VERSION;
    use crate::rule::{Rule, Scope, Target};

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("linkunbound-store-{name}"));
        let _ = fs::remove_dir_all(&dir);
        dir
    }

    fn a_rule() -> Rule {
        Rule {
            id: "r1".to_owned(),
            scope: Scope::Site("github.com".to_owned()),
            source_app: None,
            target: Target {
                browser_id: "firefox".to_owned(),
                profile_id: None,
            },
            private: false,
        }
    }

    #[test]
    fn a_first_run_finds_no_rules_rather_than_an_error() {
        let store = Store::at(scratch("empty"));
        assert!(store.rules().unwrap().rules.is_empty());
        assert!(store.browsers().unwrap().browsers.is_empty());
    }

    #[test]
    fn what_is_saved_is_what_comes_back() {
        let store = Store::at(scratch("roundtrip"));
        let mut set = RuleSet {
            schema_version: SCHEMA_VERSION,
            rules: Vec::new(),
        };
        set.upsert(a_rule());
        store.save_rules(&set).unwrap();

        let read = store.rules().unwrap();
        assert_eq!(read.rules.len(), 1);
        assert_eq!(read.schema_version, SCHEMA_VERSION);
        assert!(
            read.resolve("https://gist.github.com/x", "gist.github.com", None)
                .is_some()
        );
    }

    #[test]
    fn saving_leaves_no_temporary_file_behind() {
        let dir = scratch("staging");
        let store = Store::at(&dir);
        store
            .save_rules(&RuleSet {
                schema_version: SCHEMA_VERSION,
                rules: vec![a_rule()],
            })
            .unwrap();
        let leftovers: Vec<_> = fs::read_dir(&dir)
            .unwrap()
            .flatten()
            .filter(|e| e.path().extension().is_some_and(|x| x == "tmp"))
            .collect();
        assert!(leftovers.is_empty());
    }

    #[test]
    fn a_corrupt_file_names_itself_instead_of_reading_as_no_rules() {
        let dir = scratch("corrupt");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("rules.json"), "{ not json").unwrap();
        let err = Store::at(&dir).rules().unwrap_err();
        assert!(matches!(err, StoreError::Content { .. }));
        assert!(err.to_string().contains("rules.json"));
    }
}
