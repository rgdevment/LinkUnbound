use objc2::rc::Retained;
use objc2_app_kit::NSWorkspace;
use objc2_foundation::{NSArray, NSBundle, NSFileManager, NSString, NSURL};

const LEGACY_BUNDLE_ID: &str = "com.rgdevment.linkunbound";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Legacy {
    pub path: String,
    pub version: Option<String>,
}

fn version_of(app: &NSURL) -> Option<String> {
    let bundle = NSBundle::bundleWithURL(app)?;
    let said =
        bundle.objectForInfoDictionaryKey(&NSString::from_str("CFBundleShortVersionString"))?;
    said.downcast::<NSString>().ok().map(|it| it.to_string())
}

fn the_old_copy() -> Option<Retained<NSURL>> {
    let found = NSWorkspace::sharedWorkspace()
        .URLForApplicationWithBundleIdentifier(&NSString::from_str(LEGACY_BUNDLE_ID))?;
    let theirs = found.path()?.to_string();
    let ours = NSBundle::mainBundle()
        .bundleURL()
        .path()
        .map(|it| it.to_string());
    (ours.as_deref() != Some(theirs.as_str())).then_some(found)
}

#[must_use]
pub fn installed() -> Option<Legacy> {
    objc2::rc::autoreleasepool(|_| {
        let app = the_old_copy()?;
        Some(Legacy {
            path: app.path()?.to_string(),
            version: version_of(&app),
        })
    })
}

pub fn retire() -> Result<(), String> {
    objc2::rc::autoreleasepool(|_| {
        let app = the_old_copy().ok_or_else(|| "legacyGone".to_owned())?;
        NSFileManager::defaultManager()
            .trashItemAtURL_resultingItemURL_error(&app, None)
            .map_err(|error| error.localizedDescription().to_string())
    })
}

pub fn reveal() {
    objc2::rc::autoreleasepool(|_| {
        let Some(app) = the_old_copy() else {
            return;
        };
        NSWorkspace::sharedWorkspace()
            .activateFileViewerSelectingURLs(&NSArray::from_retained_slice(&[app]));
    });
}

#[cfg(test)]
mod tests {
    use super::{LEGACY_BUNDLE_ID, installed, retire};

    #[test]
    fn the_old_identifier_is_not_the_one_this_runs_under() {
        assert!(!crate::OWN_BUNDLE_IDS[0].eq_ignore_ascii_case(LEGACY_BUNDLE_ID));
        assert_eq!(crate::OWN_BUNDLE_IDS[1], LEGACY_BUNDLE_ID);
    }

    #[test]
    fn with_no_old_copy_there_is_nothing_to_offer() {
        if installed().is_none() {
            assert_eq!(retire(), Err("legacyGone".to_owned()));
        }
    }
}
