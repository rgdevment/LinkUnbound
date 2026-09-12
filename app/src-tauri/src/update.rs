use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// The three feeds live on an orphan branch that holds nothing else. Serving them from the
/// release of each version is what cannot work: `releases/latest/download/` follows the date a
/// release was published rather than the version in it, and never points at a prerelease at all,
/// so the candidate channel would be dead and a maintenance release for an older series would
/// walk the feed backwards. A branch also leaves the feed's history in git, where a bad write is
/// one revert away.
const MANIFEST: &str =
    "https://raw.githubusercontent.com/rgdevment/LinkUnbound/manifest/release-manifest.json";
/// One holds a stable version and the other a candidate, never both, so the track a copy belongs
/// to can be read off the version it is being offered.
pub const LATEST: &str =
    "https://raw.githubusercontent.com/rgdevment/LinkUnbound/manifest/latest.json";
pub const CANDIDATE: &str =
    "https://raw.githubusercontent.com/rgdevment/LinkUnbound/manifest/candidate.json";

const PATIENCE: Duration = Duration::from_secs(5);
const APART: u64 = 24 * 60 * 60;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Route {
    Store,
    Brew,
    Download,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Kept {
    pub route: Route,
    pub package: Option<&'static str>,
}

impl Kept {
    const fn plain(route: Route) -> Self {
        Self {
            route,
            package: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Ready {
    pub version: String,
    pub route: Route,
    pub package: Option<&'static str>,
    pub installs: bool,
}

/// A copy the store keeps cannot replace itself: `WindowsApps` is read only and the package
/// identity is the store's. It gets taken to the store instead.
pub const fn self_installs(route: Route) -> bool {
    matches!(route, Route::Brew | Route::Download)
}

/// Where a release of ours can possibly come from. The plugin fetches whatever address the feed
/// names, so a feed that was tampered with could otherwise send the download anywhere.
const FROM: &str = "github.com";
/// Where GitHub serves the bytes a release asset redirects to. It carries no path of ours to
/// recognise, so an asset from here is only ever reached by following our own release.
const STORED_AT: &str = "objects.githubusercontent.com";
const RELEASES: &str = "/rgdevment/LinkUnbound/releases/download/";

/// The signature covers the installer's bytes and nothing else: not the manifest, not the version
/// the manifest claims. So a feed that was tampered with can name a new version and hand over the
/// signed installer of an old one, and it verifies. Pinning the address to the tag of the version
/// being offered is what ties the bytes to the claim.
#[must_use]
pub fn ours(url: &str, version: &str) -> bool {
    let Ok(at) = url.parse::<url::Url>() else {
        return false;
    };
    if at.scheme() != "https" || at.port().is_some() {
        return false;
    }
    match at.host_str() {
        Some(STORED_AT) => true,
        Some(FROM) => at.path().starts_with(&format!("{RELEASES}v{version}/")),
        _ => false,
    }
}

#[must_use]
pub fn channel_for(version: &str) -> &'static str {
    match version.parse::<semver::Version>() {
        Ok(said) if !said.pre.is_empty() => CANDIDATE,
        _ => LATEST,
    }
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct Manifest {
    latest: String,
    #[serde(default)]
    latest_prerelease: Option<String>,
}

/// Which track a copy is on. Nobody having said is not the same as somebody having said no: a
/// candidate that was installed by hand goes on being offered candidates until its owner says
/// otherwise, and saying otherwise is what walks it back to the stable track.
#[must_use]
pub fn tracking(now: &str, wants: Option<bool>) -> bool {
    wants.unwrap_or_else(|| {
        now.parse::<semver::Version>()
            .is_ok_and(|here| !here.pre.is_empty())
    })
}

#[must_use]
pub fn newer(now: &str, manifest: &str, kept: Kept, wants: Option<bool>) -> Option<Ready> {
    let here: semver::Version = now.parse().ok()?;
    let read: Manifest = serde_json::from_str(manifest).ok()?;

    let mut best: semver::Version = read.latest.parse().ok()?;
    // A stable copy stays on the stable track whatever the manifest says, so a hostile one cannot
    // walk it onto a less-tested build. Asking for the candidates is the one way across, and it is
    // asked for here, not out there.
    let tracking = tracking(now, wants);
    if !tracking && !best.pre.is_empty() {
        return None;
    }
    if tracking
        && let Some(said) = read.latest_prerelease.as_deref()
        && let Ok(candidate) = said.parse::<semver::Version>()
        && candidate > best
    {
        best = candidate;
    }

    (best > here).then(|| offered(best.to_string(), kept))
}

/// `self_installs` says no for the Store because a copy kept there cannot replace itself from a
/// download; an offer the Store itself made is the one it can take without leaving the window.
#[must_use]
pub fn from_the_shop(version: &str, now: &str) -> Option<Ready> {
    let here: semver::Version = now.parse().ok()?;
    let said: semver::Version = version.parse().ok()?;

    (said > here).then(|| Ready {
        version: said.to_string(),
        route: Route::Store,
        package: None,
        installs: true,
    })
}

fn offered(version: String, kept: Kept) -> Ready {
    Ready {
        version,
        route: kept.route,
        package: kept.package,
        installs: self_installs(kept.route),
    }
}

/// What the last look found, so closing the window does not take the offer away with it. The copy
/// may have moved between a download and a cask since, so where it stands is read again.
#[must_use]
pub fn remembered(now: &str, said: Option<&str>, kept: Kept) -> Option<Ready> {
    let here: semver::Version = now.parse().ok()?;
    let kept_version: semver::Version = said?.parse().ok()?;

    (kept_version > here).then(|| offered(kept_version.to_string(), kept))
}

/// A clock put back leaves the last look in the future, and a copy that only counts forward from
/// it would wait out the difference before ever looking again.
#[must_use]
pub fn due(last: Option<u64>, now: u64) -> bool {
    last.is_none_or(|at| at > now || now - at >= APART)
}

#[must_use]
pub fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |since| since.as_secs())
}

#[must_use]
pub fn route() -> Kept {
    chosen(std::env::current_exe().ok().as_deref(), |at| at.is_dir())
}

/// A copy running from the mounted disk image cannot replace itself: the volume is read only, and
/// the plugin only finds that out after the whole download.
#[must_use]
pub fn mounted(running: Option<&Path>) -> bool {
    cfg!(target_os = "macos")
        && running.is_some_and(|at| at.starts_with("/Volumes/") || at.starts_with("/private/tmp/"))
}

#[must_use]
pub fn from_a_mount() -> bool {
    mounted(std::env::current_exe().ok().as_deref())
}

const PREFIXES: [&str; 2] = ["/opt/homebrew", "/usr/local"];
const CASKS: [&str; 2] = ["linkunbound", "linkunbound-beta"];

fn chosen(running: Option<&Path>, there: impl Fn(&Path) -> bool) -> Kept {
    let packaged = running.is_some_and(|at| {
        at.to_string_lossy()
            .split(['/', '\\'])
            .any(|part| part.eq_ignore_ascii_case("WindowsApps"))
    });
    if packaged {
        return Kept::plain(Route::Store);
    }

    for package in CASKS {
        let brewed = PREFIXES
            .iter()
            .any(|root| there(Path::new(&format!("{root}/Caskroom/{package}"))));
        if brewed {
            return Kept {
                route: Route::Brew,
                package: Some(package),
            };
        }
    }
    Kept::plain(Route::Download)
}

#[must_use]
pub fn fetch() -> Option<String> {
    let asked = reqwest::blocking::Client::builder()
        .timeout(PATIENCE)
        .user_agent(concat!("linkunbound/", env!("CARGO_PKG_VERSION")))
        .build()
        .ok()?
        .get(MANIFEST)
        .send()
        .ok()?;

    asked.status().is_success().then(|| asked.text().ok())?
}

/// Kept beside the settings, not in them: when the last look happened is not something the user
/// chose, and resetting the configuration has no business deciding an update is owed.
#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct Looked {
    #[serde(default)]
    pub checked_at: Option<u64>,
    #[serde(default)]
    pub found_version: Option<String>,
    /// Unset until somebody chooses, which is not the same as having chosen no.
    #[serde(default)]
    pub candidates: Option<bool>,
}

fn beside(dir: &Path) -> std::path::PathBuf {
    dir.join("update.json")
}

#[must_use]
pub fn looked(dir: &Path) -> Looked {
    std::fs::read_to_string(beside(dir))
        .ok()
        .and_then(|raw| serde_json::from_str(linkunbound_core::unmarked(&raw)).ok())
        .unwrap_or_default()
}

/// Written the way the rest of the store is: through a sibling file and a rename, so a crash
/// leaves either the old content or the new one, never half of either.
pub fn keep(dir: &Path, looked: &Looked) {
    let Ok(body) = serde_json::to_string_pretty(looked) else {
        return;
    };
    let _ = std::fs::create_dir_all(dir);
    let aside = dir.join(format!("update.{}.tmp", std::process::id()));
    if std::fs::write(&aside, body).is_ok() && std::fs::rename(&aside, beside(dir)).is_err() {
        let _ = std::fs::remove_file(&aside);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FEED: &str = r#"{"schema":1,"latest":"2.1.0","latestPrerelease":"2.2.0-rc1"}"#;

    #[test]
    fn a_newer_release_is_offered() {
        let found =
            newer("2.0.0", FEED, Kept::plain(Route::Download), None).expect("2.1.0 is newer");

        assert_eq!(found.version, "2.1.0");
    }

    #[test]
    fn a_manifest_cannot_walk_a_stable_copy_onto_the_candidates_track() {
        let feed = r#"{"latest":"9.9.9-rc1"}"#;

        assert!(newer("2.0.0", feed, Kept::plain(Route::Download), None).is_none());
    }

    #[test]
    fn a_candidate_is_offered_the_newest_of_either() {
        assert_eq!(
            newer("2.1.0-rc1", FEED, Kept::plain(Route::Download), None)
                .unwrap()
                .version,
            "2.2.0-rc1"
        );
    }

    /// `2.2.0` is greater than `2.2.0-rc1` in semver, but any comparison that looked at the three
    /// numbers alone would call them equal and leave the user stuck on the candidate on the very
    /// day it was meant to be replaced.
    #[test]
    fn a_candidate_is_replaced_by_the_release_it_was_a_candidate_for() {
        let feed = r#"{"latest":"2.2.0","latestPrerelease":"2.2.0-rc1"}"#;

        let found = newer("2.2.0-rc1", feed, Kept::plain(Route::Download), None)
            .expect("the release it was a candidate for is newer than it");
        assert_eq!(found.version, "2.2.0");

        assert!(
            newer("2.2.0", feed, Kept::plain(Route::Download), None).is_none(),
            "and once taken, the candidate is not offered back"
        );
    }

    #[test]
    fn a_candidate_takes_the_stable_one_when_it_is_ahead() {
        let feed = r#"{"latest":"2.3.0","latestPrerelease":"2.2.0-rc1"}"#;

        assert_eq!(
            newer("2.2.0-rc1", feed, Kept::plain(Route::Download), None)
                .unwrap()
                .version,
            "2.3.0"
        );
    }

    #[test]
    fn the_same_version_is_not_an_update() {
        assert!(newer("2.1.0", FEED, Kept::plain(Route::Download), None).is_none());
        assert!(newer("2.2.0-rc1", FEED, Kept::plain(Route::Download), None).is_none());
    }

    #[test]
    fn a_manifest_that_makes_no_sense_says_nothing() {
        assert!(newer("2.0.0", "not json", Kept::plain(Route::Download), None).is_none());
        assert!(
            newer(
                "2.0.0",
                r#"{"latest":"tomorrow"}"#,
                Kept::plain(Route::Download),
                None
            )
            .is_none()
        );
    }

    #[test]
    fn a_manifest_without_a_candidate_still_reads() {
        let feed = r#"{"latest":"2.1.0"}"#;

        assert_eq!(
            newer("2.0.0-rc1", feed, Kept::plain(Route::Download), None)
                .unwrap()
                .version,
            "2.1.0"
        );
    }

    /// The switch exists to move this, and nothing was asserting that it did: with the track
    /// hard-coded either way the rest of these tests still passed.
    #[test]
    fn a_candidate_is_only_taken_by_a_copy_that_asked_for_one() {
        let feed = r#"{"latest":"2.1.0","latestPrerelease":"2.2.0-rc1"}"#;

        assert_eq!(
            newer("2.1.0", feed, Kept::plain(Route::Download), Some(true))
                .unwrap()
                .version,
            "2.2.0-rc1",
            "a stable copy that asked for candidates crosses over"
        );
        assert!(
            newer("2.1.0", feed, Kept::plain(Route::Download), None).is_none(),
            "and one that never asked stays where it is"
        );
    }

    /// Saying no is what walks a candidate back, and it lands on the stable release that follows.
    #[test]
    fn a_candidate_that_asks_to_come_back_is_offered_the_stable_track() {
        let feed = r#"{"latest":"2.2.0","latestPrerelease":"2.3.0-rc1"}"#;

        assert_eq!(
            newer("2.2.0-rc1", feed, Kept::plain(Route::Download), Some(false))
                .unwrap()
                .version,
            "2.2.0"
        );
    }

    #[test]
    fn which_track_a_copy_is_on_is_read_off_its_version_until_somebody_says() {
        assert!(tracking("2.1.0-rc1", None), "a candidate was installed");
        assert!(!tracking("2.1.0", None), "a stable one was");
        assert!(tracking("2.1.0", Some(true)), "asked for");
        assert!(!tracking("2.1.0-rc1", Some(false)), "asked to come back");
        assert!(!tracking("tomorrow", None), "unreadable means stable");
    }

    /// The store's own offer is the one a store copy can take without leaving the window, and it
    /// is the only place `installs` is decided by anything other than the route.
    #[test]
    fn an_offer_the_store_itself_made_is_one_it_can_install() {
        let found = from_the_shop("2.1.0", "2.0.0").expect("the store has something newer");

        assert_eq!(found.version, "2.1.0");
        assert_eq!(found.route, Route::Store);
        assert!(
            found.installs,
            "a store copy cannot install from a download, but it can take what the store offered"
        );
        assert!(
            !self_installs(Route::Store),
            "which is why the route alone must not decide this"
        );
    }

    #[test]
    fn the_store_offering_nothing_newer_is_not_an_update() {
        assert!(from_the_shop("2.0.0", "2.0.0").is_none());
        assert!(from_the_shop("1.9.0", "2.0.0").is_none());
        assert!(
            from_the_shop("2.0.0.4", "2.0.0").is_none(),
            "a package version has four parts and a release has three"
        );
    }

    #[test]
    fn only_a_copy_that_owns_its_own_folder_replaces_itself() {
        assert!(self_installs(Route::Download));
        assert!(self_installs(Route::Brew));
        assert!(!self_installs(Route::Store), "the store keeps its own");
    }

    #[test]
    fn an_installer_is_only_ever_taken_from_where_our_releases_live() {
        assert!(ours(
            "https://github.com/rgdevment/LinkUnbound/releases/download/v2.1.0/linkunbound.exe",
            "2.1.0"
        ));
        assert!(ours(
            "https://objects.githubusercontent.com/whatever",
            "2.1.0"
        ));

        assert!(
            !ours("http://github.com/rgdevment/LinkUnbound/x.exe", "2.1.0"),
            "plain http"
        );
        assert!(
            !ours("https://github.com.example.invalid/x.exe", "2.1.0"),
            "a lookalike host"
        );
        assert!(
            !ours("https://raw.githubusercontent.com/x.exe", "2.1.0"),
            "not where releases live"
        );
        assert!(!ours("file:///C:/x.exe", "2.1.0"));
        assert!(!ours("nonsense", "2.1.0"));
    }

    /// The signature proves the bytes were ours; it says nothing about which version they are.
    /// Without pinning the tag, a feed could announce a new version and serve the signed
    /// installer of an old one, and the update would verify and roll the copy backwards.
    #[test]
    fn a_feed_cannot_hand_over_the_installer_of_a_different_version() {
        let old = "https://github.com/rgdevment/LinkUnbound/releases/download/v2.0.0/setup.exe";

        assert!(
            !ours(old, "2.1.0"),
            "announcing 2.1.0 and serving the 2.0.0 installer is a rollback"
        );
        assert!(ours(old, "2.0.0"), "and the same address is fine for 2.0.0");
    }

    /// Only our own releases, not any release on the host.
    #[test]
    fn somebody_elses_repository_is_not_ours() {
        for at in [
            "https://github.com/attacker/LinkUnbound/releases/download/v2.1.0/x.exe",
            "https://github.com/rgdevment/Other/releases/download/v2.1.0/x.exe",
            "https://github.com/rgdevment/LinkUnbound/raw/main/x.exe",
        ] {
            assert!(!ours(at, "2.1.0"), "{at}");
        }
    }

    #[test]
    fn a_port_of_its_own_is_not_our_release_page() {
        assert!(!ours(
            "https://github.com:8443/rgdevment/LinkUnbound/releases/download/v2.1.0/x.exe",
            "2.1.0"
        ));
    }

    #[test]
    fn a_candidate_asks_the_candidates_feed_and_a_stable_one_the_stable_feed() {
        assert_eq!(channel_for("2.1.0"), LATEST);
        assert_eq!(channel_for("2.2.0-rc1"), CANDIDATE);
        assert_eq!(channel_for("tomorrow"), LATEST, "unreadable means stable");
    }

    #[test]
    fn the_feed_follows_what_is_offered_rather_than_what_is_running() {
        let feed = r#"{"latest":"2.3.0","latestPrerelease":"2.2.0-rc1"}"#;
        let found =
            newer("2.2.0-rc1", feed, Kept::plain(Route::Download), None).expect("2.3.0 is newer");

        assert_eq!(
            channel_for(&found.version),
            LATEST,
            "a candidate sent to a stable release must be pointed at the stable feed"
        );
    }

    #[test]
    fn what_was_found_is_still_offered_after_the_window_closed() {
        let kept = Kept::plain(Route::Download);

        assert_eq!(
            remembered("2.0.0", Some("2.1.0"), kept).unwrap().version,
            "2.1.0"
        );
        assert!(remembered("2.1.0", Some("2.1.0"), kept).is_none());
        assert!(
            remembered("2.2.0", Some("2.1.0"), kept).is_none(),
            "a copy updated by hand is not owed the old offer"
        );
        assert!(remembered("2.0.0", None, kept).is_none());
        assert!(remembered("2.0.0", Some("tomorrow"), kept).is_none());
    }

    #[test]
    fn an_offer_says_whether_this_copy_can_take_it() {
        let feed = r#"{"latest":"2.1.0","store":"2.1.0"}"#;

        assert!(
            newer("2.0.0", feed, Kept::plain(Route::Download), None)
                .unwrap()
                .installs
        );
        assert!(
            !newer("2.0.0", feed, Kept::plain(Route::Store), None)
                .unwrap()
                .installs
        );
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn a_copy_still_inside_its_disk_image_knows_it_cannot_replace_itself() {
        let at = Path::new("/Volumes/LinkUnbound/LinkUnbound.app/Contents/MacOS/linkunbound");

        assert!(mounted(Some(at)));
        assert!(!mounted(Some(Path::new(APP))));
        assert!(!mounted(None));
    }

    #[test]
    #[cfg(not(target_os = "macos"))]
    fn nothing_is_mounted_anywhere_but_a_mac() {
        assert!(!mounted(Some(Path::new(
            "/Volumes/LinkUnbound/LinkUnbound.app"
        ))));
    }

    fn nowhere(_: &Path) -> bool {
        false
    }

    fn at(said: &str) -> Option<&Path> {
        Some(Path::new(said))
    }

    fn only(named: &'static str) -> impl Fn(&Path) -> bool {
        move |what| what == Path::new(named)
    }

    const APP: &str = "/Applications/LinkUnbound.app/Contents/MacOS/linkunbound";

    const MSIX: &str = r"C:\Program Files\WindowsApps\rgdevment.LinkUnbound_2.0.0.0_x64__8wekyb3d8bbwe\linkunbound-settings.exe";

    #[test]
    fn a_copy_under_windowsapps_is_kept_by_the_store() {
        assert_eq!(chosen(at(MSIX), nowhere), Kept::plain(Route::Store));
    }

    #[test]
    fn the_separator_is_read_the_same_on_every_system() {
        assert_eq!(
            chosen(at(&MSIX.replace('\\', "/")), nowhere),
            Kept::plain(Route::Store)
        );
    }

    #[test]
    fn a_folder_is_named_windowsapps_or_it_is_not() {
        let alike = r"C:\Program Files\WindowsAppsBackup\LinkUnbound\linkunbound-settings.exe";

        assert_eq!(chosen(at(alike), nowhere), Kept::plain(Route::Download));
        assert_eq!(
            chosen(at(&MSIX.to_lowercase()), nowhere),
            Kept::plain(Route::Store),
            "Windows does not distinguish the case of a folder"
        );
    }

    #[test]
    fn a_cask_answers_with_the_name_it_was_installed_under() {
        assert_eq!(
            chosen(at(APP), only("/opt/homebrew/Caskroom/linkunbound")),
            Kept {
                route: Route::Brew,
                package: Some("linkunbound")
            }
        );
        assert_eq!(
            chosen(at(APP), only("/opt/homebrew/Caskroom/linkunbound-beta")),
            Kept {
                route: Route::Brew,
                package: Some("linkunbound-beta")
            }
        );
    }

    #[test]
    fn the_older_homebrew_root_answers_too() {
        assert_eq!(
            chosen(at(APP), only("/usr/local/Caskroom/linkunbound")),
            Kept {
                route: Route::Brew,
                package: Some("linkunbound")
            }
        );
    }

    #[test]
    fn what_is_running_wins_over_what_is_merely_installed() {
        assert_eq!(chosen(at(MSIX), |_| true), Kept::plain(Route::Store));
    }

    #[test]
    fn everything_else_gets_the_page() {
        assert_eq!(
            chosen(
                at(r"C:\Program Files\LinkUnbound\linkunbound-settings.exe"),
                nowhere
            ),
            Kept::plain(Route::Download)
        );
        assert_eq!(chosen(at(APP), nowhere), Kept::plain(Route::Download));
        assert_eq!(chosen(None, nowhere), Kept::plain(Route::Download));
    }

    #[test]
    fn nothing_is_owed_before_the_interval_is_up() {
        let now = 1_800_000_000;

        assert!(due(None, now), "a copy that never looked should look");
        assert!(!due(Some(now - 3 * 60 * 60), now));
        assert!(due(Some(now - 25 * 60 * 60), now));
    }

    /// Counting forward from a look that sits in the future would wait out the whole difference,
    /// which after a clock is put back can be years.
    #[test]
    fn a_look_left_in_the_future_is_owed_now_rather_than_waited_out() {
        assert!(due(Some(2_000_000_000), 1_800_000_000));
    }

    #[test]
    fn what_the_last_look_found_survives_a_restart() {
        let dir = std::env::temp_dir().join("lu-update-test");
        let _ = std::fs::remove_dir_all(&dir);

        assert!(looked(&dir).checked_at.is_none());

        keep(
            &dir,
            &Looked {
                checked_at: Some(1_800_000_000),
                found_version: Some("2.1.0".to_owned()),
                candidates: Some(true),
            },
        );

        let read = looked(&dir);
        assert_eq!(read.checked_at, Some(1_800_000_000));
        assert_eq!(read.found_version.as_deref(), Some("2.1.0"));
        assert_eq!(read.candidates, Some(true), "the track is remembered too");

        let _ = std::fs::remove_dir_all(&dir);
    }
}
