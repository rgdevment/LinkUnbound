use windows::ApplicationModel::{StartupTask, StartupTaskState};
use windows::core::HSTRING;

const TASK_ID: &str = "LinkUnboundStartup";

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

/// Only a packaged build has a startup task; the plain installer arranges it
/// through the registry instead.
#[must_use]
pub fn state() -> Option<Startup> {
    let state = task()?.State().ok()?;
    Some(Startup {
        enabled: state == StartupTaskState::Enabled || state == StartupTaskState::EnabledByPolicy,
        ours_to_change: state != StartupTaskState::DisabledByUser
            && state != StartupTaskState::DisabledByPolicy,
    })
}

pub fn set(enabled: bool) -> Option<Startup> {
    let task = task()?;
    if enabled {
        task.RequestEnableAsync().ok()?.get().ok()?;
    } else {
        task.Disable().ok()?;
    }
    state()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A `cargo test` binary is never a packaged MSIX identity, so `task()` has no
    /// startup task to find and this is deterministic rather than environment-dependent.
    #[test]
    fn asking_outside_a_package_answers_nothing_rather_than_failing() {
        assert!(state().is_none());
    }

    #[test]
    fn setting_it_outside_a_package_answers_nothing_rather_than_panicking() {
        assert!(set(true).is_none());
    }

    #[test]
    fn nothing_saved_yet_defaults_to_off_but_still_ours_to_change() {
        let default = Startup::default();
        assert!(!default.enabled);
        assert!(default.ours_to_change);
    }
}
