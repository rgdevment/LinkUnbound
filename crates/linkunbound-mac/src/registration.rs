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

pub const DOCUMENTS: [&str; 3] = ["public.html", "public.xhtml", "public.svg-image"];

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
    objc2::rc::autoreleasepool(|_| holds("https"))
}

#[must_use]
pub fn association_report() -> Vec<(String, bool)> {
    objc2::rc::autoreleasepool(|_| {
        SCHEMES
            .iter()
            .map(|scheme| ((*scheme).to_owned(), holds(scheme)))
            .collect()
    })
}

fn is_ours(bundle_id: &str) -> bool {
    is_ours_among(bundle_id, own_bundle_id().as_deref())
}

/// The tests write somewhere of their own: the binary they run in has no
/// bundle, and its standard domain would leave a file behind per build.
fn defaults() -> Retained<NSUserDefaults> {
    #[cfg(test)]
    {
        tests::suite()
    }
    #[cfg(not(test))]
    {
        NSUserDefaults::standardUserDefaults()
    }
}

fn remember(handed_from: Option<&str>, key: &str) {
    let Some(previous) = handed_from.filter(|id| !is_ours(id)) else {
        return;
    };
    unsafe {
        defaults().setObject_forKey(
            Some(&NSString::from_str(previous)),
            &NSString::from_str(key),
        );
    }
}

fn remembered(key: &str) -> Option<String> {
    defaults()
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

#[derive(Debug, PartialEq, Eq)]
enum Step {
    Scheme(&'static str),
    Document(&'static str),
}

/// What is left once `http` has been asked for: pointing it makes the system
/// set both schemes in one prompt, so `https` is only asked for when it did
/// not, and asking for it on its own is what the system refuses outright.
fn remaining(held: impl Fn(&str) -> bool) -> Vec<Step> {
    SCHEMES
        .iter()
        .filter(|scheme| !held(scheme))
        .map(|scheme| Step::Scheme(scheme))
        .chain(DOCUMENTS.iter().map(|document| Step::Document(document)))
        .collect()
}

fn is_ours_among(bundle_id: &str, mine: Option<&str>) -> bool {
    mine.is_some_and(|mine| mine.eq_ignore_ascii_case(bundle_id))
}

fn handed_back<T>(remembered: Option<T>, safari: Option<T>) -> Result<T, String> {
    remembered
        .or(safari)
        .ok_or_else(|| "noPreviousBrowser".to_owned())
}

fn walk(app: &NSURL, document_target: impl Fn(&str) -> Retained<NSURL>) -> Result<(), String> {
    point(app, "http")?;
    for step in remaining(|scheme| points_at(app, scheme)) {
        match step {
            Step::Scheme(scheme) => point(app, scheme)?,
            Step::Document(document) => point_documents(&document_target(document), document)?,
        }
    }
    Ok(())
}

pub fn register() -> Result<(), String> {
    objc2::rc::autoreleasepool(|_| {
        let app = ours().ok_or_else(|| "notBundled".to_owned())?;
        remember(handler_for("https").as_deref(), HANDED_FROM);
        for document in DOCUMENTS {
            remember(document_handler(document).as_deref(), &key_for(document));
        }
        walk(&app, |_| app.clone())
    })
}

fn app_remembered_as(key: &str) -> Option<Retained<NSURL>> {
    let id = remembered(key).filter(|id| !is_ours(id))?;
    NSWorkspace::sharedWorkspace().URLForApplicationWithBundleIdentifier(&NSString::from_str(&id))
}

/// The system has no "no default browser", so letting go hands the schemes
/// back to whoever held them before, and to Safari when nobody is remembered.
pub fn unregister() -> Result<(), String> {
    objc2::rc::autoreleasepool(|_| {
        let safari = NSWorkspace::sharedWorkspace()
            .URLForApplicationWithBundleIdentifier(&NSString::from_str(SAFARI));
        let browser = handed_back(app_remembered_as(HANDED_FROM), safari)?;
        walk(&browser, |document| {
            app_remembered_as(&key_for(document)).unwrap_or_else(|| browser.clone())
        })
    })
}

#[cfg(test)]
mod tests {
    use super::{
        SCHEMES, association_report, handler_for, is_bundled, is_default_browser, remember,
        remembered,
    };
    use objc2::rc::Retained;
    use objc2_foundation::{NSString, NSUserDefaults};

    const SUITE: &str = "dev.rgdevment.linkunbound.tests";

    pub(super) fn suite() -> Retained<NSUserDefaults> {
        use objc2::AllocAnyThread;
        NSUserDefaults::initWithSuiteName(NSUserDefaults::alloc(), Some(&NSString::from_str(SUITE)))
            .expect("a suite of our own")
    }

    fn forget(key: &str) {
        suite().removeObjectForKey(&NSString::from_str(key));
        NSUserDefaults::standardUserDefaults()
            .removePersistentDomainForName(&NSString::from_str(SUITE));
    }

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
        let holder = NSWorkspace::sharedWorkspace()
            .URLForApplicationWithBundleIdentifier(&NSString::from_str(
                &handler_for("https").expect("something opens https"),
            ))
            .expect("the handler is installed");
        assert!(super::points_at(&holder, "https"));
        assert!(!super::points_at(&holder, "lu-no-such-scheme"));
        let terminal = objc2_foundation::NSURL::fileURLWithPath(&NSString::from_str(
            "/System/Applications/Utilities/Terminal.app",
        ));
        assert_eq!(
            super::bundle_id_of(&terminal).as_deref(),
            Some("com.apple.Terminal")
        );
        assert!(!super::points_at(&terminal, "https"));
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
        forget(&key);
    }

    #[test]
    fn once_http_is_held_nothing_but_the_documents_remain() {
        use super::Step;
        assert_eq!(
            super::remaining(|_| true),
            [
                Step::Document("public.html"),
                Step::Document("public.xhtml"),
                Step::Document("public.svg-image"),
            ]
        );
        assert_eq!(
            super::remaining(|scheme| scheme == "http"),
            [
                Step::Scheme("https"),
                Step::Document("public.html"),
                Step::Document("public.xhtml"),
                Step::Document("public.svg-image"),
            ],
            "https is asked for only when http did not bring it along"
        );
        assert_eq!(
            super::remaining(|_| false).first(),
            Some(&Step::Scheme("http")),
            "a refusal the system answered with silence is asked again"
        );
    }

    #[test]
    fn links_go_back_to_whoever_had_them_and_to_safari_only_when_nobody_did() {
        assert_eq!(
            super::handed_back(Some("firefox"), Some("safari")),
            Ok("firefox")
        );
        assert_eq!(super::handed_back(None, Some("safari")), Ok("safari"));
        assert_eq!(
            super::handed_back::<&str>(None, None),
            Err("noPreviousBrowser".to_owned())
        );
    }

    #[test]
    fn our_own_identifier_is_never_remembered_as_a_previous_browser() {
        assert!(super::is_ours_among(
            "DEV.rgdevment.LinkUnbound",
            Some("dev.rgdevment.linkunbound")
        ));
        assert!(!super::is_ours_among(
            "com.apple.Safari",
            Some("dev.rgdevment.linkunbound")
        ));
        assert!(!super::is_ours_among("dev.rgdevment.linkunbound", None));
    }

    /// What the bundle declares, what the resident opens and what registration hands over are
    /// three lists that have to agree, or a double-clicked file lands on a copy that refuses it.
    #[test]
    fn every_document_type_is_declared_by_the_bundle_and_opened_by_the_resident() {
        use objc2_uniform_type_identifiers::UTType;
        let plist = include_str!("../../../app/src-tauri/Info.plist");
        for scheme in SCHEMES {
            assert!(
                plist.contains(&format!("<string>{scheme}</string>")),
                "{scheme}"
            );
        }
        for document in super::DOCUMENTS {
            assert!(
                plist.contains(&format!("<string>{document}</string>")),
                "{document}"
            );
            let extension = UTType::typeWithIdentifier(&NSString::from_str(document))
                .and_then(|kind| kind.preferredFilenameExtension())
                .map(|e| e.to_string())
                .expect("a known type");
            assert!(
                linkunbound_core::local_web_file_extensions().contains(&extension.as_str()),
                "{document} is written as .{extension}, which the resident would refuse"
            );
        }
        assert!(
            plist.contains(&format!("<string>{}</string>", crate::OWN_BUNDLE_IDS[0]))
                || include_str!("../../../app/src-tauri/tauri.conf.json")
                    .contains(&format!("\"identifier\": \"{}\"", crate::OWN_BUNDLE_IDS[0])),
            "the identifier the bundle carries is the one this crate treats as its own"
        );
    }

    #[test]
    fn the_browser_that_held_the_links_is_remembered_and_nothing_is_not() {
        let key = format!("HandedFromTest{}", std::process::id());
        remember(None, &key);
        assert!(remembered(&key).is_none());
        remember(Some("com.example.browser"), &key);
        assert_eq!(remembered(&key).as_deref(), Some("com.example.browser"));
        forget(&key);
        assert!(remembered(&key).is_none());
    }
}
