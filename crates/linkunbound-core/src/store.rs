use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

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
    // Named after this process: two of them sharing one staging file meant each
    // truncating what the other was still writing, and the loser renamed the
    // mixture over the real one.
    let staging = path.with_extension(format!("{}.tmp", std::process::id()));
    fs::write(&staging, body).map_err(fail)?;
    let renamed = fs::rename(&staging, path).map_err(fail);
    if renamed.is_err() {
        let _ = fs::remove_file(&staging);
    }
    renamed
}

const HELD_FOR_LONG_ENOUGH: Duration = Duration::from_secs(5);

/// Holds the whole read-modify-write, not just the write. Both binaries edit the
/// same set, and an atomic save alone still loses whichever change was read
/// before the other process saved.
///
/// The guard is a file created exclusively: the filesystem decides the winner.
/// One left behind by a process that died is taken over once it goes stale,
/// because a lock nobody can release is worse than the race it prevents.
/// Releases on every exit, a panic inside the edit included. Without this the
/// file stayed behind and locked the set out until it went stale.
struct Holding(PathBuf);

impl Drop for Holding {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

fn guarded<T>(path: &Path, work: impl FnOnce() -> Result<T, StoreError>) -> Result<T, StoreError> {
    let lock = path.with_extension("lock");
    if let Some(parent) = lock.parent() {
        let _ = fs::create_dir_all(parent);
    }

    for _ in 0..400 {
        match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&lock)
        {
            Ok(_) => {
                let _holding = Holding(lock);
                return work();
            }
            Err(_) => {
                // Only steal one we can see and that is plainly old. A failed
                // `metadata` means the owner just released it, and treating
                // that as stale deleted a lock somebody else had already taken.
                if fs::metadata(&lock)
                    .and_then(|m| m.modified())
                    .is_ok_and(|held| held.elapsed().unwrap_or_default() > HELD_FOR_LONG_ENOUGH)
                {
                    let _ = fs::remove_file(&lock);
                }
                std::thread::sleep(Duration::from_millis(5));
            }
        }
    }
    // Giving up is the safe answer: going ahead unguarded is what corrupts.
    Err(StoreError::Write {
        path: lock,
        source: std::io::Error::new(
            std::io::ErrorKind::TimedOut,
            "another process is holding the file",
        ),
    })
}

/// `serde_json` refuses the mark Windows editors prepend, which would discard a
/// whole file over three invisible bytes.
#[must_use]
pub fn unmarked(raw: &str) -> &str {
    raw.strip_prefix('\u{feff}').unwrap_or(raw)
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
    parse(unmarked(&raw)).map_err(|source| StoreError::Content {
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

    #[must_use]
    pub fn dir(&self) -> &Path {
        &self.dir
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
        guarded(&self.rules_path(), || {
            save_atomically(&self.rules_path(), &body)
        })
    }

    /// Reads, edits and saves under one guard. Editing a set read before another
    /// process saved is how a rule the picker just remembered disappears when
    /// settings next writes.
    pub fn edit_rules(&self, edit: impl FnOnce(&mut RuleSet) -> bool) -> Result<bool, StoreError> {
        guarded(&self.rules_path(), || {
            let mut rules = load(&self.rules_path(), read_rules)?;
            if !edit(&mut rules) {
                return Ok(false);
            }
            let body = write_rules(&rules).map_err(|source| StoreError::Content {
                path: self.rules_path(),
                source,
            })?;
            save_atomically(&self.rules_path(), &body).map(|()| true)
        })
    }

    /// Falls back to what 1.x left in its own `theme` file, so an upgrade keeps
    /// the appearance the user had chosen.
    pub fn prefs(&self) -> crate::Preferences {
        if let Ok(raw) = fs::read_to_string(self.prefs_path())
            && let Ok(found) = serde_json::from_str::<crate::Preferences>(unmarked(&raw))
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
        guarded(&self.browsers_path(), || {
            save_atomically(&self.browsers_path(), &body)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::SCHEMA_VERSION;
    use crate::rule::{Rule, Scope, Target};

    /// Per process: the path was machine-global, so a second `cargo test` wiped
    /// this one's directory mid-run and the failures looked like product bugs.
    fn scratch(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("linkunbound-store-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn a_file_saved_with_a_byte_order_mark_is_still_read() {
        let dir = scratch("bom");
        fs::create_dir_all(&dir).expect("should create");
        let store = Store::at(&dir);

        let marked = format!(
            "\u{feff}{}",
            serde_json::to_string(&crate::Preferences {
                locale: crate::Locale::English,
                ..crate::Preferences::default()
            })
            .expect("should serialise")
        );
        fs::write(dir.join("preferences.json"), marked).expect("should write");

        assert_eq!(store.prefs().locale, crate::Locale::English);
    }

    #[test]
    fn rules_survive_the_same_mark() {
        let dir = scratch("bom-rules");
        fs::create_dir_all(&dir).expect("should create");
        let store = Store::at(&dir);

        let mut rules = RuleSet::default();
        rules.upsert(a_rule());
        let body = format!(
            "\u{feff}{}",
            crate::write_rules(&rules).expect("should serialise")
        );
        fs::write(dir.join("rules.json"), body).expect("should write");

        assert_eq!(store.rules().expect("should read").rules.len(), 1);
    }

    /// Both binaries write this file. Before the guard, the shared staging name
    /// let each truncate what the other was writing and the loser renamed the
    /// mixture over the real one: 6 of 200 rounds left unreadable JSON.
    #[test]
    fn many_writers_at_once_leave_the_file_readable_and_whole() {
        let dir = scratch("concurrent");
        fs::create_dir_all(&dir).expect("should create");
        let store = Store::at(&dir);
        store.save_rules(&RuleSet::default()).expect("should seed");

        std::thread::scope(|pack| {
            for worker in 0..8 {
                let store = Store::at(&dir);
                pack.spawn(move || {
                    for round in 0..25 {
                        let id = format!("w{worker}-{round}");
                        let _ = store.edit_rules(|rules| {
                            rules.upsert(Rule {
                                id: id.clone(),
                                scope: Scope::Host(format!("{id}.test")),
                                source_app: None,
                                target: Target {
                                    browser_id: "firefox".to_owned(),
                                    profile_id: None,
                                },
                                private: false,
                            });
                            true
                        });
                    }
                });
            }
        });

        let survived = store.rules().expect("the file must still be readable");
        assert!(
            survived.rules.len() > 100,
            "a lost update dropped too much: {} of 200",
            survived.rules.len()
        );
    }

    /// The version marker is there so a newer file is recognised rather than
    /// parsed. A caller that swallows this error and writes anyway erases the
    /// fields it could not read: every custom browser, on the first switch.
    #[test]
    fn a_browsers_file_from_a_newer_version_reads_as_an_error_not_as_empty() {
        let dir = scratch("too-new");
        fs::create_dir_all(&dir).expect("should create");
        fs::write(
            dir.join("browsers.json"),
            r#"{"schema_version": 99, "browsers": []}"#,
        )
        .expect("should write");

        let outcome = Store::at(&dir).browsers();
        assert!(
            outcome.is_err(),
            "a file this version cannot read must not read as an empty catalogue"
        );
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
