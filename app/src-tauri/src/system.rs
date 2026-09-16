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
#[cfg_attr(not(windows), allow(dead_code))]
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
    /// Running from the disk image it was downloaded as: the registration dies when it is ejected.
    #[cfg(target_os = "macos")]
    Mounted,
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
        // The manifest is the registration inside a package, and there is no path by which it can
        // be wrong: it shipped with the build.
        if linkunbound_win::packaged() {
            return Health::Fine;
        }
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
        let registered = what_is_registered(command, handler);
        // A build tree that holds the registration was registered on purpose, by the button
        // that says so; one that does not is the mistake the check exists to name.
        if is_build_tree(&handler.to_string_lossy()) && registered != Registered::Correct {
            return Health::BuildTree;
        }
        if !present {
            return Health::NoResident;
        }
        match registered {
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
        // A packaged build is registered by its manifest, and its writes to these keys land in a
        // container the shell never reads — so writing them would only teach the health panel to
        // report a registration nothing honours.
        if linkunbound_win::packaged() {
            return;
        }
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
    /// Straight to our own page rather than the list of every app on the machine: the person
    /// pressed a button that named us, and landing on an alphabetical list of a hundred entries
    /// leaves them to find us. The name is the one under `RegisteredApplications`.
    pub fn open_default_apps() -> Result<(), String> {
        std::process::Command::new("cmd")
            .args([
                "/C",
                "start",
                "",
                "ms-settings:defaultapps?registeredAppUser=LinkUnbound",
            ])
            .spawn()
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

    /// From a build tree, with eyes open: the registration points into `target/` until the
    /// next reconcile refuses to touch it, and the links stop when the folder goes.
    pub fn register_anyway() -> Result<SystemState, String> {
        let handler = handler().ok_or_else(|| "ownPathUnknown".to_owned())?;
        if !handler.exists() {
            return Err("noResident".to_owned());
        }
        Registration::default()
            .register_wherever(&handler.to_string_lossy())
            .map_err(|e| e.to_string())?;
        notify_associations_changed();
        Ok(state())
    }

    pub fn set_registered(enabled: bool) -> Result<SystemState, String> {
        let registration = Registration::default();
        let outcome = if enabled {
            let handler = handler().ok_or_else(|| "ownPathUnknown".to_owned())?;
            registration.register(&handler.to_string_lossy())
        } else {
            registration.unregister()
        };
        outcome.map_err(|e| e.to_string())?;
        notify_associations_changed();
        Ok(state())
    }

    pub fn set_starts_with_system(enabled: bool) -> Result<SystemState, String> {
        set_startup(enabled).ok_or_else(|| "noStartupTask".to_owned())?;
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

        /// A build tree cannot own the registration by accident: its path disappears when the
        /// project is cleaned, and the dead key hijacks the installed copy. Holding it on purpose,
        /// through the button, reads as fine — that is what the button is for.
        #[test]
        fn a_build_tree_is_named_before_anything_else_is_judged() {
            let built = r"D:\Code\LinkUnbound\target\release\linkunbound-shell.exe";
            let command = format!("\"{built}\" \"%1\"");
            assert_eq!(
                verdict(Some(&command), Path::new(built), true),
                Health::Fine,
                "registered on purpose, from the build tree, and pointing here"
            );
            assert_eq!(
                verdict(
                    Some(r#""C:\Elsewhere\linkunbound-shell.exe" "%1""#),
                    Path::new(built),
                    true
                ),
                Health::BuildTree,
                "a build tree holding nothing of its own is the mistake the check names"
            );
            assert_eq!(
                verdict(None, Path::new(built), true),
                Health::BuildTree,
                "and so is one not registered at all"
            );
        }
    }
}

#[cfg(target_os = "macos")]
mod platform {
    use super::{Association, Health, SystemState};
    use linkunbound_mac::{
        association_report, is_bundled, is_default_browser, own_bundle_id, register, set_startup,
        startup_state, unregister,
    };

    /// Where the person is sent to choose. Ventura moved the default browser out
    /// of its own pane and into Desktop & Dock.
    const DEFAULT_BROWSER_PANE: &str =
        "x-apple.systempreferences:com.apple.Desktop-Settings.extension";

    /// The declaration lives in the bundle's `Info.plist` and ships with the
    /// build: there is nothing to reconcile at launch, and Launch Services owns
    /// the final choice the way `UserChoice` does on Windows.
    pub fn reconcile() {}

    /// Only two states can be true here. A copy outside a bundle declares no
    /// schemes at all, and the registration it would claim is the directory it
    /// happens to sit in.
    fn health(bundled: bool, default: bool, mounted: bool) -> Health {
        if !bundled {
            return Health::BuildTree;
        }
        if mounted {
            return Health::Mounted;
        }
        if default {
            Health::Fine
        } else {
            Health::NotRegistered
        }
    }

    pub fn state() -> SystemState {
        let startup = startup_state();
        let bundled = is_bundled();
        let default = is_default_browser();
        SystemState {
            health: health(bundled, default, crate::update::from_a_mount()),
            registered_path: own_bundle_id(),
            edge_installed: false,
            registered: bundled,
            is_default: default,
            associations: association_report()
                .into_iter()
                .map(|(scheme, held)| Association { scheme, held })
                .collect(),
            starts_with_system: startup.as_ref().is_some_and(|s| s.enabled),
            startup_is_ours: startup.is_none_or(|s| s.ours_to_change),
        }
    }

    pub fn open_default_apps() -> Result<(), String> {
        linkunbound_core::spawn_and_forget(
            std::process::Command::new("/usr/bin/open").arg(DEFAULT_BROWSER_PANE),
        )
        .map_err(|e| e.to_string())
    }

    /// Nothing outside a bundle can be registered, from a build tree or anywhere else.
    pub fn register_anyway() -> Result<SystemState, String> {
        Err("ownPathUnknown".to_owned())
    }

    /// The system has no "no default browser", so letting go hands the schemes
    /// back to Safari rather than leaving links with nowhere to arrive.
    pub fn set_registered(enabled: bool) -> Result<SystemState, String> {
        if enabled && crate::update::from_a_mount() {
            return Err("mounted".to_owned());
        }
        if enabled { register() } else { unregister() }?;
        Ok(state())
    }

    pub fn set_starts_with_system(enabled: bool) -> Result<SystemState, String> {
        if crate::update::from_a_mount() {
            return Err("mounted".to_owned());
        }
        set_startup(enabled)?;
        Ok(state())
    }

    pub fn browsers() -> Vec<linkunbound_core::Browser> {
        linkunbound_mac::installed_browsers()
    }

    #[cfg(test)]
    mod tests {
        use super::super::Health;
        use super::health;

        /// The screen draws a different panel for each, and a copy running outside a bundle
        /// has to be told so rather than shown a button that cannot work.
        #[test]
        fn each_state_gets_the_verdict_the_screen_draws() {
            assert_eq!(health(true, true, false), Health::Fine);
            assert_eq!(health(true, false, false), Health::NotRegistered);
            assert_eq!(
                health(false, false, false),
                Health::BuildTree,
                "a bare executable declares no schemes and owns no registration"
            );
            assert_eq!(
                health(false, true, false),
                Health::BuildTree,
                "whatever the system says, an unbundled copy is not what it named"
            );
            assert_eq!(
                health(true, true, true),
                Health::Mounted,
                "a registration pointing into a disk image dies with the eject"
            );
            assert_eq!(health(false, false, true), Health::BuildTree);
        }

        #[test]
        fn the_bundle_launches_the_resident_for_a_link() {
            let plist = include_str!("../Info.plist");
            let executable = plist
                .split("<key>CFBundleExecutable</key>")
                .nth(1)
                .and_then(|rest| rest.split("<string>").nth(1))
                .and_then(|rest| rest.split("</string>").next())
                .expect("the plist names an executable");
            assert_eq!(executable, "linkunbound-shell");
            assert!(plist.contains("<key>LSUIElement</key>"));
        }

        /// A test binary is not a bundle: the screen has to be told so, and both switches
        /// have to refuse with a reason rather than register the directory the binary sits in.
        /// Letting go is never tried here, since it would hand the links to Safari for real.
        #[test]
        fn a_copy_outside_a_bundle_is_reported_and_refused_rather_than_registered() {
            let state = super::state();
            assert_eq!(state.health, Health::BuildTree);
            assert!(!state.registered);
            assert!(state.registered_path.is_none());
            assert!(!state.is_default);
            assert_eq!(state.associations.len(), 2);
            assert!(state.associations.iter().all(|a| !a.held));
            assert!(!state.starts_with_system);
            assert!(state.startup_is_ours);
            assert_eq!(
                super::set_registered(true).map(|_| ()),
                Err("notBundled".to_owned())
            );
            assert_eq!(
                super::set_starts_with_system(true).map(|_| ()),
                Err("noStartupTask".to_owned())
            );
        }
    }
}

#[cfg(not(any(windows, target_os = "macos")))]
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

    pub fn register_anyway() -> Result<SystemState, String> {
        Err("notOnThisPlatform".to_owned())
    }

    pub fn set_registered(_enabled: bool) -> Result<SystemState, String> {
        Err("notOnThisPlatform".to_owned())
    }

    pub fn set_starts_with_system(_enabled: bool) -> Result<SystemState, String> {
        Err("notOnThisPlatform".to_owned())
    }

    pub fn browsers() -> Vec<linkunbound_core::Browser> {
        Vec::new()
    }

    pub fn open_default_apps() -> Result<(), String> {
        Err("notOnThisPlatform".to_owned())
    }
}

pub use platform::{
    browsers, open_default_apps, reconcile, register_anyway, set_registered,
    set_starts_with_system, state,
};
