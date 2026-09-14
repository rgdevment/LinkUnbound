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

fn abandoned(held: Duration) -> bool {
    held > HELD_FOR_LONG_ENOUGH
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
                    .is_ok_and(|held| abandoned(held.elapsed().unwrap_or_default()))
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
#[must_use]
pub fn data_dir() -> PathBuf {
    under(base())
}

fn under(base: Option<PathBuf>) -> PathBuf {
    base.unwrap_or_else(std::env::temp_dir).join("LinkUnbound")
}

#[cfg(windows)]
fn base() -> Option<PathBuf> {
    std::env::var_os("LOCALAPPDATA")
        .or_else(|| std::env::var_os("APPDATA"))
        .map(PathBuf::from)
}

#[cfg(target_os = "macos")]
fn base() -> Option<PathBuf> {
    std::env::var_os("HOME").map(|home| {
        PathBuf::from(home)
            .join("Library")
            .join("Application Support")
    })
}

#[cfg(not(any(windows, target_os = "macos")))]
fn base() -> Option<PathBuf> {
    None
}

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

    /// Falls back to the files 1.x left beside this one — one per setting, holding a word or the
    /// single character `1`. Reading only the theme, as this did, sent somebody who had chosen
    /// Spanish, hidden the tray and set a shortcut back to the defaults on all three.
    pub fn prefs(&self) -> crate::Preferences {
        if let Ok(raw) = fs::read_to_string(self.prefs_path())
            && let Ok(found) = serde_json::from_str::<crate::Preferences>(unmarked(&raw))
        {
            return found;
        }
        const LEGACY: [&str; 5] = [
            "theme",
            "locale",
            "hide_tray",
            "edge_warning_dismissed",
            "global_hotkey",
        ];
        LEGACY.iter().fold(
            crate::Preferences::default(),
            |said, named| match fs::read_to_string(self.dir.join(named)) {
                Ok(raw) => said.with_legacy(named, &raw),
                Err(_) => said,
            },
        )
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

    #[test]
    fn the_files_live_in_a_directory_of_their_own() {
        let home = under(Some(PathBuf::from("/somewhere")));
        assert_eq!(home, Path::new("/somewhere").join("LinkUnbound"));
    }

    #[test]
    fn a_system_that_names_no_home_still_yields_somewhere_absolute() {
        let nowhere = under(None);
        assert!(nowhere.is_absolute(), "{nowhere:?}");
        assert!(nowhere.ends_with("LinkUnbound"));
    }

    /// Stealing a lock is destructive: the other process is mid-write and loses what it was
    /// saving. On the mark it is still somebody's, and only past it is it plainly nobody's.
    #[test]
    fn a_lock_is_taken_over_only_once_it_is_plainly_abandoned() {
        assert!(!abandoned(Duration::ZERO), "just taken");
        assert!(
            !abandoned(HELD_FOR_LONG_ENOUGH),
            "on the mark it is still somebody's"
        );
        assert!(abandoned(HELD_FOR_LONG_ENOUGH + Duration::from_millis(1)));
        assert!(
            abandoned(Duration::from_secs(3600)),
            "an hour old is nobody's, or the next save waits for ever"
        );
    }

    /// Only a file that is not there is a first run. Anything else that stops the read is a
    /// fault, and answering it with the defaults reads on screen as rules that vanished — and
    /// the next save writes the empty set over what was still on disk.
    #[test]
    fn a_read_that_fails_for_any_other_reason_is_not_a_first_run() {
        let dir = std::env::temp_dir().join("lu-unreadable-rules");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("rules.json")).expect("a directory where a file goes");

        assert!(
            Store::at(&dir).rules().is_err(),
            "a path that cannot be read is not an empty set of rules"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Saving that answers "fine" without writing anything is the worst kind of failure: the
    /// screen shows the new value, the file still holds the old one, and the next launch
    /// silently undoes what the person did.
    #[test]
    fn what_was_saved_is_what_comes_back() {
        let dir = std::env::temp_dir().join("lu-save-roundtrip");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("a place to write");
        let store = Store::at(&dir);

        let prefs = crate::Preferences {
            hide_tray: true,
            locale: crate::Locale::English,
            ..Default::default()
        };
        store.save_prefs(&prefs).expect("saved");
        let read = Store::at(&dir).prefs();
        assert!(read.hide_tray, "the file has to carry it");
        assert_eq!(read.locale, crate::Locale::English);

        let mut browsers = crate::BrowserConfig::default();
        browsers.browsers.push(crate::Browser {
            id: "chrome".to_owned(),
            name: "Google Chrome".to_owned(),
            exe: "chrome.exe".to_owned(),
            profiles: Vec::new(),
            extra_args: Vec::new(),
            private_flag: Some("--incognito".to_owned()),
            icon_path: None,
            custom: true,
            hidden: false,
        });
        store.save_browsers(&browsers).expect("saved");
        let read = Store::at(&dir).browsers().expect("read back");
        assert_eq!(read.browsers.len(), 1, "the file has to carry it");
        assert_eq!(read.browsers[0].id, "chrome");

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A file that is not there is a first run and answers with the defaults. A file that is
    /// there and cannot be read is a fault, and answering it with the defaults would look like
    /// somebody's rules had simply vanished.
    #[test]
    fn a_missing_file_is_a_first_run_and_a_broken_one_is_a_fault() {
        let dir = std::env::temp_dir().join("lu-missing-vs-broken");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("a place to write");

        let said = Store::at(&dir)
            .rules()
            .expect("a first run is not an error");
        assert!(said.rules.is_empty());

        std::fs::write(dir.join("rules.json"), "{ not json at all").expect("written");
        assert!(
            Store::at(&dir).rules().is_err(),
            "a file that is there and unreadable is not an empty set"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// What somebody upgrading from 1.x actually has on disk: no preferences.json, and one small
    /// file per setting beside it. Reading only the theme put the language, the tray and the
    /// shortcut back to their defaults without saying so.
    #[test]
    fn everything_1_x_left_on_disk_is_read_on_the_way_in() {
        let dir = std::env::temp_dir().join("lu-legacy-prefs");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("a place to put them");
        for (named, said) in [
            ("theme", "dark"),
            ("locale", "es"),
            ("hide_tray", "1"),
            ("edge_warning_dismissed", "1"),
            ("global_hotkey", "ctrl+shift+l"),
        ] {
            std::fs::write(dir.join(named), said).expect("written");
        }

        let said = Store::at(&dir).prefs();
        assert_eq!(said.theme, crate::Theme::Dark);
        assert_eq!(said.locale, crate::Locale::Spanish);
        assert!(said.hide_tray);
        assert!(said.edge_warning_dismissed);
        assert_eq!(said.shortcut.as_deref(), Some("Ctrl+Shift+L"));
        assert!(
            said.reachable(),
            "a hidden tray with a shortcut is still reachable"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }
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
