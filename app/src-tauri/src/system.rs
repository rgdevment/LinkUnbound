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
    pub health: Health,
    /// Present when the machine has Edge, which is what makes Teams and Outlook
    /// bypass the default browser.
    pub edge_installed: bool,
}

/// Why the app might not be receiving links even though it looks registered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Health {
    Fine,
    /// The registration points somewhere this executable no longer lives.
    Stale,
    /// Running from a build tree, which cannot own the registration at all.
    BuildTree,
    NotRegistered,
}

#[derive(Debug, Clone, Serialize)]
pub struct Association {
    pub scheme: String,
    pub held: bool,
}

#[cfg(windows)]
mod platform {
    use super::Health;
    use super::{Association, SystemState};
    use linkunbound_win::{
        Registration, association_report, installed_browsers, is_build_tree, is_default_browser,
        notify_associations_changed, set_startup, startup_state,
    };

    /// Looking registered is not the same as working: the command can point at a
    /// path this executable no longer occupies, and nothing else would say so.
    fn health(registration: &Registration) -> Health {
        let Some(exe) = own_path() else {
            return Health::NotRegistered;
        };
        if is_build_tree(&exe) {
            return Health::BuildTree;
        }
        if registration.is_registered_as(&exe) {
            return Health::Fine;
        }
        if registration.is_registered() {
            return Health::Stale;
        }
        Health::NotRegistered
    }

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
        let registration = Registration::default();
        SystemState {
            health: health(&registration),
            edge_installed: installed_browsers().iter().any(|b| b.id.contains("edge")),
            registered: registration.is_registered(),
            is_default: is_default_browser(),
            associations: association_report()
                .into_iter()
                .map(|(scheme, held)| Association { scheme, held })
                .collect(),
            starts_with_system: startup.is_some_and(|s| s.enabled),
            startup_is_ours: startup.is_none_or(|s| s.ours_to_change),
        }
    }

    /// Only Windows may pick the default, so the most we can do is open the very
    /// panel where the user picks it.
    pub fn open_default_apps() -> Result<(), String> {
        std::process::Command::new("cmd")
            .args(["/C", "start", "", "ms-settings:defaultapps"])
            .spawn()
            .map(|_| ())
            .map_err(|e| e.to_string())
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
    use super::{Health, SystemState};

    pub fn reconcile() {}

    pub fn state() -> SystemState {
        SystemState {
            registered: false,
            is_default: false,
            associations: Vec::new(),
            starts_with_system: false,
            startup_is_ours: true,
            health: super::Health::NotRegistered,
            edge_installed: false,
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

    pub fn open_default_apps() -> Result<(), String> {
        Err("only Windows has a default apps panel".to_owned())
    }
}

pub use platform::{
    browsers, open_default_apps, reconcile, set_registered, set_starts_with_system, state,
};
