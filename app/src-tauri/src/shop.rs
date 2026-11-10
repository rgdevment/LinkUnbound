#![cfg_attr(not(windows), allow(dead_code))]

use crate::HERE;
use crate::update::{self, Ahead, Coming, Ready, Route};
use linkunbound_core::update::{Looked, ShopSaid};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Shelf {
    Waiting,
    Landed(String),
    Queued,
    Paused,
    Current,
    Silent,
    NotSold,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Memory {
    pub version: Option<String>,
    pub said: Option<ShopSaid>,
}

impl Memory {
    #[must_use]
    pub fn of(looked: &Looked) -> Self {
        let kept = looked.found_route.as_deref() == Some(Route::Store.name());
        Self {
            version: looked.found_version.clone(),
            said: looked.found_in_the_shop.filter(|_| kept),
        }
    }

    pub fn written_into(self, looked: &mut Looked) {
        looked.found_route = self.said.map(|_| Route::Store.name().to_owned());
        looked.found_installs = self.said.map(|said| said == ShopSaid::Offered);
        looked.found_in_the_shop = self.said;
        looked.found_version = self.version;
    }

    const fn offered(&self) -> bool {
        matches!(self.said, Some(ShopSaid::Offered | ShopSaid::Landed))
    }

    const fn coming(&self) -> bool {
        matches!(
            self.said,
            Some(ShopSaid::Queued | ShopSaid::Paused | ShopSaid::NotYet)
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Verdict {
    pub ready: Option<Ready>,
    pub memory: Memory,
    pub unanswered: bool,
}

impl Verdict {
    /// A look nobody pressed for has nobody to tell that the Store did not answer.
    pub fn settled_into(
        self,
        looked: &mut Looked,
        pressed: bool,
    ) -> Result<Option<Ready>, &'static str> {
        self.memory.written_into(looked);
        if pressed && self.unanswered {
            return Err("updateUnanswered");
        }
        Ok(self.ready)
    }
}

#[must_use]
pub fn waits(pressed: bool, ahead: &Ahead) -> bool {
    pressed && *ahead != Ahead::Nothing
}

fn told(said: ShopSaid, version: Option<&str>) -> Ready {
    match said {
        ShopSaid::Offered => update::from_the_shop(version),
        ShopSaid::Landed => Ready {
            installs: false,
            ..update::from_the_shop(version)
        },
        ShopSaid::Queued => update::on_its_way(version, Coming::Queued),
        ShopSaid::Paused => update::on_its_way(version, Coming::Paused),
        ShopSaid::NotYet => update::on_its_way(version, Coming::NotYet),
    }
}

#[must_use]
pub fn last_said(now: &str, was: &Memory) -> Option<Ready> {
    let version = was
        .version
        .as_deref()
        .filter(|one| update::later(now, one))?;
    Some(told(was.said?, Some(version)))
}

#[must_use]
pub fn verdict(shelf: Shelf, ahead: Ahead, now: &str, was: Memory) -> Verdict {
    let number = match &ahead {
        Ahead::Is(version) => Some(version.clone()),
        Ahead::Nothing => None,
        Ahead::Unheard => was.version.clone().filter(|one| update::later(now, one)),
    };
    let tell = |said: ShopSaid, version: Option<String>| Verdict {
        ready: Some(told(said, version.as_deref())),
        memory: Memory {
            version,
            said: Some(said),
        },
        unanswered: false,
    };
    let none = |unanswered: bool| Verdict {
        ready: None,
        memory: Memory::default(),
        unanswered,
    };
    match (shelf, ahead) {
        (Shelf::Waiting, _) => tell(ShopSaid::Offered, number),
        (Shelf::Landed(version), _) => tell(ShopSaid::Landed, Some(version)),
        (Shelf::Queued, _) => tell(ShopSaid::Queued, number),
        (Shelf::Paused, _) => tell(ShopSaid::Paused, number),
        (Shelf::NotSold, _) => none(false),
        (_, Ahead::Is(version)) => tell(ShopSaid::NotYet, Some(version)),
        (Shelf::Current, Ahead::Nothing) => none(false),
        (Shelf::Silent, Ahead::Nothing) if !was.offered() => none(true),
        (Shelf::Current, Ahead::Unheard) if !was.coming() => none(false),
        (shelf, _) => Verdict {
            ready: last_said(now, &was),
            memory: was,
            unanswered: shelf == Shelf::Silent,
        },
    }
}

/// The Store never carries candidates, so only the stable track can stand in for its catalog.
fn coming_from_the_store() -> Ahead {
    update::ahead(HERE, Some(false), update::catalog(HERE), update::fetch)
}

#[must_use]
pub fn look(window: isize, pressed: bool, was: Memory) -> Verdict {
    let (shelf, ahead) = match settled() {
        Some(shelf) => (shelf, Ahead::Unheard),
        None if pressed => {
            let ahead = coming_from_the_store();
            (asked(window, waits(pressed, &ahead)), ahead)
        }
        None => std::thread::scope(|both| {
            let ahead = both.spawn(coming_from_the_store);
            let shelf = asked(window, false);
            (shelf, ahead.join().unwrap_or(Ahead::Unheard))
        }),
    };
    verdict(shelf, ahead, HERE, was)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Trouble {
    Gone,
    Stopped,
    Failed(String),
}

/// The Store counts the download as the first four fifths of the errand and the install as the rest.
const DOWNLOADED: f64 = 0.8;

#[must_use]
pub fn step(far: f64) -> (&'static str, u64) {
    if far >= DOWNLOADED {
        return ("installing", 100);
    }
    ("getting", (far.max(0.0) / DOWNLOADED * 100.0) as u64)
}

#[must_use]
pub fn settled() -> Option<Shelf> {
    there::settled()
}

#[must_use]
pub fn sold() -> bool {
    there::sold_here()
}

#[must_use]
pub fn asked(window: isize, waits: bool) -> Shelf {
    there::asked(window, waits)
}

pub fn take(
    window: isize,
    told: impl FnMut(&'static str, u64) + Send + 'static,
) -> Result<(), Trouble> {
    there::take(window, told)
}

#[cfg(windows)]
mod there {
    use super::{Shelf, Trouble};
    use windows::ApplicationModel::{Package, PackageSignatureKind, PackageVersion};
    use windows::Management::Deployment::PackageManager;
    use windows::Services::Store::{
        StoreContext, StorePackageUpdateState, StorePackageUpdateStatus,
        StoreQueueItemExtendedState, StoreQueueItemState,
    };
    use windows::Win32::Foundation::HWND;
    use windows::Win32::System::Com::CoIncrementMTAUsage;
    use windows::Win32::UI::Shell::IInitializeWithWindow;
    use windows::core::{HSTRING, Interface, Ref};
    use windows_future::AsyncOperationProgressHandler;

    const PATIENCE: std::time::Duration = std::time::Duration::from_secs(10);
    const AGAIN: std::time::Duration = std::time::Duration::from_secs(6);
    const UNTIL: std::time::Duration = std::time::Duration::from_secs(75);
    const AT_LENGTH: std::time::Duration = std::time::Duration::from_secs(30 * 60);

    /// The Store has been known to never answer, which would leave the window waiting for good.
    fn apart<T: Send + 'static>(
        how_long: std::time::Duration,
        work: impl FnOnce() -> T + Send + 'static,
    ) -> Option<T> {
        let (tell, hear) = std::sync::mpsc::sync_channel(1);
        std::thread::spawn(move || {
            let _ = tell.send(work());
        });
        hear.recv_timeout(how_long).ok()
    }

    pub fn sold_here() -> bool {
        Package::Current()
            .and_then(|one| one.SignatureKind())
            .is_ok_and(|kind| kind == PackageSignatureKind::Store)
    }

    pub fn settled() -> Option<Shelf> {
        if !sold_here() {
            return Some(Shelf::NotSold);
        }
        beside_us().map(Shelf::Landed)
    }

    pub fn asked(window: isize, waits: bool) -> Shelf {
        match offers(window) {
            Some(true) => Shelf::Waiting,
            Some(false) if waits => waited_out(window),
            Some(false) => in_its_queue(window),
            None => Shelf::Silent,
        }
    }

    fn offers(window: isize) -> Option<bool> {
        apart(PATIENCE, move || waiting(window)).and_then(Result::ok)
    }

    /// Asking wakes the Store's own errand and it answers with the state from before, so a press
    /// waits on what that errand leaves behind, watching what costs no rationed look.
    fn waited_out(window: isize) -> Shelf {
        let since = std::time::Instant::now();
        while since.elapsed() < UNTIL {
            std::thread::sleep(AGAIN);
            if let Some(version) = beside_us() {
                return Shelf::Landed(version);
            }
            if let Some(Ok(Some(found))) = apart(PATIENCE, move || queued(window)) {
                return found;
            }
        }
        match offers(window) {
            Some(true) => Shelf::Waiting,
            Some(false) => in_its_queue(window),
            None => Shelf::Silent,
        }
    }

    fn in_its_queue(window: isize) -> Shelf {
        match apart(PATIENCE, move || queued(window)) {
            Some(Ok(Some(found))) => found,
            Some(Ok(None)) => Shelf::Current,
            Some(Err(_)) | None => Shelf::Silent,
        }
    }

    fn queued(window: isize) -> windows::core::Result<Option<Shelf>> {
        let ours = Package::Current()?.Id()?.FamilyName()?;
        let items = shop(window)?.GetAssociatedStoreQueueItemsAsync()?.join()?;
        for one in 0..items.Size()? {
            let item = items.GetAt(one)?;
            if item.PackageFamilyName().unwrap_or_default() != ours {
                continue;
            }
            let status = item.GetCurrentStatus()?;
            let state = status.PackageInstallState()?;
            if matches!(
                state,
                StoreQueueItemState::Active | StoreQueueItemState::Paused
            ) {
                let reason = status.PackageInstallExtendedState().unwrap_or_default();
                return Ok(Some(of_the_queue(state, reason)));
            }
        }
        Ok(None)
    }

    pub(super) fn of_the_queue(
        state: StoreQueueItemState,
        reason: StoreQueueItemExtendedState,
    ) -> Shelf {
        let waits_for_us = matches!(
            reason,
            StoreQueueItemExtendedState::PausedPackagesInUse
                | StoreQueueItemExtendedState::PausedReadyToInstall
        );
        if state == StoreQueueItemState::Paused && !waits_for_us {
            Shelf::Paused
        } else {
            Shelf::Queued
        }
    }

    fn beside_us() -> Option<String> {
        let ours = Package::Current().ok()?;
        let family = ours.Id().ok()?.FamilyName().ok()?;
        let mine: semver::Version = numbered(&ours.Id().ok()?.Version().ok()?).parse().ok()?;
        let this_user = HSTRING::new();
        PackageManager::new()
            .ok()?
            .FindPackagesByUserSecurityIdPackageFamilyName(&this_user, &family)
            .ok()?
            .into_iter()
            .filter_map(|one| numbered(&one.Id().ok()?.Version().ok()?).parse().ok())
            .filter(|said: &semver::Version| *said > mine)
            .max()
            .map(|newest| newest.to_string())
    }

    /// The package an offer names is the one installed here, so its number says nothing of what comes.
    fn waiting(window: isize) -> windows::core::Result<bool> {
        let updates = shop(window)?
            .GetAppAndOptionalStorePackageUpdatesAsync()?
            .join()?;
        let ours = Package::Current()?.Id()?.FamilyName()?;
        for one in 0..updates.Size()? {
            if updates.GetAt(one)?.Package()?.Id()?.FamilyName()? == ours {
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub fn take(
        window: isize,
        told: impl FnMut(&'static str, u64) + Send + 'static,
    ) -> Result<(), Trouble> {
        apart(AT_LENGTH, move || taken(window, told)).unwrap_or_else(|| {
            Err(Trouble::Failed(format!(
                "the Store was left to it for {} minutes and never came back",
                AT_LENGTH.as_secs() / 60
            )))
        })
    }

    fn taken(
        window: isize,
        told: impl FnMut(&'static str, u64) + Send + 'static,
    ) -> Result<(), Trouble> {
        let shop = shop(window).map_err(sour)?;
        let updates = shop
            .GetAppAndOptionalStorePackageUpdatesAsync()
            .and_then(|asking| asking.join())
            .map_err(sour)?;
        if updates.Size().map_err(sour)? == 0 {
            return Err(Trouble::Gone);
        }

        let asking = shop
            .RequestDownloadAndInstallStorePackageUpdatesAsync(&updates)
            .map_err(sour)?;
        let told = std::sync::Mutex::new(told);
        asking
            .SetProgress(&AsyncOperationProgressHandler::new(
                move |_, far: Ref<'_, StorePackageUpdateStatus>| {
                    let (stage, how) = super::step(far.ok()?.PackageDownloadProgress);
                    if let Ok(mut told) = told.lock() {
                        told(stage, how);
                    }
                    Ok(())
                },
            ))
            .map_err(sour)?;

        // Windows takes the process with it to put the new package in place, so on the ordinary
        // path nothing below this line is ever reached.
        let done = asking.join().map_err(sour)?;
        match done.OverallState().map_err(sour)? {
            StorePackageUpdateState::Completed => Ok(()),
            StorePackageUpdateState::Canceled => Err(Trouble::Stopped),
            other => Err(Trouble::Failed(format!("{other:?}"))),
        }
    }

    fn shop(window: isize) -> windows::core::Result<StoreContext> {
        apartment();
        let shop = StoreContext::GetDefault()?;
        owned(&shop, window)?;
        Ok(shop)
    }

    /// The Store raises dialogs of its own and refuses with ERROR_INVALID_WINDOW_HANDLE unless it
    /// is told which window owns them.
    #[allow(unsafe_code)]
    fn owned(shop: &StoreContext, window: isize) -> windows::core::Result<()> {
        let owner: IInitializeWithWindow = shop.cast()?;
        unsafe { owner.Initialize(HWND(window as *mut core::ffi::c_void)) }
    }

    /// A worker thread of the async runtime belongs to no apartment, and the activation fails there
    /// before it begins.
    #[allow(unsafe_code)]
    fn apartment() {
        static ONCE: std::sync::OnceLock<()> = std::sync::OnceLock::new();
        ONCE.get_or_init(|| unsafe {
            let _ = CoIncrementMTAUsage();
        });
    }

    /// A package version has four parts and a release has three: the fourth is the store's own
    /// rebuild count and would read as a version nobody ever published.
    fn numbered(version: &PackageVersion) -> String {
        format!("{}.{}.{}", version.Major, version.Minor, version.Build)
    }

    fn sour(why: windows::core::Error) -> Trouble {
        Trouble::Failed(why.message())
    }
}

#[cfg(not(windows))]
mod there {
    use super::{Shelf, Trouble};

    pub const fn settled() -> Option<Shelf> {
        None
    }

    pub const fn sold_here() -> bool {
        false
    }

    pub const fn asked(_window: isize, _waits: bool) -> Shelf {
        Shelf::Silent
    }

    pub fn take(
        _window: isize,
        _told: impl FnMut(&'static str, u64) + Send + 'static,
    ) -> Result<(), Trouble> {
        Err(Trouble::Gone)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const NOW: &str = "2.8.0";

    fn said_before(said: ShopSaid, version: Option<&str>) -> Memory {
        Memory {
            version: version.map(str::to_owned),
            said: Some(said),
        }
    }

    fn offered_before(version: Option<&str>) -> Memory {
        said_before(ShopSaid::Offered, version)
    }

    fn coming_before(version: &str) -> Memory {
        said_before(ShopSaid::NotYet, Some(version))
    }

    fn known(version: &str) -> Ahead {
        Ahead::Is(version.to_owned())
    }

    #[test]
    fn the_download_fills_the_bar_on_its_own() {
        assert_eq!(step(0.0), ("getting", 0));
        assert_eq!(step(0.4), ("getting", 50));
        assert_eq!(step(0.79), ("getting", 98));
    }

    #[test]
    fn what_the_store_counts_as_the_install_is_shown_as_installing() {
        assert_eq!(step(0.8), ("installing", 100));
        assert_eq!(step(0.9), ("installing", 100));
        assert_eq!(step(1.0), ("installing", 100));
    }

    #[test]
    fn a_figure_that_makes_no_sense_still_leaves_a_bar_that_does() {
        assert_eq!(step(-1.0), ("getting", 0));
        assert_eq!(step(2.0), ("installing", 100));
    }

    #[test]
    fn the_store_s_offer_can_be_taken_whatever_is_known_of_its_number() {
        let named = verdict(Shelf::Waiting, known("2.9.0"), NOW, Memory::default());
        let ready = named.ready.expect("an offer");
        assert!(ready.installs);
        assert_eq!(ready.version, "2.9.0");
        assert_eq!(named.memory, offered_before(Some("2.9.0")));

        for ahead in [Ahead::Nothing, Ahead::Unheard] {
            let ready = verdict(Shelf::Waiting, ahead, NOW, Memory::default()).ready;
            let ready = ready.expect("an offer with no number is still an offer");
            assert!(ready.installs);
            assert_eq!(ready.version, "");
        }
    }

    #[test]
    fn with_nobody_able_to_say_the_version_known_stays_while_it_is_still_ahead() {
        let ahead_of_us = verdict(Shelf::Waiting, Ahead::Unheard, NOW, coming_before("2.9.0"));
        assert_eq!(ahead_of_us.ready.expect("an offer").version, "2.9.0");

        let behind_us = verdict(Shelf::Waiting, Ahead::Unheard, NOW, coming_before("2.8.0"));
        assert_eq!(behind_us.ready.expect("an offer").version, "");
    }

    #[test]
    fn what_already_landed_offers_nothing_to_press() {
        let landed = verdict(
            Shelf::Landed("2.9.0".into()),
            Ahead::Nothing,
            NOW,
            Memory::default(),
        );
        let ready = landed.ready.expect("it is here");
        assert!(!ready.installs);
        assert_eq!(ready.version, "2.9.0");
        assert_eq!(ready.coming, None);
    }

    #[test]
    fn a_queue_held_up_is_told_apart_from_one_that_finishes_by_itself() {
        let moving = verdict(Shelf::Queued, known("2.9.0"), NOW, Memory::default());
        assert_eq!(
            moving.ready.expect("on its way").coming,
            Some(Coming::Queued)
        );

        let paused = verdict(Shelf::Paused, known("2.9.0"), NOW, Memory::default());
        let paused = paused.ready.expect("paused there");
        assert_eq!(paused.coming, Some(Coming::Paused));
        assert!(!paused.installs, "only the Store can start it again");
    }

    #[test]
    fn a_copy_the_store_did_not_sell_is_never_told_the_store_brings_anything() {
        let said = verdict(
            Shelf::NotSold,
            known("2.9.0"),
            NOW,
            offered_before(Some("2.9.0")),
        );
        assert_eq!(said.ready, None);
        assert_eq!(said.memory, Memory::default());
        assert!(!said.unanswered, "the Store was never asked");
    }

    #[test]
    fn a_version_the_catalog_lists_is_on_its_way_even_when_the_store_says_nothing() {
        for shelf in [Shelf::Current, Shelf::Silent] {
            let said = verdict(shelf, known("2.9.0"), NOW, Memory::default());
            let ready = said.ready.expect("on its way");
            assert_eq!(ready.coming, Some(Coming::NotYet));
            assert!(!ready.installs);
            assert_eq!(said.memory, coming_before("2.9.0"));
            assert!(!said.unanswered);
        }
    }

    #[test]
    fn a_store_that_answered_and_a_catalog_with_nothing_newer_end_the_matter() {
        let said = verdict(
            Shelf::Current,
            Ahead::Nothing,
            NOW,
            offered_before(Some("2.9.0")),
        );
        assert_eq!(said.ready, None);
        assert_eq!(said.memory, Memory::default());
        assert!(!said.unanswered);
    }

    #[test]
    fn a_store_that_did_not_answer_is_reported_even_when_the_catalog_lists_nothing_newer() {
        let offer = verdict(
            Shelf::Silent,
            Ahead::Nothing,
            NOW,
            offered_before(Some("2.9.0")),
        );
        assert!(offer.unanswered);
        assert!(offer.ready.expect("its own offer stands").installs);
        assert_eq!(offer.memory, offered_before(Some("2.9.0")));

        let coming = verdict(Shelf::Silent, Ahead::Nothing, NOW, coming_before("2.9.0"));
        assert!(coming.unanswered);
        assert_eq!(
            coming.ready, None,
            "the catalog says no such version is out"
        );
        assert_eq!(coming.memory, Memory::default());
    }

    #[test]
    fn the_store_saying_it_has_no_offer_takes_back_the_offer_it_made() {
        let said = verdict(
            Shelf::Current,
            Ahead::Unheard,
            NOW,
            offered_before(Some("2.9.0")),
        );
        assert_eq!(said.ready, None);
        assert_eq!(said.memory, Memory::default());

        let coming = verdict(Shelf::Current, Ahead::Unheard, NOW, coming_before("2.9.0"));
        assert!(coming.ready.expect("still on its way").coming.is_some());
        assert_eq!(coming.memory, coming_before("2.9.0"));
    }

    #[test]
    fn a_store_that_did_not_answer_leaves_what_it_last_said_standing() {
        let said = verdict(
            Shelf::Silent,
            Ahead::Unheard,
            NOW,
            offered_before(Some("2.9.0")),
        );
        assert!(said.ready.expect("still standing").installs);
        assert_eq!(said.memory, offered_before(Some("2.9.0")));
        assert!(said.unanswered);
    }

    #[test]
    fn what_was_last_said_ends_when_this_copy_catches_up_or_had_no_number() {
        assert_eq!(last_said(NOW, &offered_before(Some("2.8.0"))), None);
        assert_eq!(last_said(NOW, &coming_before("2.8.0")), None);
        assert_eq!(last_said(NOW, &offered_before(None)), None);
        assert_eq!(last_said(NOW, &Memory::default()), None);
        let offer = last_said(NOW, &offered_before(Some("2.9.0")));
        assert!(offer.expect("an offer").installs);
        let coming = last_said(NOW, &coming_before("2.9.0"));
        assert!(coming.expect("on its way").coming.is_some());
    }

    #[test]
    fn only_a_press_is_waited_for_and_only_when_there_may_be_something() {
        assert!(waits(true, &known("2.9.0")));
        assert!(waits(true, &Ahead::Unheard));
        assert!(!waits(true, &Ahead::Nothing));
        assert!(!waits(false, &known("2.9.0")));
        assert!(!waits(false, &Ahead::Unheard));
    }

    #[test]
    fn only_a_press_is_told_the_store_did_not_answer() {
        let settled = |shelf: Shelf, pressed: bool| {
            let mut looked = Looked::default();
            verdict(shelf, Ahead::Unheard, NOW, offered_before(Some("2.9.0")))
                .settled_into(&mut looked, pressed)
                .map(|ready| (ready.is_some(), looked.found_in_the_shop))
        };

        assert_eq!(settled(Shelf::Silent, true), Err("updateUnanswered"));
        assert_eq!(
            settled(Shelf::Silent, false),
            Ok((true, Some(ShopSaid::Offered))),
            "unpressed, the last offer simply stands"
        );
        assert_eq!(settled(Shelf::Current, true), Ok((false, None)));
    }

    #[test]
    fn the_file_the_resident_reads_keeps_what_the_store_said() {
        for memory in [
            offered_before(Some("2.9.0")),
            coming_before("2.9.0"),
            offered_before(None),
            Memory::default(),
        ] {
            let mut looked = Looked {
                candidates: Some(true),
                ..Looked::default()
            };
            memory.clone().written_into(&mut looked);
            assert_eq!(Memory::of(&looked), memory);
            assert_eq!(
                looked.candidates,
                Some(true),
                "the track is not the Store's"
            );
        }

        let written = |memory: Memory| {
            let mut looked = Looked::default();
            memory.written_into(&mut looked);
            looked
        };
        let offer = written(offered_before(Some("2.9.0")));
        assert_eq!(offer.found_route.as_deref(), Some("store"));
        assert_eq!(offer.found_installs, Some(true));
        assert_eq!(offer.found_in_the_shop, Some(ShopSaid::Offered));

        for said in [
            ShopSaid::Landed,
            ShopSaid::Queued,
            ShopSaid::Paused,
            ShopSaid::NotYet,
        ] {
            let waiting = written(said_before(said, Some("2.9.0")));
            assert_eq!(waiting.found_installs, Some(false), "{said:?}");
            assert_eq!(waiting.found_in_the_shop, Some(said));
        }
    }

    #[test]
    fn what_landed_and_is_remembered_still_offers_nothing_to_press() {
        let landed = said_before(ShopSaid::Landed, Some("2.9.0"));
        let replayed = last_said(NOW, &landed).expect("still here");
        assert!(!replayed.installs);
        assert_eq!(replayed.coming, None);

        let silent = verdict(Shelf::Silent, Ahead::Nothing, NOW, landed);
        assert!(!silent.ready.expect("still here").installs);
    }

    #[test]
    fn a_queue_remembered_keeps_why_it_waits() {
        let paused = said_before(ShopSaid::Paused, Some("2.9.0"));
        assert_eq!(
            last_said(NOW, &paused).expect("still paused").coming,
            Some(Coming::Paused)
        );
    }

    #[test]
    fn a_download_s_offer_written_down_is_not_taken_for_the_store_s() {
        let looked = Looked {
            found_version: Some("2.9.0".into()),
            found_route: Some("download".into()),
            found_installs: Some(true),
            ..Looked::default()
        };
        assert_eq!(Memory::of(&looked).said, None);
    }

    #[cfg(windows)]
    #[test]
    fn only_what_quitting_linkunbound_will_not_clear_counts_as_paused() {
        use windows::Services::Store::{
            StoreQueueItemExtendedState as Reason, StoreQueueItemState as State,
        };

        for reason in [
            Reason::Paused,
            Reason::PausedLowBattery,
            Reason::PausedWiFiRequired,
        ] {
            assert_eq!(there::of_the_queue(State::Paused, reason), Shelf::Paused);
        }
        for reason in [Reason::PausedPackagesInUse, Reason::PausedReadyToInstall] {
            assert_eq!(there::of_the_queue(State::Paused, reason), Shelf::Queued);
        }
        assert_eq!(
            there::of_the_queue(State::Active, Reason::ActiveDownloading),
            Shelf::Queued
        );
    }
}
