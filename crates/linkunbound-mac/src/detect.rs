use std::path::{Path, PathBuf};

use linkunbound_core::{Browser, Profile, id_for, private_flag_for, profiles_in};
use objc2::Message;
use objc2_app_kit::NSWorkspace;
use objc2_foundation::{NSArray, NSBundle, NSDictionary, NSString, NSURL};

/// Asked of a link rather than of a scheme: `LSCopyAllHandlersForURLScheme` is
/// the deprecated half of this pair and answers the same question.
const PROBE: &str = "https://example.com";

const WEB_SCHEMES: [&str; 2] = ["http", "https"];

/// Apple's own word for "I can open one of these, but opening them is not what
/// I am": terminals and editors register that way and are not destinations.
const NOT_A_BROWSER: [&str; 2] = ["alternate", "none"];

fn is_destination(bundle_id: &str, web_ranks: &[String]) -> bool {
    if crate::is_one_of_ours(bundle_id) {
        return false;
    }
    // A bundle whose declaration cannot be read is left to Launch Services,
    // which already answered that it opens links: dropping it there would take
    // somebody's browser out of the picker over a plist this could not parse.
    web_ranks.is_empty()
        || web_ranks
            .iter()
            .any(|rank| !NOT_A_BROWSER.contains(&rank.to_ascii_lowercase().as_str()))
}

fn text(value: Option<objc2::rc::Retained<NSString>>) -> Option<String> {
    value.map(|s| s.to_string()).filter(|s| !s.is_empty())
}

fn info_string(bundle: &NSBundle, key: &str) -> Option<String> {
    let value = bundle.objectForInfoDictionaryKey(&NSString::from_str(key))?;
    text(value.downcast::<NSString>().ok())
}

/// The Finder name, which is what 1.x stored and what the picker has always
/// shown. The bundle's own file name is the last resort.
fn name_of(bundle: &NSBundle, app: &Path) -> Option<String> {
    info_string(bundle, "CFBundleDisplayName")
        .or_else(|| info_string(bundle, "CFBundleName"))
        .or_else(|| app.file_stem().map(|s| s.to_string_lossy().into_owned()))
        .filter(|name| !name.is_empty())
}

fn strings_in(array: &NSArray) -> Vec<String> {
    array
        .iter()
        .filter_map(|value| text(value.downcast::<NSString>().ok()))
        .collect()
}

fn names_a_web_scheme(entry: &NSDictionary) -> bool {
    entry
        .objectForKey(&NSString::from_str("CFBundleURLSchemes"))
        .and_then(|schemes| schemes.downcast::<NSArray>().ok())
        .is_some_and(|schemes| {
            strings_in(&schemes)
                .iter()
                .any(|scheme| WEB_SCHEMES.contains(&scheme.to_ascii_lowercase().as_str()))
        })
}

/// An entry that names no rank is `Default`, which is what Apple documents and
/// what every browser here relies on.
fn rank_of(entry: &NSDictionary) -> String {
    entry
        .objectForKey(&NSString::from_str("LSHandlerRank"))
        .and_then(|rank| text(rank.downcast::<NSString>().ok()))
        .unwrap_or_else(|| "Default".to_owned())
}

fn web_ranks(bundle: &NSBundle) -> Vec<String> {
    let Some(info) = bundle.infoDictionary() else {
        return Vec::new();
    };
    let Some(types) = info.objectForKey(&NSString::from_str("CFBundleURLTypes")) else {
        return Vec::new();
    };
    let Ok(types) = types.downcast::<NSArray>() else {
        return Vec::new();
    };
    types
        .iter()
        .filter_map(|entry| entry.downcast::<NSDictionary>().ok())
        .filter(|entry| names_a_web_scheme(entry))
        .map(|entry| rank_of(&entry))
        .collect()
}

/// Chromium keeps its profiles beside the browser's own support directory, with
/// no `User Data` level in between as on Windows.
fn user_data_dir(app: &Path) -> Option<PathBuf> {
    let needle = app.to_string_lossy().to_ascii_lowercase();
    let suffix = if needle.contains("microsoft edge") {
        "Microsoft Edge"
    } else if needle.contains("brave") {
        "BraveSoftware/Brave-Browser"
    } else if needle.contains("vivaldi") {
        "Vivaldi"
    } else if needle.contains("chrome canary") {
        "Google/Chrome Canary"
    } else if needle.contains("chrome beta") {
        "Google/Chrome Beta"
    } else if needle.contains("chrome dev") {
        "Google/Chrome Dev"
    } else if needle.contains("chrome") {
        "Google/Chrome"
    } else if needle.contains("chromium") {
        "Chromium"
    } else if needle.contains("/arc.app") {
        "Arc/User Data"
    } else if needle.contains("opera gx") {
        "com.operasoftware.OperaGX"
    } else if needle.contains("opera") {
        "com.operasoftware.Opera"
    } else {
        return None;
    };
    Some(
        PathBuf::from(std::env::var_os("HOME")?)
            .join("Library")
            .join("Application Support")
            .join(suffix),
    )
}

#[must_use]
pub fn chromium_profiles(app: &str) -> Vec<Profile> {
    let Some(dir) = user_data_dir(Path::new(app)) else {
        return Vec::new();
    };
    let Ok(raw) = std::fs::read_to_string(dir.join("Local State")) else {
        return Vec::new();
    };
    profiles_in(&raw)
}

fn preferred(bundle_id: &str) -> Option<objc2::rc::Retained<NSURL>> {
    NSWorkspace::sharedWorkspace()
        .URLForApplicationWithBundleIdentifier(&NSString::from_str(bundle_id))
}

fn read_bundle(found: &NSURL) -> Option<Browser> {
    let bundle_id = text(NSBundle::bundleWithURL(found)?.bundleIdentifier())?;
    let url = preferred(&bundle_id).unwrap_or_else(|| found.retain());
    let path = text(url.path())?;
    let bundle = NSBundle::bundleWithURL(&url)?;
    if !is_destination(&bundle_id, &web_ranks(&bundle)) {
        return None;
    }
    let app = Path::new(&path);
    Some(Browser {
        id: id_for(&bundle_id),
        name: name_of(&bundle, app)?,
        private_flag: private_flag_for(&path).map(str::to_owned),
        profiles: chromium_profiles(&path),
        exe: path,
        extra_args: Vec::new(),
        icon_path: None,
        custom: false,
        hidden: false,
    })
}

fn add_unseen(found: &mut Vec<Browser>, browser: Browser) {
    if !found.iter().any(|b| b.id == browser.id) {
        found.push(browser);
    }
}

#[must_use]
pub fn installed_browsers() -> Vec<Browser> {
    let Some(probe) = NSURL::URLWithString(&NSString::from_str(PROBE)) else {
        return Vec::new();
    };
    let mut found: Vec<Browser> = Vec::new();
    for url in NSWorkspace::sharedWorkspace().URLsForApplicationsToOpenURL(&probe) {
        if let Some(browser) = read_bundle(&url) {
            add_unseen(&mut found, browser);
        }
    }
    found.sort_by_key(|b| b.name.to_lowercase());
    found
}

#[cfg(test)]
mod tests {
    use super::{
        add_unseen, chromium_profiles, info_string, installed_browsers, is_destination,
        read_bundle, user_data_dir, web_ranks,
    };
    use linkunbound_core::Browser;
    use objc2_foundation::NSBundle;
    use std::path::Path;

    fn safari() -> objc2::rc::Retained<objc2_foundation::NSURL> {
        super::preferred("com.apple.Safari").expect("Safari is always there")
    }

    fn one(id: &str) -> Browser {
        Browser {
            id: id.to_owned(),
            name: id.to_owned(),
            exe: format!("/Applications/{id}.app"),
            profiles: Vec::new(),
            extra_args: Vec::new(),
            private_flag: None,
            icon_path: None,
            custom: false,
            hidden: false,
        }
    }

    fn ranked(ranks: &[&str]) -> Vec<String> {
        ranks.iter().map(|r| (*r).to_owned()).collect()
    }

    /// We register for the same schemes, so without this the picker offers itself as a
    /// destination and a link handed to it comes straight back. The 1.x identifier is refused
    /// too: an old copy still on the machine is no more a destination than this one.
    #[test]
    fn neither_this_copy_nor_the_one_1_x_installed_is_a_destination() {
        let browser = ranked(&["Default"]);
        assert!(is_destination("com.apple.Safari", &browser));
        assert!(is_destination("org.mozilla.firefox", &browser));
        assert!(!is_destination("dev.rgdevment.linkunbound", &browser));
        assert!(!is_destination("DEV.RGDEVMENT.LINKUNBOUND", &browser));
        assert!(!is_destination("com.rgdevment.linkunbound", &browser));
    }

    /// Launch Services answers with everything that *can* open a link, which on this machine
    /// meant a terminal in the picker. `Alternate` is how those apps say so themselves.
    #[test]
    fn an_app_that_merely_accepts_links_is_not_offered_as_a_browser() {
        assert!(!is_destination(
            "com.googlecode.iterm2",
            &ranked(&["Alternate"])
        ));
        assert!(!is_destination("com.example.editor", &ranked(&["None"])));
        assert!(!is_destination(
            "com.example.editor",
            &ranked(&["alternate"])
        ));
        assert!(is_destination("com.apple.Safari", &ranked(&["Default"])));
        assert!(is_destination("com.example.browser", &ranked(&["Owner"])));
    }

    /// Firefox declares one entry per scheme, and a browser that ranks itself plainly for one
    /// of them is a destination whatever it says about the other.
    #[test]
    fn one_entry_that_claims_the_web_plainly_is_enough() {
        assert!(is_destination(
            "org.mozilla.firefox",
            &ranked(&["Alternate", "Default"])
        ));
    }

    /// Dropping a browser over a plist this could not parse takes somebody's browser out of
    /// the picker; Launch Services already answered that it opens links.
    #[test]
    fn a_bundle_that_declares_nothing_readable_is_left_to_launch_services() {
        assert!(is_destination("com.example.browser", &[]));
    }

    /// The path carries no `User Data` level here, and looking for the Windows one found
    /// nothing: every Chromium browser reported a single nameless profile.
    #[test]
    fn each_chromium_family_is_looked_for_where_it_actually_keeps_its_profiles() {
        let of = |app: &str| {
            user_data_dir(Path::new(app))
                .map(|d| d.to_string_lossy().into_owned())
                .unwrap_or_default()
        };
        assert!(
            of("/Applications/Google Chrome.app").ends_with("Application Support/Google/Chrome")
        );
        assert!(
            of("/Applications/Microsoft Edge.app").ends_with("Application Support/Microsoft Edge")
        );
        assert!(
            of("/Applications/Brave Browser.app")
                .ends_with("Application Support/BraveSoftware/Brave-Browser")
        );
        assert!(of("/Applications/Vivaldi.app").ends_with("Application Support/Vivaldi"));
    }

    #[test]
    fn each_chrome_channel_keeps_its_own_profiles() {
        let of = |app: &str| {
            user_data_dir(Path::new(app))
                .map(|d| d.to_string_lossy().into_owned())
                .unwrap_or_default()
        };
        assert!(of("/Applications/Google Chrome Canary.app").ends_with("Google/Chrome Canary"));
        assert!(of("/Applications/Google Chrome Beta.app").ends_with("Google/Chrome Beta"));
        assert!(of("/Applications/Google Chrome Dev.app").ends_with("Google/Chrome Dev"));
        assert!(of("/Applications/Google Chrome.app").ends_with("Google/Chrome"));
    }

    #[test]
    fn the_other_chromium_families_keep_their_profiles_where_they_do() {
        let of = |app: &str| {
            user_data_dir(Path::new(app))
                .map(|d| d.to_string_lossy().into_owned())
                .unwrap_or_default()
        };
        assert!(of("/Applications/Chromium.app").ends_with("Application Support/Chromium"));
        assert!(of("/Applications/Arc.app").ends_with("Application Support/Arc/User Data"));
        assert!(of("/Applications/Opera.app").ends_with("com.operasoftware.Opera"));
        assert!(of("/Applications/Opera GX.app").ends_with("com.operasoftware.OperaGX"));
        assert!(
            of("/Users/marc/Applications/Firefox.app").is_empty(),
            "a name is not a path"
        );
    }

    /// Edge is Chromium but keeps its profiles somewhere of its own, and the generic marker
    /// would send it to Chrome's directory: the profiles shown would be another browser's.
    #[test]
    fn edge_is_not_sent_to_the_directory_chrome_keeps() {
        let edge = user_data_dir(Path::new("/Applications/Microsoft Edge.app")).expect("a place");
        assert!(!edge.to_string_lossy().contains("Google"));
    }

    #[test]
    fn a_browser_outside_the_chromium_families_reports_no_profiles() {
        assert!(chromium_profiles("/Applications/Firefox.app").is_empty());
        assert!(chromium_profiles("/Applications/Safari.app").is_empty());
    }

    #[test]
    fn a_real_bundle_reads_back_its_name_and_the_rank_it_claims_for_the_web() {
        use objc2_foundation::{NSBundle, NSString};
        let safari = super::preferred("com.apple.Safari").expect("Safari is always there");
        let bundle = NSBundle::bundleWithURL(&safari).expect("a bundle");
        let path = std::path::PathBuf::from(safari.path().expect("a path").to_string());
        assert_eq!(super::name_of(&bundle, &path).as_deref(), Some("Safari"));
        let ranks = super::web_ranks(&bundle);
        assert!(!ranks.is_empty(), "Safari declares http and https");
        assert!(ranks.iter().all(|r| !r.is_empty()));
        assert!(super::is_destination("com.apple.Safari", &ranks));
        assert_eq!(super::text(Some(NSString::from_str(""))), None);
        assert_eq!(
            super::text(Some(NSString::from_str("x"))).as_deref(),
            Some("x")
        );
        assert_eq!(super::text(None), None);
        let ours = super::preferred("dev.rgdevment.linkunbound");
        if let Some(ours) = ours {
            assert!(
                super::read_bundle(&ours).is_none(),
                "this app is never a destination"
            );
        }
    }

    #[test]
    fn a_browser_is_named_by_the_copy_the_system_would_launch() {
        let safari = super::preferred("com.apple.Safari").expect("Safari is always there");
        let path = safari.path().expect("a path").to_string();
        assert!(path.ends_with("Safari.app"), "{path}");
        assert!(super::preferred("com.example.nothing.installed").is_none());
    }

    #[test]
    fn safari_declares_itself_a_browser_and_only_for_the_web() {
        let bundle = NSBundle::bundleWithURL(&safari()).expect("a bundle");
        assert_eq!(
            info_string(&bundle, "CFBundleIdentifier").as_deref(),
            Some("com.apple.Safari")
        );
        assert_eq!(info_string(&bundle, "LinkUnboundNeverWroteThis"), None);
        assert_eq!(web_ranks(&bundle), ["Default"]);
        assert!(
            web_ranks(&NSBundle::mainBundle()).is_empty(),
            "a test binary declares nothing"
        );
    }

    #[test]
    fn safari_is_read_as_the_browser_it_is() {
        let read = read_bundle(&safari()).expect("a browser");
        assert_eq!(read.id, "com-apple-safari");
        assert_eq!(read.name, "Safari");
        assert!(read.exe.ends_with("Safari.app"), "{}", read.exe);
        assert!(read.profiles.is_empty());
        assert!(read.private_flag.is_none());
    }

    #[test]
    fn a_browser_listed_twice_by_launch_services_is_offered_once() {
        let mut found = Vec::new();
        add_unseen(&mut found, one("com-apple-safari"));
        add_unseen(&mut found, one("com-apple-safari"));
        add_unseen(&mut found, one("org-mozilla-firefox"));
        let ids: Vec<&str> = found.iter().map(|b| b.id.as_str()).collect();
        assert_eq!(ids, ["com-apple-safari", "org-mozilla-firefox"]);
    }

    #[test]
    fn whatever_is_installed_carries_a_launchable_path() {
        let installed = installed_browsers();
        assert!(
            installed.iter().any(|b| b.id == "com-apple-safari"),
            "Safari is always there"
        );
        for browser in installed {
            assert!(!browser.exe.is_empty());
            assert!(!browser.id.is_empty());
            assert!(!browser.name.is_empty());
            assert!(
                Path::new(&browser.exe).exists(),
                "{} names nothing on disk",
                browser.exe
            );
            for profile in &browser.profiles {
                assert!(
                    profile
                        .args
                        .iter()
                        .any(|a| a.starts_with("--profile-directory="))
                );
            }
        }
    }
}
