use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct SystemState {
    /// Whether the app is offered by the shell as a browser at all.
    pub registered: bool,
    /// Whether the user actually picked it. Windows owns this and no
    /// application may write it, so it is only ever read.
    pub is_default: bool,
    pub associations: Vec<Association>,
    pub starts_with_system: bool,
    /// False when the startup task was disabled outside the app, which cannot be
    /// undone from here: the toggle has to explain rather than pretend.
    pub startup_is_ours: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct Association {
    pub scheme: String,
    pub held: bool,
}

#[cfg(windows)]
mod platform {
    use super::{Association, SystemState};
    use linkunbound_win::{
        Registration, association_report, installed_browsers, is_default_browser,
        notify_associations_changed, set_startup, startup_state,
    };

    fn own_path() -> Option<String> {
        Some(std::env::current_exe().ok()?.to_string_lossy().into_owned())
    }

    /// Reconciles on every launch, the way 1.x did: an update moves the
    /// executable and the keys keep pointing at a path that no longer exists.
    pub fn reconcile() {
        let Some(exe) = own_path() else { return };
        let registration = Registration::default();
        if registration.register(&exe).is_ok() {
            notify_associations_changed();
        }
    }

    pub fn state() -> SystemState {
        let startup = startup_state();
        SystemState {
            registered: Registration::default().is_registered(),
            is_default: is_default_browser(),
            associations: association_report()
                .into_iter()
                .map(|(scheme, held)| Association { scheme, held })
                .collect(),
            starts_with_system: startup.is_some_and(|s| s.enabled),
            startup_is_ours: startup.is_none_or(|s| s.ours_to_change),
        }
    }

    pub fn set_registered(enabled: bool) -> Result<SystemState, String> {
        let registration = Registration::default();
        let outcome = if enabled {
            let exe = own_path().ok_or_else(|| "cannot find our own path".to_owned())?;
            registration.register(&exe)
        } else {
            registration.unregister()
        };
        outcome.map_err(|e| e.to_string())?;
        notify_associations_changed();
        Ok(state())
    }

    pub fn set_starts_with_system(enabled: bool) -> Result<SystemState, String> {
        set_startup(enabled).ok_or_else(|| "no startup task in this build".to_owned())?;
        Ok(state())
    }

    pub fn browsers() -> Vec<linkunbound_core::Browser> {
        installed_browsers()
    }
}

#[cfg(not(windows))]
mod platform {
    use super::SystemState;

    pub fn reconcile() {}

    pub fn state() -> SystemState {
        SystemState {
            registered: false,
            is_default: false,
            associations: Vec::new(),
            starts_with_system: false,
            startup_is_ours: true,
        }
    }

    pub fn set_registered(_enabled: bool) -> Result<SystemState, String> {
        Err("not supported on this platform yet".to_owned())
    }

    pub fn set_starts_with_system(_enabled: bool) -> Result<SystemState, String> {
        Err("not supported on this platform yet".to_owned())
    }

    pub fn browsers() -> Vec<linkunbound_core::Browser> {
        Vec::new()
    }
}

pub use platform::{browsers, reconcile, set_registered, set_starts_with_system, state};
