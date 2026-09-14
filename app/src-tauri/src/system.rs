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
    /// What the shell would actually run. Shown as-is so a registration left by
    /// a build tree or another install names itself instead of hiding behind a
    /// verdict.
    pub registered_path: Option<String>,
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
    /// Registered to our own settings window, which cannot open a link.
    WrongBinary,
    /// The command names the resident, and the resident is not there. A package that shipped
    /// without it reads as healthy otherwise, while every link runs nothing.
    NoResident,
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
    use linkunbound_core::{Registered, link_handler, what_is_registered};
    use linkunbound_win::{
        Registration, association_report, installed_browsers, is_build_tree, is_default_browser,
        notify_associations_changed, set_startup, startup_state,
    };
    use std::path::PathBuf;

    /// Looking registered is not the same as working: the command can point at a
    /// path this executable no longer occupies, and nothing else would say so.
    fn health(registration: &Registration) -> Health {
        let Some(handler) = handler() else {
            return Health::NotRegistered;
        };
        verdict(
            registration.registered_command().as_deref(),
            &handler,
            handler.exists(),
        )
    }

    /// Whether the resident is there is handed in rather than read here, so the verdict stays a
    /// function of its arguments and every branch can be put in front of it.
    fn verdict(command: Option<&str>, handler: &std::path::Path, present: bool) -> Health {
        if is_build_tree(&handler.to_string_lossy()) {
            return Health::BuildTree;
        }
        if !present {
            return Health::NoResident;
        }
        match what_is_registered(command, handler) {
            Registered::Correct => Health::Fine,
            Registered::WrongBinary(_) => Health::WrongBinary,
            Registered::Elsewhere(_) => Health::Stale,
            Registered::Absent => Health::NotRegistered,
        }
    }

    /// The resident, never this process: settings cannot open a link, and
    /// registering it is the one mistake this whole check exists to catch.
    fn handler() -> Option<PathBuf> {
        Some(link_handler(&std::env::current_exe().ok()?))
    }

    /// Reconciles on every launch, the way 1.x did: an update moves the
    /// executable and the keys keep pointing at a path that no longer exists.
    pub fn reconcile() {
        let Some(handler) = handler() else { return };
        // Registering a path nothing occupies hands every link to a process that cannot start.
        if !handler.exists() {
            return;
        }
        let registration = Registration::default();
        if registration.register(&handler.to_string_lossy()).is_ok() {
            notify_associations_changed();
        }
    }

    pub fn state() -> SystemState {
        let startup = startup_state();
        let registration = Registration::default();
        SystemState {
            health: health(&registration),
            registered_path: registration.registered_command(),
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
            let handler = handler().ok_or_else(|| "cannot find our own path".to_owned())?;
            registration.register(&handler.to_string_lossy())
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
    #[cfg(test)]
    mod tests {
        use super::super::Health;
        use super::verdict;
        use std::path::Path;

        /// The screen's reaction to each verdict is tested with literals; this
        /// is the side that decides which verdict it gets.
        #[test]
        fn each_registration_gets_the_verdict_the_screen_draws() {
            let installed = Path::new(r"C:\Program Files\LinkUnbound\linkunbound-shell.exe");

            assert_eq!(
                verdict(
                    Some(r#""C:\Program Files\LinkUnbound\linkunbound-shell.exe" "%1""#),
                    installed,
                    true
                ),
                Health::Fine
            );
            assert_eq!(
                verdict(
                    Some(r#""C:\Program Files\LinkUnbound\linkunbound-settings.exe" "%1""#),
                    installed,
                    true
                ),
                Health::WrongBinary,
                "settings cannot open a link, and registering it is the mistake this catches"
            );
            assert_eq!(
                verdict(
                    Some(r#""C:\Otra\LinkUnbound\linkunbound-shell.exe" "%1""#),
                    installed,
                    true
                ),
                Health::Stale
            );
            assert_eq!(verdict(None, installed, true), Health::NotRegistered);
        }

        /// A build that shipped without the resident read as healthy, because the command and the
        /// path it was compared against were the same absent file.
        #[test]
        fn a_missing_resident_is_named_rather_than_read_as_healthy() {
            let installed = Path::new(r"C:\Program Files\LinkUnbound\linkunbound-shell.exe");
            let command = format!("\"{}\" \"%1\"", installed.to_string_lossy());

            assert_eq!(
                verdict(Some(&command), installed, false),
                Health::NoResident,
                "a package that shipped without the resident read as Fine, because the command \
                 and the path it was compared against were the same absent file"
            );
        }

        /// A build tree cannot own the registration: its path disappears when
        /// the project is cleaned, and the dead key hijacks the installed copy.
        #[test]
        fn a_build_tree_is_named_before_anything_else_is_judged() {
            let built = r"D:\Code\LinkUnbound\target\release\linkunbound-shell.exe";
            let command = format!("\"{built}\" \"%1\"");
            assert_eq!(
                verdict(Some(&command), Path::new(built), true),
                Health::BuildTree,
                "even when the command points at itself"
            );
        }
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
            health: Health::NotRegistered,
            registered_path: None,
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
