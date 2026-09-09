#[cfg(windows)]
mod detect;
#[cfg(windows)]
mod native;
#[cfg(windows)]
mod registration;
#[cfg(windows)]
mod startup;

#[cfg(windows)]
pub use detect::{chromium_profiles, installed_browsers};
#[cfg(windows)]
pub use native::{notify_associations_changed, source_app};
#[cfg(windows)]
pub use registration::{
    PROG_ID, Registration, association_report, is_default_browser, prog_id_is_ours,
};
#[cfg(windows)]
pub use startup::{Startup, set as set_startup, state as startup_state};

#[derive(Debug, thiserror::Error)]
pub enum RegistrationError {
    #[error("the registry refused the write: {0}")]
    Registry(#[from] std::io::Error),
}

#[cfg(test)]
mod audit {
    use std::fs;
    use std::path::Path;

    /// `native.rs` is the audited exception. Anything else reaching for `unsafe`
    /// has to be reviewed and added here deliberately, never by accident.
    const AUDITED: [&str; 1] = ["native.rs"];

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
    fn unsafe_stays_in_the_one_place_it_was_audited() {
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
