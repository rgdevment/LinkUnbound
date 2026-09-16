//! What the settings window found out about updates, kept in two small files beside the
//! preferences so the resident — another process, with no network of its own — can read them:
//! the last look, and the install under way.

use std::path::{Path, PathBuf};

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

fn read<T: for<'de> Deserialize<'de>>(at: &Path) -> Option<T> {
    std::fs::read_to_string(at)
        .ok()
        .and_then(|raw| serde_json::from_str(crate::unmarked(&raw)).ok())
}

fn write<T: Serialize>(dir: &Path, at: &Path, value: &T) {
    let Ok(body) = serde_json::to_string_pretty(value) else {
        return;
    };
    let _ = std::fs::create_dir_all(dir);
    let name = at
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    let aside = dir.join(format!("{name}.{}.tmp", std::process::id()));
    if std::fs::write(&aside, body).is_ok() && std::fs::rename(&aside, at).is_err() {
        let _ = std::fs::remove_file(&aside);
    }
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
    use super::{Looked, Progress, keep, looked, newer_than, progress, settle, tell};

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
                candidates: Some(true),
            },
        );

        let read = looked(&dir);
        assert_eq!(read.checked_at, Some(1_800_000_000));
        assert_eq!(read.found_version.as_deref(), Some("2.1.0"));
        assert_eq!(read.found_route.as_deref(), Some("store"));
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
