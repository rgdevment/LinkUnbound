use windows::ApplicationModel::{StartupTask, StartupTaskState};
use windows::core::HSTRING;
use winreg::RegKey;
use winreg::enums::{HKEY_CURRENT_USER, KEY_READ, KEY_WRITE};

const TASK_ID: &str = "LinkUnboundStartup";
const RUN: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
/// The resident is what has to be there at sign-in: it owns the tray and the shortcut, and it is
/// what makes the first link of a session open without waiting for a cold start.
const RUN_NAME: &str = "LinkUnbound";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Startup {
    pub enabled: bool,
    /// False when Windows or the user disabled it outside the app, which cannot
    /// be undone from here: the toggle has to explain instead of pretending.
    pub ours_to_change: bool,
}

impl Default for Startup {
    fn default() -> Self {
        Self {
            enabled: false,
            ours_to_change: true,
        }
    }
}

fn task() -> Option<StartupTask> {
    StartupTask::GetAsync(&HSTRING::from(TASK_ID))
        .ok()?
        .get()
        .ok()
}

/// A packaged build has an identity and an unpackaged one does not, which is the only honest way
/// to tell them apart. It matters because a packaged app's writes to the Run key land in a
/// container Windows never reads at sign-in: the switch would say yes and nothing would start.
#[must_use]
pub fn packaged() -> bool {
    windows::ApplicationModel::Package::Current().is_ok()
}

/// A packaged build has a startup task and nothing else: its writes to the Run key land in a
/// container Windows never reads at sign-in. Everywhere else the Run key is the whole mechanism.
#[must_use]
pub fn state() -> Option<Startup> {
    if packaged() {
        // Inside a package the startup task is the only mechanism, so a failure to read it is a
        // failure to answer — not a reason to go looking somewhere Windows does not read.
        let state = task()?.State().ok()?;
        return Some(Startup {
            enabled: state == StartupTaskState::Enabled
                || state == StartupTaskState::EnabledByPolicy,
            ours_to_change: state != StartupTaskState::DisabledByUser
                && state != StartupTaskState::DisabledByPolicy,
        });
    }
    if let Some(task) = task() {
        let state = task.State().ok()?;
        return Some(Startup {
            enabled: state == StartupTaskState::Enabled
                || state == StartupTaskState::EnabledByPolicy,
            ours_to_change: state != StartupTaskState::DisabledByUser
                && state != StartupTaskState::DisabledByPolicy,
        });
    }
    Some(Startup {
        enabled: listed(),
        ours_to_change: true,
    })
}

pub fn set(enabled: bool) -> Option<Startup> {
    if packaged() {
        let task = task()?;
        if enabled {
            task.RequestEnableAsync().ok()?.get().ok()?;
        } else {
            task.Disable().ok()?;
        }
        return state();
    }
    if let Some(task) = task() {
        if enabled {
            task.RequestEnableAsync().ok()?.get().ok()?;
        } else {
            task.Disable().ok()?;
        }
        return state();
    }
    list(enabled)?;
    state()
}

fn listed() -> bool {
    RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey_with_flags(RUN, KEY_READ)
        .and_then(|run| run.get_value::<String, _>(RUN_NAME))
        .is_ok()
}

/// Quoted, because a path with a space in it is read as a command and its arguments otherwise —
/// and `%LOCALAPPDATA%\Programs\LinkUnbound` is where the installer puts this.
fn command(running: &std::path::Path) -> String {
    format!(
        "\"{}\" --hushed",
        linkunbound_core::link_handler(running).display()
    )
}

fn list(enabled: bool) -> Option<()> {
    let run = RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey_with_flags(RUN, KEY_READ | KEY_WRITE)
        .ok()?;
    if enabled {
        run.set_value(RUN_NAME, &command(&std::env::current_exe().ok()?))
            .ok()?;
    } else {
        // Deleting what is not there is not a failure: the switch reads as off either way.
        let _ = run.delete_value(RUN_NAME);
    }
    Some(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A `cargo test` binary is never a packaged MSIX identity, so there is no startup task to
    /// find and the Run key is what answers — which is the standalone install's whole mechanism.
    #[test]
    fn outside_a_package_the_run_key_is_what_answers() {
        let said = state().expect("the registry can always be read");
        assert!(
            said.ours_to_change,
            "nothing but a packaged build can have it disabled by policy"
        );
    }

    /// The resident is what has to be there at sign-in, not the window that was running when the
    /// switch was flipped.
    #[test]
    fn what_would_be_listed_is_the_resident() {
        let from = std::path::Path::new(
            r"C:\Users\x\AppData\Local\Programs\LinkUnbound\linkunbound-settings.exe",
        );
        let said = command(from);

        assert!(said.contains("linkunbound-shell"), "{said}");
        assert!(!said.contains("settings"), "{said}");
        assert!(
            said.ends_with("--hushed"),
            "sign-in is the one launch that must not open a window: {said}"
        );
    }

    /// Unquoted, `C:\Program Files\...` is read as `C:\Program` with `Files\...` for arguments,
    /// and nothing starts at sign-in.
    #[test]
    fn a_path_with_a_space_is_quoted() {
        let from = std::path::Path::new(r"C:\Program Files\LinkUnbound\linkunbound-settings.exe");
        let said = command(from);

        assert!(
            said.starts_with(r#""C:\Program Files\"#),
            "unquoted, the path is read as a command with arguments: {said}"
        );
    }
}
