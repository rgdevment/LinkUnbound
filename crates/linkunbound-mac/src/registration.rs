#![allow(unsafe_code)]

use std::sync::mpsc::channel;
use std::time::Duration;

use block2::RcBlock;
use objc2_app_kit::NSWorkspace;
use objc2_foundation::{NSBundle, NSError, NSString, NSURL};

pub const SCHEMES: [&str; 2] = ["http", "https"];

const SAFARI: &str = "com.apple.Safari";

/// The person is asked by the system and may take their time, or say no.
const LONG_ENOUGH_TO_ANSWER: Duration = Duration::from_secs(120);

/// `Bundle.main`, never the Launch Services lookup: that returns whichever copy
/// the system happens to prefer, so with a debug build on the machine it would
/// register a path inside the build tree, and the association dies with the next
/// `cargo clean`.
fn ours() -> Option<objc2::rc::Retained<NSURL>> {
    let bundle = NSBundle::mainBundle();
    bundle.bundleIdentifier()?;
    Some(bundle.bundleURL())
}

#[must_use]
pub fn own_bundle_id() -> Option<String> {
    NSBundle::mainBundle()
        .bundleIdentifier()
        .map(|id| id.to_string())
}

/// True when this copy is running from a real bundle. A bare executable has no
/// identifier, and registering the directory it sits in claims the whole folder.
#[must_use]
pub fn is_bundled() -> bool {
    own_bundle_id().is_some()
}

fn handler_for(scheme: &str) -> Option<String> {
    let probe = NSURL::URLWithString(&NSString::from_str(&format!("{scheme}://example.com")))?;
    let app = NSWorkspace::sharedWorkspace().URLForApplicationToOpenURL(&probe)?;
    NSBundle::bundleWithURL(&app)?
        .bundleIdentifier()
        .map(|id| id.to_string())
}

fn holds(scheme: &str) -> bool {
    match (handler_for(scheme), own_bundle_id()) {
        (Some(held), Some(mine)) => held.eq_ignore_ascii_case(&mine),
        _ => false,
    }
}

#[must_use]
pub fn is_default_browser() -> bool {
    holds("https")
}

#[must_use]
pub fn association_report() -> Vec<(String, bool)> {
    SCHEMES
        .iter()
        .map(|scheme| ((*scheme).to_owned(), holds(scheme)))
        .collect()
}

/// Waits for the system's answer rather than firing and reporting success. The
/// person is prompted and may refuse; discarding that left the screen claiming a
/// registration nobody had granted.
fn point(app: &NSURL, scheme: &str) -> Result<(), String> {
    let (tx, rx) = channel();
    let handler = RcBlock::new(move |error: *mut NSError| {
        let said = if error.is_null() {
            Ok(())
        } else {
            // SAFETY: not null, and the framework owns it for the call.
            Err(unsafe { &*error }.localizedDescription().to_string())
        };
        let _ = tx.send(said);
    });

    NSWorkspace::sharedWorkspace()
        .setDefaultApplicationAtURL_toOpenURLsWithScheme_completionHandler(
            app,
            &NSString::from_str(scheme),
            Some(&handler),
        );

    match rx.recv_timeout(LONG_ENOUGH_TO_ANSWER) {
        Ok(said) => said,
        Err(_) => Err("defaultNoAnswer".to_owned()),
    }
}

pub fn register() -> Result<(), String> {
    let app = ours().ok_or_else(|| "notBundled".to_owned())?;
    for scheme in SCHEMES {
        point(&app, scheme)?;
    }
    Ok(())
}

/// The system has no "no default browser", so letting go means handing the
/// schemes back to Safari, which is where they were before.
pub fn unregister() -> Result<(), String> {
    let safari = NSWorkspace::sharedWorkspace()
        .URLForApplicationWithBundleIdentifier(&NSString::from_str(SAFARI))
        .ok_or_else(|| "noSafari".to_owned())?;
    for scheme in SCHEMES {
        point(&safari, scheme)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{SCHEMES, association_report, handler_for, is_bundled, is_default_browser};

    /// Both are what a browser is asked to carry, and holding one without the other is the
    /// state the screen has to be able to describe.
    #[test]
    fn the_report_names_every_scheme_a_browser_is_asked_to_carry() {
        let report = association_report();
        assert_eq!(report.len(), SCHEMES.len());
        for (scheme, _) in &report {
            assert!(SCHEMES.contains(&scheme.as_str()), "{scheme}");
        }
    }

    /// The test binary is not a bundle, so it holds nothing and must say so rather than
    /// reading some neighbouring directory as itself.
    #[test]
    fn a_binary_that_is_not_a_bundle_claims_nothing() {
        assert!(!is_bundled(), "a test binary is not an app bundle");
        assert!(!is_default_browser());
        assert!(association_report().iter().all(|(_, held)| !held));
    }

    /// Whatever is registered on this machine, the lookup has to answer with something a
    /// bundle identifier can be read from, or the comparison silently reads false for ever.
    #[test]
    fn the_machine_names_a_handler_for_the_web() {
        let held = handler_for("https");
        assert!(held.is_some(), "no application opens https here");
        assert!(held.is_some_and(|id| id.contains('.')), "not a bundle id");
    }
}
