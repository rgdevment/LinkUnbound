#[cfg(target_os = "macos")]
mod detect;
#[cfg(target_os = "macos")]
mod native;
#[cfg(target_os = "macos")]
mod registration;
#[cfg(target_os = "macos")]
mod startup;

#[cfg(target_os = "macos")]
pub use detect::installed_browsers;
#[cfg(target_os = "macos")]
pub use native::{
    copy_text, cursor, menu_bar_is_light, shift_is_down, source_app, windows_are_light,
    work_area_at,
};
#[cfg(target_os = "macos")]
pub use registration::{
    association_report, is_bundled, is_default_browser, own_bundle_id, register, unregister,
};
#[cfg(target_os = "macos")]
pub use startup::{Startup, set as set_startup, state as startup_state};

#[cfg(test)]
mod audit {
    use std::fs;
    use std::path::Path;

    /// `native.rs` is the audited exception, and reaches for it in one place: the pasteboard
    /// type is a framework constant, which `objc2` cannot declare safe. Anything else reaching
    /// for `unsafe` has to be reviewed and added here deliberately, never by accident.
    const AUDITED: [&str; 3] = ["native.rs", "registration.rs", "startup.rs"];

    /// Split so this file does not match its own search.
    const NEEDLE: &str = concat!("allow(unsafe", "_code)");

    fn rust_files(dir: &Path, found: &mut Vec<String>) {
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                rust_files(&path, found);
            } else if path.extension().is_some_and(|e| e == "rs") {
                let Ok(body) = fs::read_to_string(&path) else {
                    continue;
                };
                if body.contains(NEEDLE) {
                    let name = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or_default()
                        .to_owned();
                    found.push(name);
                }
            }
        }
    }

    #[test]
    fn unsafe_stays_in_the_places_it_was_audited() {
        let mut found = Vec::new();
        rust_files(
            Path::new(env!("CARGO_MANIFEST_DIR")).join("src").as_path(),
            &mut found,
        );
        found.sort();
        assert_eq!(
            found, AUDITED,
            "unsafe is allowed somewhere new; audit it and update this list"
        );
    }
}
