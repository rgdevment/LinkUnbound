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
    #[serde(default)]
    pub found_in_the_shop: Option<ShopSaid>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ShopSaid {
    Offered,
    Landed,
    Queued,
    Paused,
    NotYet,
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
    // A virus scanner or the indexer holds the target for a moment after every write; the
    // rename fails, and a stage the resident never learns of reads as an install gone quiet.
    let saved = crate::store::written_through(&aside, &body)
        .and_then(|()| crate::store::renamed_with_patience(&aside, at));
    if saved.is_err() {
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

#[must_use]
pub fn on_the_candidate_track(here: &str, wants: Option<bool>) -> bool {
    wants.unwrap_or_else(|| {
        semver::Version::parse(here.trim_start_matches('v')).is_ok_and(|here| !here.pre.is_empty())
    })
}

#[must_use]
pub fn worth_offering(found: &str, here: &str, wants: Option<bool>) -> bool {
    if !newer_than(found, here) {
        return false;
    }
    let candidate = semver::Version::parse(found.trim_start_matches('v'))
        .is_ok_and(|found| !found.pre.is_empty());
    !candidate || on_the_candidate_track(here, wants)
}

/// The eight id bytes after the algorithm on line two of base64-wrapped minisign text.
#[must_use]
pub fn key_id(wrapped: &str) -> Option<[u8; 8]> {
    use base64::Engine;
    let text = base64::engine::general_purpose::STANDARD
        .decode(wrapped.trim())
        .ok()?;
    let line = std::str::from_utf8(&text).ok()?.lines().nth(1)?;
    let raw = base64::engine::general_purpose::STANDARD
        .decode(line.trim())
        .ok()?;
    if !matches!(raw.get(..2)?, b"Ed" | b"ED") {
        return None;
    }
    raw.get(2..10)?.try_into().ok()
}

/// Picks the key that checks a download; the plugin still verifies the signature itself.
#[must_use]
pub fn signed_by(signature: &str, pubkey: &str) -> bool {
    key_id(signature).is_some_and(|signed| key_id(pubkey) == Some(signed))
}

#[must_use]
pub fn minor_of(version: &str) -> Option<(u64, u64)> {
    semver::Version::parse(version.trim_start_matches('v'))
        .ok()
        .map(|parsed| (parsed.major, parsed.minor))
}

#[must_use]
pub fn asks_about_automatic(prefs: &crate::Preferences, here: &str, installs_here: bool) -> bool {
    installs_here
        && prefs.looks_for_updates
        && prefs.updates_automatically.is_none()
        && minor_of(here).is_some()
        && prefs.updates_asked_in.as_deref().and_then(minor_of) != minor_of(here)
}

/// A version an earlier attempt already went for is never retried without a press.
#[must_use]
pub fn takes_by_itself(
    prefs: &crate::Preferences,
    looked: &Looked,
    under_way: Option<&Progress>,
    here: &str,
) -> bool {
    prefs.updates_automatically == Some(true)
        && prefs.looks_for_updates
        && looked.found_version.as_deref().is_some_and(|found| {
            worth_offering(found, here, looked.candidates)
                && under_way.is_none_or(|attempt| attempt.version != found)
        })
}

#[cfg(test)]
mod tests {
    use super::{
        Looked, Progress, asks_about_automatic, gone_quiet, keep, looked, minor_of, newer_than,
        on_the_candidate_track, progress, settle, takes_by_itself, tell, under_way_for,
        worth_offering,
    };
    use super::{key_id, signed_by};
    use crate::Preferences;
    use base64::Engine;
    use std::time::Duration;

    fn unanswered() -> Preferences {
        Preferences::default()
    }

    fn asked_in(version: &str) -> Preferences {
        Preferences {
            updates_asked_in: Some(version.to_owned()),
            ..unanswered()
        }
    }

    fn wanting() -> Preferences {
        Preferences {
            updates_automatically: Some(true),
            ..unanswered()
        }
    }

    fn found(version: &str) -> Looked {
        Looked {
            found_version: Some(version.to_owned()),
            ..Looked::default()
        }
    }

    fn attempt(version: &str, stage: &str) -> Progress {
        Progress {
            version: version.to_owned(),
            stage: stage.to_owned(),
            far: 0,
        }
    }

    #[test]
    fn fix_releases_share_the_minor_of_the_version_they_fix() {
        assert_eq!(minor_of("2.11.0"), Some((2, 11)));
        assert_eq!(minor_of("2.11.7"), minor_of("2.11.0"));
        assert_ne!(minor_of("2.12.0"), minor_of("2.11.0"));
        assert_ne!(minor_of("3.11.0"), minor_of("2.11.0"));
    }

    #[test]
    fn a_leading_v_and_a_pre_release_still_name_their_minor() {
        assert_eq!(minor_of("v2.11.3"), Some((2, 11)));
        assert_eq!(minor_of("2.12.0-rc.1"), Some((2, 12)));
    }

    #[test]
    fn a_version_that_is_not_semver_has_no_minor() {
        assert_eq!(minor_of("tomorrow"), None);
        assert_eq!(minor_of("2.11"), None);
        assert_eq!(minor_of(""), None);
    }

    #[test]
    fn a_copy_that_installs_itself_and_never_answered_is_asked() {
        assert!(asks_about_automatic(&unanswered(), "2.11.0", true));
    }

    #[test]
    fn a_copy_that_cannot_install_itself_is_never_asked() {
        assert!(!asks_about_automatic(&unanswered(), "2.11.0", false));
    }

    #[test]
    fn a_copy_that_does_not_look_for_updates_is_never_asked() {
        let prefs = Preferences {
            looks_for_updates: false,
            ..unanswered()
        };
        assert!(!asks_about_automatic(&prefs, "2.11.0", true));
    }

    #[test]
    fn an_answer_of_either_kind_ends_the_asking() {
        for answer in [true, false] {
            let prefs = Preferences {
                updates_automatically: Some(answer),
                ..unanswered()
            };
            assert!(!asks_about_automatic(&prefs, "2.11.0", true));
        }
    }

    #[test]
    fn a_question_put_away_waits_for_the_next_minor_version() {
        assert!(!asks_about_automatic(&asked_in("2.11.0"), "2.11.0", true));
        assert!(
            !asks_about_automatic(&asked_in("2.11.0"), "2.11.4", true),
            "a fix release is not a reason to ask again"
        );
        assert!(!asks_about_automatic(&asked_in("v2.11.0"), "2.11.1", true));
        assert!(asks_about_automatic(&asked_in("2.11.4"), "2.12.0", true));
        assert!(asks_about_automatic(&asked_in("2.11.0"), "3.11.0", true));
    }

    #[test]
    fn a_question_put_away_in_a_garbled_version_is_asked_again() {
        assert!(asks_about_automatic(&asked_in("tomorrow"), "2.11.0", true));
    }

    #[test]
    fn a_copy_whose_own_version_is_not_semver_is_never_asked() {
        assert!(!asks_about_automatic(&unanswered(), "tomorrow", true));
        assert!(!asks_about_automatic(&asked_in("2.11.0"), "tomorrow", true));
    }

    #[test]
    fn a_copy_that_wants_updates_takes_a_newer_version_by_itself() {
        assert!(takes_by_itself(
            &wanting(),
            &found("2.12.0"),
            None,
            "2.11.0"
        ));
    }

    #[test]
    fn a_copy_that_never_answered_or_said_no_takes_nothing_by_itself() {
        let said_no = Preferences {
            updates_automatically: Some(false),
            ..unanswered()
        };
        assert!(!takes_by_itself(
            &unanswered(),
            &found("2.12.0"),
            None,
            "2.11.0"
        ));
        assert!(!takes_by_itself(&said_no, &found("2.12.0"), None, "2.11.0"));
    }

    #[test]
    fn a_copy_that_does_not_look_for_updates_takes_nothing_by_itself() {
        let prefs = Preferences {
            looks_for_updates: false,
            ..wanting()
        };
        assert!(!takes_by_itself(&prefs, &found("2.12.0"), None, "2.11.0"));
    }

    #[test]
    fn nothing_found_means_nothing_to_take() {
        assert!(!takes_by_itself(
            &wanting(),
            &Looked::default(),
            None,
            "2.11.0"
        ));
    }

    #[test]
    fn a_version_that_is_not_newer_is_not_taken() {
        assert!(!takes_by_itself(
            &wanting(),
            &found("2.11.0"),
            None,
            "2.11.0"
        ));
        assert!(!takes_by_itself(
            &wanting(),
            &found("2.10.0"),
            None,
            "2.11.0"
        ));
    }

    #[test]
    fn a_candidate_is_not_taken_by_a_copy_off_the_candidate_track() {
        assert!(!takes_by_itself(
            &wanting(),
            &found("2.12.0-rc.1"),
            None,
            "2.11.0"
        ));
        let on_track = Looked {
            candidates: Some(true),
            ..found("2.12.0-rc.1")
        };
        assert!(takes_by_itself(&wanting(), &on_track, None, "2.11.0"));
    }

    #[test]
    fn a_version_an_earlier_attempt_went_for_is_not_retried_at_any_stage() {
        for stage in ["starting", "getting", "installing", "failed"] {
            assert!(
                !takes_by_itself(
                    &wanting(),
                    &found("2.12.0"),
                    Some(&attempt("2.12.0", stage)),
                    "2.11.0"
                ),
                "retried after {stage}"
            );
        }
    }

    #[test]
    fn an_attempt_at_another_version_does_not_hold_back_a_newer_one() {
        assert!(takes_by_itself(
            &wanting(),
            &found("2.12.1"),
            Some(&attempt("2.12.0", "failed")),
            "2.11.0"
        ));
    }

    #[test]
    fn a_copy_that_is_itself_a_candidate_stays_on_that_track_until_told_otherwise() {
        assert!(on_the_candidate_track("2.1.0-rc.1", None));
        assert!(!on_the_candidate_track("2.1.0", None));
        assert!(on_the_candidate_track("2.1.0", Some(true)));
        assert!(!on_the_candidate_track("2.1.0-rc.1", Some(false)));
        assert!(!on_the_candidate_track("tomorrow", None));
    }

    #[test]
    fn a_leading_v_is_the_same_version_either_way() {
        assert!(on_the_candidate_track("v2.1.0-rc.1", None));
        assert!(worth_offering("v2.2.0", "v2.1.0", None));
    }

    #[test]
    fn a_stable_copy_is_never_walked_onto_a_candidate() {
        assert!(
            !worth_offering("2.2.0-rc.1", "2.1.0", None),
            "the strip offered candidates the errand behind the button then refused"
        );
        assert!(
            worth_offering("2.2.0-rc.1", "2.1.0", Some(true)),
            "asked for"
        );
        assert!(
            worth_offering("2.2.0-rc.1", "2.2.0-rc.0", None),
            "already on it"
        );
    }

    #[test]
    fn nothing_older_or_unreadable_is_ever_worth_offering() {
        assert!(!worth_offering("2.0.0", "2.1.0", None));
        assert!(!worth_offering("2.1.0", "2.1.0", None));
        assert!(!worth_offering("tomorrow", "2.1.0", Some(true)));
        assert!(worth_offering("2.2.0", "2.1.0", None));
    }

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
                found_in_the_shop: Some(super::ShopSaid::Queued),
            },
        );

        let read = looked(&dir);
        assert_eq!(read.checked_at, Some(1_800_000_000));
        assert_eq!(read.found_version.as_deref(), Some("2.1.0"));
        assert_eq!(read.found_route.as_deref(), Some("store"));
        assert_eq!(read.found_installs, Some(false));
        assert_eq!(read.candidates, Some(true), "the track is remembered too");
        assert_eq!(read.found_in_the_shop, Some(super::ShopSaid::Queued));

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

    const MAIN_PUB: &str = "dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXk6IDUxREVBNzU0QUJCQTRBOUEKUldTYVNycXJWS2ZlVWJjeGFJdFc0ZkQrZTRsSUhubG9FaDdseTZzcUl6MTB1NS9TanAzcEE4S0oK";
    const MAIN_SIG: &str = "dW50cnVzdGVkIGNvbW1lbnQ6IHNpZ25hdHVyZSBmcm9tIHRhdXJpIHNlY3JldCBrZXkKUlVTYVNycXJWS2ZlVWZvamc2WWhNZFNQOFp5UExUWi9FbTNxdkZqNWk4eklYRElDRjBTNW1ySHg4Rk5COWQxaVBWYW9EUkNZY1pjblN3RVhSTFFIRENQWjFlMWV4TklXMmdzPQp0cnVzdGVkIGNvbW1lbnQ6IHRpbWVzdGFtcDoxNzk1NzEzNzMxCWZpbGU6bGlua3VuYm91bmQtaW5zdGFsbGVyLTIuMTEuMC13aW5kb3dzLXg4Nl82NC5leGUJdmVyc2lvbjoyLjExLjAKRDhtNHEwVzB3RTBNMUdrVFpUNHN5Wi9oVkpCNUxzbE1IS29RUmUyR3RySnNIcEhOMGF2MU51NVZPUkR4T2RaL0VEQTBIOHUwNlpsVG1xVk8yZEtRQWc9PQo=";
    const SPARE_PUB: &str = "dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXk6IEIwMTA2QjBCRTYyOTVDMjAKUldRZ1hDbm1DMnNRc09zMEVrWXlwSjArSzhLVTZZVnQ1VlY0NWJCdm9NZVdTbXo2dVF1TnI0TFYK";
    const SPARE_SIG: &str = "dW50cnVzdGVkIGNvbW1lbnQ6IHNpZ25hdHVyZSBmcm9tIHRhdXJpIHNlY3JldCBrZXkKUlVRZ1hDbm1DMnNRc0M1a3Y3NElpUnRzNWttdUVjaDB0Y1VVUnBzcnlQV1h1NjVZRTdQbDdKdEd0d3hLS1BOSW1KZWZoL2Zydmc3blFEVmxuWkM5YnUzdU1vRVpZMzdBdFE4PQp0cnVzdGVkIGNvbW1lbnQ6IHRpbWVzdGFtcDoxNzk1NzIyNjI5CWZpbGU6cGF5bG9hZC50eHQKamRucEQxb2lFUVpRdjBzWG81aDdaT09kQXoxczlPcEpUbmFqbmZiRlZWOHVZNnVjS3M1NGEyMnhiSkZzVTdsL0RQOVQzeGZ2TzZhOGEwYXN2ZTNsQ0E9PQo=";

    fn wrap(text: &str) -> String {
        base64::engine::general_purpose::STANDARD.encode(text)
    }

    fn wrap_line_two(raw: &[u8]) -> String {
        let line = base64::engine::general_purpose::STANDARD.encode(raw);
        wrap(&format!("untrusted comment: x\n{line}\n"))
    }

    fn hex_reversed(id: [u8; 8]) -> String {
        id.iter().rev().map(|byte| format!("{byte:02X}")).collect()
    }

    #[test]
    fn a_public_key_and_its_signature_carry_the_same_key_id() {
        assert!(key_id(MAIN_PUB).is_some());
        assert_eq!(key_id(MAIN_PUB), key_id(MAIN_SIG));
        assert_eq!(key_id(SPARE_PUB), key_id(SPARE_SIG));
        assert_ne!(key_id(MAIN_PUB), key_id(SPARE_PUB));
    }

    #[test]
    fn the_key_id_bytes_read_backwards_are_the_hex_minisign_prints() {
        assert_eq!(hex_reversed(key_id(MAIN_PUB).unwrap()), "51DEA754ABBA4A9A");
        assert_eq!(hex_reversed(key_id(SPARE_PUB).unwrap()), "B0106B0BE6295C20");
    }

    #[test]
    fn a_signature_is_signed_by_its_own_key_and_by_no_other() {
        assert!(signed_by(MAIN_SIG, MAIN_PUB));
        assert!(signed_by(SPARE_SIG, SPARE_PUB));
        assert!(!signed_by(MAIN_SIG, SPARE_PUB));
        assert!(!signed_by(SPARE_SIG, MAIN_PUB));
    }

    #[test]
    fn text_that_is_not_minisign_has_no_key_id_and_is_signed_by_nothing() {
        let one_line = wrap("untrusted comment: x");
        let line_not_base64 = wrap("untrusted comment: x\n!!not base64!!\n");
        for bad in [
            "",
            "   ",
            "not base64 at all!",
            one_line.as_str(),
            line_not_base64.as_str(),
        ] {
            assert_eq!(key_id(bad), None, "{bad}");
            assert!(!signed_by(bad, MAIN_PUB), "{bad}");
            assert!(!signed_by(MAIN_SIG, bad), "{bad}");
        }
        assert!(
            !signed_by("", ""),
            "two unreadable texts are not the same key"
        );
    }

    #[test]
    fn a_key_line_too_short_or_of_another_algorithm_has_no_key_id() {
        let mut raw = b"Ed".to_vec();
        raw.extend_from_slice(&[7; 8]);
        assert_eq!(key_id(&wrap_line_two(&raw)), Some([7; 8]));
        raw[0] = b'E';
        raw[1] = b'D';
        assert_eq!(key_id(&wrap_line_two(&raw)), Some([7; 8]));
        assert_eq!(key_id(&wrap_line_two(&raw[..9])), None, "one byte short");
        assert_eq!(key_id(&wrap_line_two(b"Ed")), None);
        assert_eq!(key_id(&wrap_line_two(b"")), None);
        raw[0] = b'X';
        assert_eq!(key_id(&wrap_line_two(&raw)), None);
        raw[1] = b'X';
        assert_eq!(key_id(&wrap_line_two(&raw)), None);
        raw[0] = b'E';
        raw[1] = b'x';
        assert_eq!(key_id(&wrap_line_two(&raw)), None);
    }

    #[test]
    fn only_the_key_id_decides_so_a_different_id_never_matches() {
        let mut raw = b"Ed".to_vec();
        raw.extend_from_slice(&[1; 8]);
        let one = wrap_line_two(&raw);
        raw[9] = 2;
        let other = wrap_line_two(&raw);
        assert!(signed_by(&one, &one));
        assert!(!signed_by(&one, &other));
    }
}
