#![allow(unsafe_code)]

use objc2_service_management::{SMAppService, SMAppServiceStatus};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Startup {
    pub enabled: bool,
    /// False when the person turned it off in System Settings, which cannot be
    /// undone from here: the toggle has to explain instead of pretending.
    pub ours_to_change: bool,
}

/// `RequiresApproval` is the system saying the person went to Login Items and
/// switched it off. Registering again does nothing, and a toggle that springs
/// back is worse than one that says why.
fn read(status: SMAppServiceStatus) -> Startup {
    Startup {
        enabled: status == SMAppServiceStatus::Enabled,
        ours_to_change: status != SMAppServiceStatus::RequiresApproval,
    }
}

fn service() -> Option<objc2::rc::Retained<SMAppService>> {
    if !crate::registration::is_bundled() {
        return None;
    }
    // SAFETY: a class method taking nothing and returning the shared service.
    Some(unsafe { SMAppService::mainAppService() })
}

#[must_use]
pub fn state() -> Option<Startup> {
    let service = service()?;
    // SAFETY: reads a property of the service above.
    Some(read(unsafe { service.status() }))
}

/// `Err` carries the system's own words: an ad-hoc signed or translocated
/// copy is refused with a reason, and "no service" would hide it.
pub fn set(enabled: bool) -> Result<Startup, String> {
    let service = service().ok_or_else(|| "noStartupTask".to_owned())?;
    // SAFETY: both take nothing and report through a returned error.
    let done = unsafe {
        if enabled {
            service.registerAndReturnError()
        } else {
            service.unregisterAndReturnError()
        }
    };
    done.map_err(|error| error.localizedDescription().to_string())?;
    state().ok_or_else(|| "noStartupTask".to_owned())
}

#[cfg(test)]
mod tests {
    use super::{SMAppServiceStatus, read, set, state};

    /// Each state the system reports has to reach the toggle as a different thing: enabled,
    /// off but ours to turn on, and off in a way only System Settings can undo.
    #[test]
    fn every_status_the_system_reports_reaches_the_toggle_as_itself() {
        let on = read(SMAppServiceStatus::Enabled);
        assert!(on.enabled && on.ours_to_change);

        let off = read(SMAppServiceStatus::NotRegistered);
        assert!(!off.enabled && off.ours_to_change);

        let refused = read(SMAppServiceStatus::RequiresApproval);
        assert!(
            !refused.enabled && !refused.ours_to_change,
            "a toggle that springs back is worse than one that says why"
        );

        let missing = read(SMAppServiceStatus::NotFound);
        assert!(!missing.enabled && missing.ours_to_change);
    }

    /// A bare executable is not a login item the system can register, and asking it to be one
    /// registers whatever directory it happens to sit in.
    #[test]
    fn a_binary_that_is_not_a_bundle_has_no_login_item_to_offer() {
        assert!(state().is_none());
        assert_eq!(set(true), Err("noStartupTask".to_owned()));
        assert_eq!(set(false), Err("noStartupTask".to_owned()));
    }
}
