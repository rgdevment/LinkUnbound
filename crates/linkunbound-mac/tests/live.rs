#![cfg(target_os = "macos")]

use linkunbound_mac::installed_browsers;

/// Run with `cargo test -p linkunbound-mac -- --ignored --nocapture` to see
/// what this machine actually has installed.
#[test]
#[ignore = "reports the local machine rather than asserting"]
fn list_what_is_installed() {
    for browser in installed_browsers() {
        println!("{} [{}] -> {}", browser.name, browser.id, browser.exe);
        println!("  private: {:?}", browser.private_flag);
        for profile in &browser.profiles {
            println!("  profile: {} ({})", profile.name, profile.id);
        }
    }
}

/// The one thing no unit test reaches: a link actually leaving for a browser. Both routes
/// are exercised — `open` for a link carrying nothing, the executable inside the bundle for
/// one carrying a switch — because only the second is what a private window or a profile
/// takes, and `open` drops those silently when the browser is already running.
///
/// Run with `cargo test -p linkunbound-mac -- --ignored live`. It opens real windows.
#[test]
#[ignore = "opens browser windows on this machine"]
fn a_link_reaches_a_browser_by_both_routes() {
    let browsers = installed_browsers();
    assert!(!browsers.is_empty(), "nothing to launch into");

    for browser in &browsers {
        let sent = linkunbound_core::launch(
            &browsers,
            &browser.id,
            None,
            false,
            "https://example.com/plain",
        );
        assert!(sent.is_ok(), "{}: {sent:?}", browser.name);
        println!("open        -> {} [{}]", browser.name, browser.id);
    }

    for browser in browsers.iter().filter(|b| b.supports_private()) {
        let sent = linkunbound_core::launch(
            &browsers,
            &browser.id,
            None,
            true,
            "https://example.com/private",
        );
        assert!(sent.is_ok(), "{}: {sent:?}", browser.name);
        println!(
            "inside ({}) -> {} [{}]",
            browser.private_flag.as_deref().unwrap_or(""),
            browser.name,
            browser.id
        );
    }
}
