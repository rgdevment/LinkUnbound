#![cfg_attr(windows, allow(unsafe_code))]

/// A shop that answers it has nothing is not a shop that never answered: the first means this copy
/// is current, the second that nothing is known and the old wording still stands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Shelf {
    Waiting(String),
    Current,
    Silent,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Trouble {
    Gone,
    Stopped,
    Failed(String),
}

/// The Store counts the download as the first four fifths of the errand and the install as the
/// rest, so what it hands over is stretched back into a download of its own.
const DOWNLOADED: f64 = 0.8;

#[must_use]
pub fn step(far: f64) -> (&'static str, u64) {
    if far >= DOWNLOADED {
        return ("installing", 100);
    }
    ("getting", (far.max(0.0) / DOWNLOADED * 100.0) as u64)
}

#[must_use]
pub fn asked(window: isize) -> Shelf {
    there::asked(window)
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
    use windows::ApplicationModel::{Package, PackageVersion};
    use windows::Services::Store::{
        StoreContext, StorePackageUpdateState, StorePackageUpdateStatus,
    };
    use windows::Win32::Foundation::HWND;
    use windows::Win32::System::Com::CoIncrementMTAUsage;
    use windows::Win32::UI::Shell::IInitializeWithWindow;
    use windows::core::{Interface, Ref};
    use windows_future::AsyncOperationProgressHandler;

    const PATIENCE: std::time::Duration = std::time::Duration::from_secs(10);
    /// Long past the point where the process should already have been taken down by the install.
    const AT_LENGTH: std::time::Duration = std::time::Duration::from_secs(30 * 60);

    /// The Store answers on a thread of its own and has been known to never answer at all, which
    /// would otherwise leave the window waiting on it for the rest of the session.
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

    pub fn asked(window: isize) -> Shelf {
        match apart(PATIENCE, move || waiting(window)) {
            Some(Ok(Some(version))) => Shelf::Waiting(version),
            Some(Ok(None)) => Shelf::Current,
            Some(Err(_)) | None => Shelf::Silent,
        }
    }

    /// The errand covers optional packages too, and the version of one of those is not a version
    /// of LinkUnbound to put in front of anybody.
    fn waiting(window: isize) -> windows::core::Result<Option<String>> {
        let updates = shop(window)?
            .GetAppAndOptionalStorePackageUpdatesAsync()?
            .get()?;
        let ours = Package::Current()?.Id()?.FamilyName()?;
        for one in 0..updates.Size()? {
            let id = updates.GetAt(one)?.Package()?.Id()?;
            if id.FamilyName()? == ours {
                return Ok(Some(numbered(&id.Version()?)));
            }
        }
        Ok(None)
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
        mut told: impl FnMut(&'static str, u64) + Send + 'static,
    ) -> Result<(), Trouble> {
        let shop = shop(window).map_err(sour)?;
        let updates = shop
            .GetAppAndOptionalStorePackageUpdatesAsync()
            .and_then(|asking| asking.get())
            .map_err(sour)?;
        if updates.Size().map_err(sour)? == 0 {
            return Err(Trouble::Gone);
        }

        let asking = shop
            .RequestDownloadAndInstallStorePackageUpdatesAsync(&updates)
            .map_err(sour)?;
        asking
            .SetProgress(&AsyncOperationProgressHandler::new(
                move |_, far: Ref<'_, StorePackageUpdateStatus>| {
                    let (stage, how) = super::step(far.ok()?.PackageDownloadProgress);
                    told(stage, how);
                    Ok(())
                },
            ))
            .map_err(sour)?;

        // Windows takes the process with it to put the new package in place, so on the ordinary
        // path nothing below this line is ever reached.
        let done = asking.get().map_err(sour)?;
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
    fn owned(shop: &StoreContext, window: isize) -> windows::core::Result<()> {
        let owner: IInitializeWithWindow = shop.cast()?;
        unsafe { owner.Initialize(HWND(window as *mut core::ffi::c_void)) }
    }

    /// A worker thread of the async runtime belongs to no apartment, and the activation fails there
    /// before it begins.
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

    pub fn asked(_window: isize) -> Shelf {
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
    use super::step;

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
}
