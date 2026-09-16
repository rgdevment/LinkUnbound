#![allow(unsafe_code)]

use std::sync::mpsc::{Receiver, channel};
use std::time::Duration;

use block2::RcBlock;
use objc2::rc::Retained;
use objc2_app_kit::NSWorkspace;
use objc2_foundation::{
    NSBundle, NSCocoaErrorDomain, NSError, NSString, NSURL, NSUserCancelledError, NSUserDefaults,
};
use objc2_uniform_type_identifiers::UTType;

pub const SCHEMES: [&str; 2] = ["http", "https"];

const DOCUMENTS: [&str; 3] = ["public.html", "public.xhtml", "public.svg-image"];

const SAFARI: &str = "com.apple.Safari";

const HANDED_FROM: &str = "HandedFrom";

/// The person is asked by the system and may take their time, or say no.
const LONG_ENOUGH_TO_ANSWER: Duration = Duration::from_secs(120);

/// `Bundle.main`, never the Launch Services lookup: that returns whichever copy
/// the system happens to prefer, so with a debug build on the machine it would
/// register a path inside the build tree, and the association dies with the next
/// `cargo clean`.
fn ours() -> Option<Retained<NSURL>> {
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

fn is_ours(bundle_id: &str) -> bool {
    own_bundle_id().is_some_and(|mine| mine.eq_ignore_ascii_case(bundle_id))
}

fn remember(handed_from: Option<&str>, key: &str) {
    let Some(previous) = handed_from.filter(|id| !is_ours(id)) else {
        return;
    };
    unsafe {
        NSUserDefaults::standardUserDefaults().setObject_forKey(
            Some(&NSString::from_str(previous)),
            &NSString::from_str(key),
        );
    }
}

fn remembered(key: &str) -> Option<String> {
    NSUserDefaults::standardUserDefaults()
        .stringForKey(&NSString::from_str(key))
        .map(|id| id.to_string())
}

fn declined(error: &NSError) -> bool {
    let cocoa = unsafe { NSCocoaErrorDomain };
    error.domain().isEqualToString(cocoa) && error.code() == NSUserCancelledError
}

fn answer(error: *mut NSError) -> Result<(), String> {
    if error.is_null() {
        return Ok(());
    }
    let error = unsafe { &*error };
    if declined(error) {
        return Err("defaultDeclined".to_owned());
    }
    Err(error.localizedDescription().to_string())
}

type Answer = Result<(), String>;

fn answered() -> (RcBlock<dyn Fn(*mut NSError)>, Receiver<Answer>) {
    let (tx, rx) = channel();
    let handler = RcBlock::new(move |error: *mut NSError| {
        let _ = tx.send(answer(error));
    });
    (handler, rx)
}

fn waited(rx: &Receiver<Answer>) -> Answer {
    match rx.recv_timeout(LONG_ENOUGH_TO_ANSWER) {
        Ok(said) => said,
        Err(_) => Err("defaultNoAnswer".to_owned()),
    }
}

/// Waits for the system's answer rather than firing and reporting success. The
/// person is prompted and may refuse; discarding that left the screen claiming a
/// registration nobody had granted.
fn point(app: &NSURL, scheme: &str) -> Result<(), String> {
    let (handler, rx) = answered();
    NSWorkspace::sharedWorkspace()
        .setDefaultApplicationAtURL_toOpenURLsWithScheme_completionHandler(
            app,
            &NSString::from_str(scheme),
            Some(&handler),
        );
    waited(&rx)
}

fn document_handler(identifier: &str) -> Option<String> {
    let kind = UTType::typeWithIdentifier(&NSString::from_str(identifier))?;
    let app = NSWorkspace::sharedWorkspace().URLForApplicationToOpenContentType(&kind)?;
    bundle_id_of(&app)
}

fn key_for(identifier: &str) -> String {
    format!("{HANDED_FROM}.{identifier}")
}

fn point_documents(app: &NSURL, identifier: &str) -> Result<(), String> {
    let Some(kind) = UTType::typeWithIdentifier(&NSString::from_str(identifier)) else {
        return Ok(());
    };
    let (handler, rx) = answered();
    NSWorkspace::sharedWorkspace().setDefaultApplicationAtURL_toOpenContentType_completionHandler(
        app,
        &kind,
        Some(&handler),
    );
    waited(&rx)
}

fn bundle_id_of(app: &NSURL) -> Option<String> {
    NSBundle::bundleWithURL(app)?
        .bundleIdentifier()
        .map(|id| id.to_string())
}

fn points_at(app: &NSURL, scheme: &str) -> bool {
    match (handler_for(scheme), bundle_id_of(app)) {
        (Some(held), Some(wanted)) => held.eq_ignore_ascii_case(&wanted),
        _ => false,
    }
}

fn point_everything(app: &NSURL) -> Result<(), String> {
    point(app, "http")?;
    for scheme in SCHEMES {
        if !points_at(app, scheme) {
            point(app, scheme)?;
        }
    }
    for document in DOCUMENTS {
        point_documents(app, document)?;
    }
    Ok(())
}

pub fn register() -> Result<(), String> {
    let app = ours().ok_or_else(|| "notBundled".to_owned())?;
    remember(handler_for("https").as_deref(), HANDED_FROM);
    for document in DOCUMENTS {
        remember(document_handler(document).as_deref(), &key_for(document));
    }
    point_everything(&app)
}

fn app_remembered_as(key: &str) -> Option<Retained<NSURL>> {
    let id = remembered(key).filter(|id| !is_ours(id))?;
    NSWorkspace::sharedWorkspace().URLForApplicationWithBundleIdentifier(&NSString::from_str(&id))
}

/// The system has no "no default browser", so letting go hands the schemes
/// back to whoever held them before, and to Safari when nobody is remembered.
pub fn unregister() -> Result<(), String> {
    let browser = app_remembered_as(HANDED_FROM)
        .or_else(|| {
            NSWorkspace::sharedWorkspace()
                .URLForApplicationWithBundleIdentifier(&NSString::from_str(SAFARI))
        })
        .ok_or_else(|| "noPreviousBrowser".to_owned())?;
    point(&browser, "http")?;
    for scheme in SCHEMES {
        if !points_at(&browser, scheme) {
            point(&browser, scheme)?;
        }
    }
    for document in DOCUMENTS {
        let target = app_remembered_as(&key_for(document)).unwrap_or_else(|| browser.clone());
        point_documents(&target, document)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        SCHEMES, association_report, handler_for, is_bundled, is_default_browser, remember,
        remembered,
    };
    use objc2_foundation::{NSString, NSUserDefaults};

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

    #[test]
    fn the_system_saying_nothing_is_a_yes_and_a_refusal_is_named() {
        use objc2_foundation::{NSCocoaErrorDomain, NSError, NSUserCancelledError};
        assert_eq!(super::answer(std::ptr::null_mut()), Ok(()));
        let cocoa = unsafe { NSCocoaErrorDomain };
        let declined =
            unsafe { NSError::errorWithDomain_code_userInfo(cocoa, NSUserCancelledError, None) };
        assert!(super::declined(&declined));
        assert_eq!(
            super::answer(objc2::rc::Retained::as_ptr(&declined).cast_mut()),
            Err("defaultDeclined".to_owned())
        );
        let other = unsafe { NSError::errorWithDomain_code_userInfo(cocoa, 256, None) };
        assert!(!super::declined(&other));
        assert!(super::answer(objc2::rc::Retained::as_ptr(&other).cast_mut()).is_err());
    }

    #[test]
    fn a_bundle_is_known_by_its_identifier_and_never_taken_for_us() {
        use objc2_app_kit::NSWorkspace;
        use objc2_foundation::NSString;
        let safari = NSWorkspace::sharedWorkspace()
            .URLForApplicationWithBundleIdentifier(&NSString::from_str("com.apple.Safari"))
            .expect("Safari is always there");
        assert_eq!(
            super::bundle_id_of(&safari).as_deref(),
            Some("com.apple.Safari")
        );
        assert!(!super::is_ours("com.apple.Safari"));
        let holds_https =
            handler_for("https").is_some_and(|id| id.eq_ignore_ascii_case("com.apple.Safari"));
        assert_eq!(super::points_at(&safari, "https"), holds_https);
    }

    #[test]
    fn each_document_type_remembers_its_own_previous_handler() {
        assert_eq!(
            super::key_for("public.svg-image"),
            "HandedFrom.public.svg-image"
        );
        assert_ne!(
            super::key_for("public.html"),
            super::key_for("public.xhtml")
        );
        let held = super::document_handler("public.html");
        assert!(
            held.is_some_and(|id| id.contains('.')),
            "html has a handler on any Mac"
        );
        assert!(super::document_handler("com.example.no.such.type").is_none());
    }

    #[test]
    fn a_binary_that_is_not_a_bundle_cannot_register_or_be_pointed_at() {
        assert!(super::ours().is_none());
        assert_eq!(super::register(), Err("notBundled".to_owned()));
    }

    #[test]
    fn the_system_is_waited_for_and_its_silence_is_not_taken_for_a_yes() {
        let (handler, rx) = super::answered();
        handler.call((std::ptr::null_mut(),));
        assert_eq!(super::waited(&rx), Ok(()));

        let (handler, rx) = super::answered();
        let cocoa = unsafe { objc2_foundation::NSCocoaErrorDomain };
        let declined = unsafe {
            objc2_foundation::NSError::errorWithDomain_code_userInfo(
                cocoa,
                objc2_foundation::NSUserCancelledError,
                None,
            )
        };
        handler.call((objc2::rc::Retained::as_ptr(&declined).cast_mut(),));
        assert_eq!(super::waited(&rx), Err("defaultDeclined".to_owned()));

        let (_, gone) = std::sync::mpsc::channel::<super::Answer>();
        assert_eq!(super::waited(&gone), Err("defaultNoAnswer".to_owned()));
    }

    #[test]
    fn a_document_type_the_system_does_not_know_is_not_pointed_anywhere() {
        use objc2_app_kit::NSWorkspace;
        use objc2_foundation::NSString;
        let safari = NSWorkspace::sharedWorkspace()
            .URLForApplicationWithBundleIdentifier(&NSString::from_str("com.apple.Safari"))
            .expect("Safari is always there");
        assert_eq!(
            super::point_documents(&safari, "com.example.no.such.type"),
            Ok(())
        );
    }

    #[test]
    fn a_remembered_browser_is_found_again_and_a_forgotten_one_is_not() {
        let key = format!("HandedFromTestApp{}", std::process::id());
        assert!(super::app_remembered_as(&key).is_none());
        remember(Some("com.apple.Safari"), &key);
        let found = super::app_remembered_as(&key).expect("Safari is always there");
        assert!(
            found
                .path()
                .is_some_and(|p| p.to_string().ends_with("Safari.app")),
            "{found:?}"
        );
        remember(Some("com.example.gone.browser"), &key);
        assert!(super::app_remembered_as(&key).is_none());
        NSUserDefaults::standardUserDefaults().removeObjectForKey(&NSString::from_str(&key));
    }

    #[test]
    fn the_browser_that_held_the_links_is_remembered_and_nothing_is_not() {
        let key = format!("HandedFromTest{}", std::process::id());
        remember(None, &key);
        assert!(remembered(&key).is_none());
        remember(Some("com.example.browser"), &key);
        assert_eq!(remembered(&key).as_deref(), Some("com.example.browser"));
        NSUserDefaults::standardUserDefaults().removeObjectForKey(&NSString::from_str(&key));
        assert!(remembered(&key).is_none());
    }
}
