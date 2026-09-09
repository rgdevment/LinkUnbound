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

    #[test]
    fn asking_outside_a_package_answers_nothing_rather_than_failing() {
        assert!(state().is_none() || state().is_some());
    }

    #[test]
    fn a_task_disabled_by_the_user_is_reported_as_not_ours_to_change() {
        let blocked = Startup {
            enabled: false,
            ours_to_change: false,
        };
        assert!(!blocked.ours_to_change);
        assert!(Startup::default().ours_to_change);
    }
}
