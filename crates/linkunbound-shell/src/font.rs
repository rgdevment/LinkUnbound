//! The family the picker and the notice write in, and what settings offers in its place.

use linkunbound_core::fonts::{FALLBACK, Family, worn};

use crate::{Notice, Picker};

#[cfg(windows)]
use linkunbound_win::fonts as here;

#[cfg(target_os = "macos")]
use linkunbound_mac::fonts as here;

/// The family the system writes its own dialogs in, the one the picker always drew with.
#[must_use]
pub fn system() -> String {
    #[cfg(windows)]
    if let Some(family) = here::system() {
        return family;
    }
    FALLBACK.to_owned()
}

#[cfg(any(windows, target_os = "macos"))]
#[must_use]
pub fn installed(name: &str) -> bool {
    here::installed(name)
}

#[cfg(not(any(windows, target_os = "macos")))]
#[must_use]
pub const fn installed(_name: &str) -> bool {
    false
}

#[cfg(any(windows, target_os = "macos"))]
#[must_use]
pub fn families() -> Vec<Family> {
    here::families()
}

#[cfg(not(any(windows, target_os = "macos")))]
#[must_use]
pub const fn families() -> Vec<Family> {
    Vec::new()
}

/// Both windows get the family asked for, or the system's when it was uninstalled since.
pub fn wear(picker: &Picker, notice: &Notice, asked: Option<&str>) {
    let family = worn(asked, installed, system);
    picker.set_family(family.as_str().into());
    notice.set_family(family.into());
}

#[cfg(test)]
mod tests {
    use super::{families, installed, system, wear};

    fn windows() -> (crate::Picker, crate::Notice) {
        thread_local!(static BACKEND: () = i_slint_backend_testing::init_no_event_loop());
        BACKEND.with(|()| ());
        (
            crate::Picker::new().expect("a picker"),
            crate::Notice::new().expect("a notice"),
        )
    }

    #[test]
    fn untouched_both_windows_write_in_the_system_family() {
        let (picker, notice) = windows();
        wear(&picker, &notice, None);
        assert_eq!(picker.get_family(), system().as_str());
        assert_eq!(notice.get_family(), system().as_str());
    }

    #[test]
    fn a_family_nobody_installed_falls_back_to_the_system_one() {
        let (picker, notice) = windows();
        wear(&picker, &notice, Some("Zzz Nobody Installed This"));
        assert_eq!(picker.get_family(), system().as_str());
        assert_eq!(notice.get_family(), system().as_str());
    }

    #[cfg(any(windows, target_os = "macos"))]
    #[test]
    fn an_installed_family_is_worn_by_both_windows() {
        let (picker, notice) = windows();
        let chosen = families()
            .into_iter()
            .map(|one| one.name)
            .find(|name| *name != system() && installed(name))
            .expect("a second family");
        wear(&picker, &notice, Some(&chosen));
        assert_eq!(picker.get_family(), chosen.as_str());
        assert_eq!(notice.get_family(), chosen.as_str());
    }

    /// Segoe UI on an English Windows; Yu Gothic UI and the like where the system writes in them.
    #[cfg(windows)]
    #[test]
    fn the_system_family_on_windows_is_the_message_font_family() {
        let family = system();
        assert_eq!(
            Some(family.as_str()),
            linkunbound_win::fonts::system().as_deref()
        );
        assert!(installed(&family), "{family}");
    }
}
