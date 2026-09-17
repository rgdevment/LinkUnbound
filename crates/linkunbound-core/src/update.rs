//! What the settings window found out about updates, kept in two small files beside the
//! preferences so the resident — another process, with no network of its own — can read them:
//! the last look, and the install under way.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use serde::{Deserialize, Serialize};

/// Kept beside the settings, not in them: when the last look happened is not something the user
/// chose, and resetting the configuration has no business deciding an update is owed.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Looked {
    #[serde(default)]
    pub checked_at: Option<u64>,
    #[serde(default)]
    pub found_version: Option<String>,
    /// Where the found version would come from — `store` is the one the resident cannot take by
    /// itself, since the Store's own installer wants a window to talk to.
    #[serde(default)]
    pub found_route: Option<String>,
    /// Whether this copy can replace itself at all: a copy run from a disk image cannot, and a
    /// button that would only ever fail is worse than a door to the window that explains why.
    #[serde(default)]
    pub found_installs: Option<bool>,
    /// Unset until somebody chooses, which is not the same as having chosen no.
    #[serde(default)]
    pub candidates: Option<bool>,
}

/// The install under way, written by whoever is downloading for whoever is watching.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Progress {
    pub version: String,
    /// `starting`, `getting`, `installing` or `failed`.
    pub stage: String,
    #[serde(default)]
    pub far: u64,
}

fn beside(dir: &Path) -> PathBuf {
    dir.join("update.json")
}

fn under_way(dir: &Path) -> PathBuf {
    dir.join("updating.json")
}

#[must_use]
pub fn looked(dir: &Path) -> Looked {
    read(&beside(dir)).unwrap_or_default()
}

/// Written the way the rest of the store is: through a sibling file and a rename, so a crash
/// leaves either the old content or the new one, never half of either.
pub fn keep(dir: &Path, looked: &Looked) {
    write(dir, &beside(dir), looked);
}

#[must_use]
pub fn progress(dir: &Path) -> Option<Progress> {
    read(&under_way(dir))
}

pub fn tell(dir: &Path, progress: &Progress) {
    write(dir, &under_way(dir), progress);
}

/// Nothing under way any more: a stale file would show the last install's bar on the next offer.
pub fn settle(dir: &Path) {
    let _ = std::fs::remove_file(under_way(dir));
}

/// How long ago the install under way last said anything. `None` when it never did, or when the
/// clock makes the answer meaningless — which the caller has to read as "too long".
#[must_use]
pub fn progress_age(dir: &Path) -> Option<Duration> {
    std::fs::metadata(under_way(dir))
        .and_then(|m| m.modified())
        .ok()
        .and_then(|at| at.elapsed().ok())
}

/// A download reports every second while it lives; an install hands over to the installer and
/// says nothing more, so it is given the minutes an installer can take. Silence past that is a
/// process that is no longer there, and a file whose age cannot be read is read the same way:
/// a clock that went backwards must not keep a dead bar on screen for the session.
#[must_use]
pub fn gone_quiet(stage: &str, age: Option<Duration>) -> bool {
    let allowed = match stage {
        "starting" | "getting" => Duration::from_secs(120),
        "installing" => Duration::from_secs(600),
        _ => return false,
    };
    age.is_none_or(|since| since > allowed)
}

/// Whether the file says an install of `version` is alive: told recently, and not over.
#[must_use]
pub fn under_way_for(dir: &Path, version: &str) -> bool {
    progress(dir).is_some_and(|one| {
        one.version == version
            && matches!(one.stage.as_str(), "starting" | "getting" | "installing")
            && !gone_quiet(&one.stage, progress_age(dir))
    })
}

fn read<T: for<'de> Deserialize<'de>>(at: &Path) -> Option<T> {
    std::fs::read_to_string(at)
        .ok()
        .and_then(|raw| serde_json::from_str(crate::unmarked(&raw)).ok())
}

/// Two writes from one process — the window's look and a switch flipped meanwhile — must not
/// share a staging file, or the later truncates what the earlier is about to rename.
static WRITES: AtomicU64 = AtomicU64::new(0);

fn write<T: Serialize>(dir: &Path, at: &Path, value: &T) {
    let Ok(body) = serde_json::to_string_pretty(value) else {
        return;
    };
    let _ = std::fs::create_dir_all(dir);
    let name = at
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    let nth = WRITES.fetch_add(1, Ordering::Relaxed);
    let aside = dir.join(format!("{name}.{}.{nth}.tmp", std::process::id()));
    if std::fs::write(&aside, body).is_err() {
        return;
    }
    // A virus scanner or the indexer holds the target for a moment after every write; the
    // rename fails, and a stage the resident never learns of reads as an install gone quiet.
    if !renamed_with_patience(&aside, at) {
        let _ = std::fs::remove_file(&aside);
    }
}

fn renamed_with_patience(from: &Path, to: &Path) -> bool {
    for attempt in 0..5 {
        if std::fs::rename(from, to).is_ok() {
            return true;
        }
        std::thread::sleep(std::time::Duration::from_millis(20 * (attempt + 1)));
    }
    false
}

/// Whether what was found is newer than the copy asking. A version that does not read as semver
/// is never newer: offering it would put a button on screen that no installer can honour.
#[must_use]
pub fn newer_than(found: &str, here: &str) -> bool {
    match (
        semver::Version::parse(found.trim_start_matches('v')),
        semver::Version::parse(here.trim_start_matches('v')),
    ) {
        (Ok(found), Ok(here)) => found > here,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Looked, Progress, gone_quiet, keep, looked, newer_than, progress, settle, tell,
        under_way_for,
    };
    use std::time::Duration;

    fn scratch(name: &str) -> std::path::PathBuf {
        let dir =
            std::env::temp_dir().join(format!("lu-core-update-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn what_the_last_look_found_survives_a_restart() {
        let dir = scratch("looked");
        assert!(looked(&dir).checked_at.is_none());

        keep(
            &dir,
            &Looked {
                checked_at: Some(1_800_000_000),
                found_version: Some("2.1.0".to_owned()),
                found_route: Some("store".to_owned()),
                found_installs: Some(false),
                candidates: Some(true),
            },
        );

        let read = looked(&dir);
        assert_eq!(read.checked_at, Some(1_800_000_000));
        assert_eq!(read.found_version.as_deref(), Some("2.1.0"));
        assert_eq!(read.found_route.as_deref(), Some("store"));
        assert_eq!(read.found_installs, Some(false));
        assert_eq!(read.candidates, Some(true), "the track is remembered too");

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A file written before the route was recorded still reads; the route is simply unknown.
    #[test]
    fn a_look_written_before_the_route_existed_still_reads() {
        let dir = scratch("old");
        let _ = std::fs::create_dir_all(&dir);
        std::fs::write(
            dir.join("update.json"),
            r#"{"checked_at": 1, "found_version": "2.0.1", "candidates": null}"#,
        )
        .unwrap();
        let read = looked(&dir);
        assert_eq!(read.found_version.as_deref(), Some("2.0.1"));
        assert!(read.found_route.is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_install_under_way_is_told_and_then_settled() {
        let dir = scratch("progress");
        assert!(progress(&dir).is_none());

        tell(
            &dir,
            &Progress {
                version: "2.0.1".to_owned(),
                stage: "getting".to_owned(),
                far: 42,
            },
        );
        let seen = progress(&dir).expect("written");
        assert_eq!(seen.stage, "getting");
        assert_eq!(seen.far, 42);

        settle(&dir);
        assert!(progress(&dir).is_none(), "settled means gone, not zeroed");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A download that stopped talking is dead; an install gets the minutes an installer takes;
    /// a file whose age cannot be read is treated as dead rather than immortal; and what is over
    /// is never quiet, since there is nothing left to wait for.
    #[test]
    fn silence_is_read_by_stage_and_a_lost_clock_ends_the_wait() {
        let s = Duration::from_secs;
        assert!(!gone_quiet("getting", Some(s(119))));
        assert!(gone_quiet("getting", Some(s(121))));
        assert!(!gone_quiet("installing", Some(s(121))));
        assert!(gone_quiet("installing", Some(s(601))));
        assert!(
            gone_quiet("starting", None),
            "no readable age is no proof of life"
        );
        assert!(!gone_quiet("failed", None));
        assert!(!gone_quiet("failed", Some(s(9_999))));
    }

    #[test]
    fn an_install_just_told_is_under_way_and_an_old_or_other_one_is_not() {
        let dir = scratch("alive");
        assert!(!under_way_for(&dir, "2.0.1"));
        tell(
            &dir,
            &Progress {
                version: "2.0.1".to_owned(),
                stage: "getting".to_owned(),
                far: 3,
            },
        );
        assert!(under_way_for(&dir, "2.0.1"));
        assert!(
            !under_way_for(&dir, "2.0.2"),
            "another version's install is not this one's"
        );
        tell(
            &dir,
            &Progress {
                version: "2.0.1".to_owned(),
                stage: "failed".to_owned(),
                far: 0,
            },
        );
        assert!(!under_way_for(&dir, "2.0.1"), "failed is over");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// The comparison is semver, so a candidate sorts below its release and a version the
    /// updater could not parse is never offered.
    #[test]
    fn only_a_version_that_reads_as_newer_is_newer() {
        assert!(newer_than("2.0.1", "2.0.0"));
        assert!(newer_than("2.1.0-rc.1", "2.0.0"));
        assert!(
            !newer_than("2.0.0-rc.2", "2.0.0"),
            "a candidate of the same release is older"
        );
        assert!(!newer_than("2.0.0", "2.0.0"));
        assert!(!newer_than("1.4.0", "2.0.0"));
        assert!(!newer_than("soon", "2.0.0"));
        assert!(
            newer_than("v2.0.1", "2.0.0"),
            "a leading v is how tags are written"
        );
    }
}
