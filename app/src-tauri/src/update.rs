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

/// Market and language are fixed, so the request says nothing of the person making it.
const CATALOG: &str =
    "https://displaycatalog.mp.microsoft.com/v7.0/products/9N9F7C8Q43KC?market=US&languages=en-US";
const STORE_FAMILY: &str = "rgdevment.LinkUnbound-BrowserPicker_kdjgfdc2rb3gc";
const STORE_ARCH: &str = if cfg!(target_arch = "aarch64") {
    "arm64"
} else {
    "x64"
};
const LISTING_AT_MOST: u64 = 256 * 1024;

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
    pub coming: Option<Coming>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Coming {
    Queued,
    Paused,
    NotYet,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Look {
    Now,
    Pressed,
}

impl Route {
    /// The word the resident reads back: it cannot take a Store update itself and has to know.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Store => "store",
            Self::Brew => "brew",
            Self::Download => "download",
        }
    }
}

/// A copy the store keeps cannot replace itself: `WindowsApps` is read only and the package
/// identity is the store's. It gets taken to the store instead.
pub const fn self_installs(route: Route) -> bool {
    matches!(route, Route::Brew | Route::Download)
}

/// Where a release of ours can possibly come from. The plugin fetches whatever address the feed
/// names, so a feed that was tampered with could otherwise send the download anywhere.
const FROM: &str = "github.com";
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
    at.host_str() == Some(FROM) && at.path().starts_with(&format!("{RELEASES}v{version}/"))
}

/// A candidate is looked for among the candidates, and then among the stable releases — because
/// the release that retires a candidate takes its channel away with it, and a copy still holding
/// that offer would otherwise meet a refusal from the network rather than an answer. The stable
/// channel does not have the version it was promised either, so what it is told is that the offer
/// is gone.
#[must_use]
pub fn feeds_for(version: &str) -> Vec<&'static str> {
    match version.parse::<semver::Version>() {
        Ok(said) if !said.pre.is_empty() => vec![CANDIDATE, LATEST],
        _ => vec![LATEST],
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
    linkunbound_core::update::on_the_candidate_track(now, wants)
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

#[must_use]
pub fn from_the_shop(version: Option<&str>) -> Ready {
    Ready {
        version: version.unwrap_or_default().to_owned(),
        route: Route::Store,
        package: None,
        installs: true,
        coming: None,
    }
}

#[must_use]
pub fn on_its_way(version: Option<&str>, coming: Coming) -> Ready {
    Ready {
        installs: false,
        coming: Some(coming),
        ..from_the_shop(version)
    }
}

#[must_use]
pub fn later(now: &str, found: &str) -> bool {
    linkunbound_core::update::newer_than(found, now)
}

fn offered(version: String, kept: Kept) -> Ready {
    Ready {
        version,
        route: kept.route,
        package: kept.package,
        // A copy running from a mounted image cannot replace itself either, and the refusal in
        // `update_install` reads the same thing: offering a button it will take away is worse
        // than not offering one.
        installs: self_installs(kept.route) && !from_a_mount(),
        coming: None,
    }
}

/// What the last look found, so closing the window does not take the offer away with it. The copy
/// may have moved between a download and a cask since, so where it stands is read again — and so
/// may the track it is on, which is why the rule is applied here too rather than trusted to
/// whatever was true when the answer was written down.
#[must_use]
pub fn remembered(now: &str, said: Option<&str>, kept: Kept, wants: Option<bool>) -> Option<Ready> {
    let said = said?;
    let found: semver::Version = said.parse().ok()?;
    linkunbound_core::update::worth_offering(said, now, wants)
        .then(|| offered(found.to_string(), kept))
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
    cfg!(target_os = "macos") && running.is_some_and(on_a_mount)
}

/// A quarantined copy run straight from Downloads is translocated to a
/// read-only path under `/private/var/folders` that changes with every launch,
/// so a registration made from there dies with the next one.
fn on_a_mount(at: &Path) -> bool {
    at.starts_with("/Volumes/")
        || at.starts_with("/private/tmp/")
        || at
            .components()
            .any(|part| part.as_os_str() == "AppTranslocation")
}

#[must_use]
pub fn from_a_mount() -> bool {
    mounted(std::env::current_exe().ok().as_deref())
}

/// The plugin asks for the chip it was built for, and under Rosetta that is not the machine's.
#[must_use]
pub fn platform(translated: bool) -> Option<&'static str> {
    translated.then_some("darwin-aarch64")
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

fn client() -> Option<reqwest::blocking::Client> {
    reqwest::blocking::Client::builder()
        .timeout(PATIENCE)
        .user_agent(concat!("linkunbound/", env!("CARGO_PKG_VERSION")))
        .build()
        .ok()
}

#[must_use]
pub fn fetch() -> Option<String> {
    let asked = client()?.get(MANIFEST).send().ok()?;

    asked.status().is_success().then(|| asked.text().ok())?
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ahead {
    Unheard,
    Nothing,
    Is(String),
}

fn ours_in_the_catalog(full: &str, arch: &str) -> Option<semver::Version> {
    let mut parts = full.split('_');
    let (name, version, built_for, publisher) = (
        parts.next()?,
        parts.next()?,
        parts.next()?,
        parts.next_back()?,
    );
    if format!("{name}_{publisher}") != STORE_FAMILY || ![arch, "neutral"].contains(&built_for) {
        return None;
    }
    let mut numbers = version.split('.');
    format!(
        "{}.{}.{}",
        numbers.next()?,
        numbers.next()?,
        numbers.next()?
    )
    .parse()
    .ok()
}

/// Microsoft does not document this answer, so one with no package of ours in it says nothing.
#[must_use]
pub fn listed(now: &str, arch: &str, said: &str) -> Ahead {
    let Ok(here) = now.parse::<semver::Version>() else {
        return Ahead::Unheard;
    };
    let Ok(read) = serde_json::from_str::<serde_json::Value>(said) else {
        return Ahead::Unheard;
    };
    let best = read
        .pointer("/Product/DisplaySkuAvailabilities")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|one| one.pointer("/Sku/Properties/Packages")?.as_array())
        .flatten()
        .filter_map(|package| package.get("PackageFullName")?.as_str())
        .filter_map(|full| ours_in_the_catalog(full, arch))
        .max();
    match best {
        None => Ahead::Unheard,
        Some(best) if best > here => Ahead::Is(best.to_string()),
        Some(_) => Ahead::Nothing,
    }
}

#[must_use]
pub fn catalog(now: &str) -> Ahead {
    use std::io::Read;

    let Some(answer) = client().and_then(|asking| asking.get(CATALOG).send().ok()) else {
        return Ahead::Unheard;
    };
    if !answer.status().is_success() {
        return Ahead::Unheard;
    }
    let mut said = String::new();
    if answer
        .take(LISTING_AT_MOST)
        .read_to_string(&mut said)
        .is_err()
    {
        return Ahead::Unheard;
    }
    listed(now, STORE_ARCH, &said)
}

pub fn ahead(
    now: &str,
    wants: Option<bool>,
    catalog: Ahead,
    feed: impl FnOnce() -> Option<String>,
) -> Ahead {
    if catalog != Ahead::Unheard {
        return catalog;
    }
    let Some(said) = feed().filter(|said| serde_json::from_str::<Manifest>(said).is_ok()) else {
        return Ahead::Unheard;
    };
    newer(now, &said, Kept::plain(Route::Store), wants)
        .map_or(Ahead::Nothing, |one| Ahead::Is(one.version))
}

pub use linkunbound_core::update::{Progress, keep, looked, progress, settle, tell, under_way_for};

#[cfg(test)]
mod tests {

    #[test]
    fn a_copy_under_rosetta_asks_for_the_native_build_and_no_other_copy_chooses() {
        assert_eq!(platform(true), Some("darwin-aarch64"));
        assert_eq!(
            platform(false),
            None,
            "the plugin's own architecture stands"
        );
    }

    /// A look that just happened is not a look that is due: counting it as due turns every call
    /// into a fetch, which for a Store copy is a round trip to the shop on every window.
    #[test]
    fn a_look_taken_this_very_second_is_not_due_again() {
        assert!(!due(Some(1_800_000_000), 1_800_000_000));
        assert!(!due(Some(1_800_000_000), 1_800_000_000 + APART - 1));
        assert!(
            due(Some(1_800_000_000), 1_800_000_000 + APART),
            "a day apart to the second is a day apart"
        );
    }

    /// Both places a disk image is opened from. The check is per path, so either one on its own
    /// has to be enough — reading it as "both" would never match anything.
    #[test]
    fn either_place_a_disk_image_opens_from_counts() {
        assert!(on_a_mount(Path::new(
            "/Volumes/LinkUnbound/LinkUnbound.app"
        )));
        assert!(on_a_mount(Path::new(
            "/private/tmp/AppTranslocation/X/LinkUnbound.app"
        )));
        assert!(on_a_mount(Path::new(
            "/private/var/folders/7r/x/T/AppTranslocation/6F1A-2B/d/LinkUnbound.app"
        )));
        assert!(!on_a_mount(Path::new("/Applications/LinkUnbound.app")));
        assert!(!on_a_mount(Path::new(
            "/Users/ana/Applications/LinkUnbound.app"
        )));
        assert!(!on_a_mount(Path::new("/Users/x/Downloads/LinkUnbound.app")));
    }

    /// Seconds since the epoch, which is what everything stored beside it is compared against. A
    /// clock reading zero makes every look overdue for ever.
    #[test]
    fn the_clock_answers_in_seconds_since_the_epoch() {
        let said = now();
        assert!(said > 1_700_000_000, "a clock stuck before 2023: {said}");
        assert!(said < 4_000_000_000, "a clock past 2096: {said}");
    }
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

        let ahead = r#"{"latest":"2.1.0","latestPrerelease":"2.3.0-rc1"}"#;
        assert!(
            newer(
                "2.2.0-rc1",
                ahead,
                Kept::plain(Route::Download),
                Some(false)
            )
            .is_none(),
            "a candidate that asked to leave waits for a stable release that passes it"
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

    #[test]
    fn an_offer_the_store_itself_made_is_one_it_can_install() {
        let found = from_the_shop(Some("2.1.0"));

        assert_eq!(found.version, "2.1.0");
        assert_eq!(found.route, Route::Store);
        assert!(
            found.installs,
            "a store copy cannot install from a download, but it can take what the store offered"
        );
        assert_eq!(found.coming, None);
        assert!(
            !self_installs(Route::Store),
            "which is why the route alone must not decide this"
        );
    }

    #[test]
    fn an_offer_with_no_number_can_still_be_taken() {
        let offer = from_the_shop(None);

        assert_eq!(offer.version, "");
        assert!(
            offer.installs,
            "the Store said it has one, and that is enough"
        );
    }

    #[test]
    fn what_the_store_is_still_bringing_has_no_button() {
        let coming = on_its_way(Some("2.1.0"), Coming::Paused);

        assert_eq!(coming.version, "2.1.0");
        assert_eq!(coming.route, Route::Store);
        assert!(!coming.installs);
        assert_eq!(coming.coming, Some(Coming::Paused));
    }

    fn catalog_naming(packages: &[&str]) -> String {
        let named: Vec<_> = packages
            .iter()
            .map(|full| serde_json::json!({"PackageFullName": full, "PackageFormat": "Msix"}))
            .collect();
        serde_json::json!({"Product": {"DisplaySkuAvailabilities": [
            {"Sku": {"SkuId": "0010", "Properties": {"Packages": named}}},
            {"Sku": {"SkuId": "0011", "Properties": {"Packages": null}}},
        ]}})
        .to_string()
    }

    const OURS_28: &str = "rgdevment.LinkUnbound-BrowserPicker_2.8.0.0_x64__kdjgfdc2rb3gc";
    const OURS_29: &str = "rgdevment.LinkUnbound-BrowserPicker_2.9.0.0_x64__kdjgfdc2rb3gc";

    #[test]
    fn the_catalog_names_the_newest_of_ours_it_lists() {
        let said = catalog_naming(&[OURS_28, OURS_29]);

        assert_eq!(listed("2.8.0", "x64", &said), Ahead::Is("2.9.0".into()));
        assert_eq!(listed("2.9.0", "x64", &said), Ahead::Nothing);
        assert_eq!(listed("3.0.0", "x64", &said), Ahead::Nothing);
    }

    #[test]
    fn a_package_that_is_not_ours_is_not_counted_as_the_catalog_answering() {
        for full in [
            "Another.App_9.0.0.0_x64__kdjgfdc2rb3gc",
            "rgdevment.LinkUnbound-BrowserPicker_9.0.0.0_x64__someoneelse",
            "rgdevment.LinkUnbound-BrowserPicker_9.0_x64__kdjgfdc2rb3gc",
            "rgdevment.LinkUnbound-BrowserPicker",
        ] {
            assert_eq!(
                listed("2.8.0", "x64", &catalog_naming(&[full])),
                Ahead::Unheard,
                "{full}"
            );
        }
    }

    #[test]
    fn a_package_built_for_another_machine_is_not_one_this_copy_can_be_given() {
        let said = catalog_naming(&[
            OURS_29,
            "rgdevment.LinkUnbound-BrowserPicker_9.0.0.0_arm64__kdjgfdc2rb3gc",
        ]);
        assert_eq!(listed("2.8.0", "x64", &said), Ahead::Is("2.9.0".into()));
        assert_eq!(listed("2.8.0", "arm64", &said), Ahead::Is("9.0.0".into()));

        let bundled = catalog_naming(&[
            "rgdevment.LinkUnbound-BrowserPicker_2.9.0.0_neutral_~_kdjgfdc2rb3gc",
        ]);
        assert_eq!(listed("2.8.0", "x64", &bundled), Ahead::Is("2.9.0".into()));
    }

    #[test]
    fn a_catalog_that_stopped_reading_as_it_did_says_nothing_rather_than_nothing_newer() {
        for said in [
            "",
            "not json",
            "{}",
            r#"{"Product":null}"#,
            r#"{"Product":{"DisplaySkuAvailabilities":[]}}"#,
        ] {
            assert_eq!(listed("2.8.0", "x64", said), Ahead::Unheard, "{said}");
        }
        assert_eq!(
            listed("tomorrow", "x64", &catalog_naming(&[OURS_29])),
            Ahead::Unheard
        );
    }

    #[test]
    fn a_catalog_that_answered_is_not_second_guessed_by_the_feed() {
        let no_feed = || panic!("the catalog answered, so nothing should go to GitHub");

        assert_eq!(
            ahead("2.8.0", None, Ahead::Is("2.9.0".into()), no_feed),
            Ahead::Is("2.9.0".into())
        );
        assert_eq!(
            ahead("2.8.0", None, Ahead::Nothing, no_feed),
            Ahead::Nothing
        );
    }

    #[test]
    fn the_feed_stands_in_for_a_catalog_nobody_could_read() {
        let feed = |said: &str| Some(format!(r#"{{"schema":1,"latest":"{said}"}}"#));

        assert_eq!(
            ahead("2.8.0", None, Ahead::Unheard, || feed("2.9.0")),
            Ahead::Is("2.9.0".into())
        );
        assert_eq!(
            ahead("2.9.0", None, Ahead::Unheard, || feed("2.9.0")),
            Ahead::Nothing
        );
        assert_eq!(
            ahead("2.8.0", None, Ahead::Unheard, || None),
            Ahead::Unheard
        );
        assert_eq!(
            ahead("2.8.0", None, Ahead::Unheard, || Some("not json".into())),
            Ahead::Unheard,
            "a feed nobody can read says nothing either"
        );
    }

    #[test]
    fn what_the_catalog_names_is_the_identity_the_release_checks() {
        let workflow = include_str!("../../../.github/workflows/release.yml");
        let (name, _) = STORE_FAMILY.split_once('_').expect("a family");

        assert!(workflow.contains(&format!("$was = \"{name}\"")));
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
        assert!(
            !ours("https://objects.githubusercontent.com/whatever", "2.1.0"),
            "the host GitHub redirects to carries no path of ours to recognise, and the feed              never names it: the plugin is handed what the feed wrote, before any redirect"
        );

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
        assert_eq!(feeds_for("2.1.0"), vec![LATEST]);
        assert_eq!(feeds_for("2.2.0-rc1"), vec![CANDIDATE, LATEST]);
        assert_eq!(
            feeds_for("tomorrow"),
            vec![LATEST],
            "unreadable means stable"
        );
    }

    /// The release that retires a candidate deletes its channel. Asking only there would meet a
    /// refusal from the network; falling through to the stable channel is what turns it into an
    /// answer the person can be told.
    #[test]
    fn a_candidate_that_was_retired_is_told_it_is_gone_rather_than_met_with_a_refusal() {
        let asked = feeds_for("2.2.0-rc1");

        assert_eq!(asked.len(), 2, "the candidate channel may not be there");
        assert_eq!(asked[1], LATEST, "and the stable one answers for it");
    }

    /// The track can change between writing an answer down and reading it back, so the rule is
    /// applied again here rather than trusted to whatever was true at the time.
    #[test]
    fn a_candidate_written_down_before_the_track_changed_is_not_offered_after_it() {
        let kept = Kept::plain(Route::Download);

        assert_eq!(
            remembered("2.1.0", Some("2.2.0-rc1"), kept, Some(true))
                .unwrap()
                .version,
            "2.2.0-rc1",
            "while the copy is on the candidates"
        );
        assert!(
            remembered("2.1.0", Some("2.2.0-rc1"), kept, Some(false)).is_none(),
            "and not once it asked to leave them"
        );
        assert!(
            remembered("2.1.0", Some("2.2.0"), kept, Some(false)).is_some(),
            "a stable offer survives the change"
        );
    }

    #[test]
    fn the_feed_follows_what_is_offered_rather_than_what_is_running() {
        let feed = r#"{"latest":"2.3.0","latestPrerelease":"2.2.0-rc1"}"#;
        let found =
            newer("2.2.0-rc1", feed, Kept::plain(Route::Download), None).expect("2.3.0 is newer");

        assert_eq!(
            feeds_for(&found.version),
            vec![LATEST],
            "a candidate sent to a stable release is pointed at the stable feed alone"
        );
    }

    #[test]
    fn what_was_found_is_still_offered_after_the_window_closed() {
        let kept = Kept::plain(Route::Download);

        assert_eq!(
            remembered("2.0.0", Some("2.1.0"), kept, None)
                .unwrap()
                .version,
            "2.1.0"
        );
        assert!(remembered("2.1.0", Some("2.1.0"), kept, None).is_none());
        assert!(
            remembered("2.2.0", Some("2.1.0"), kept, None).is_none(),
            "a copy updated by hand is not owed the old offer"
        );
        assert!(remembered("2.0.0", None, kept, None).is_none());
        assert!(remembered("2.0.0", Some("tomorrow"), kept, None).is_none());
    }

    /// `latest` sitting behind the running candidate is what a project looks like before its
    /// first stable release.
    #[test]
    fn a_project_with_no_stable_release_yet_still_works() {
        let feed = r#"{"schema":1,"latest":"0.0.0","latestPrerelease":"2.0.0-rc6"}"#;

        assert_eq!(
            newer("2.0.0-rc5", feed, Kept::plain(Route::Download), None)
                .unwrap()
                .version,
            "2.0.0-rc6"
        );
        assert!(newer("1.9.0", feed, Kept::plain(Route::Download), None).is_none());
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
}
