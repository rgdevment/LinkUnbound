use crate::browser::Browser;

/// Switches that make a browser run another binary. A `browsers.json` is an
/// ordinary file in the user's profile: anything that can write there would
/// otherwise own every click, because this app is what the shell launches for
/// a link and it launches whatever the file says.
const RUNS_SOMETHING_ELSE: [&str; 8] = [
    "--gpu-launcher",
    "--utility-cmd-prefix",
    "--renderer-cmd-prefix",
    "--browser-subprocess-path",
    "--load-extension",
    "--disable-extensions-except",
    "--remote-debugging-port",
    "--headless",
];

/// A path served over the network authenticates the user against whoever holds
/// it. Reaching one by accident hands over an NTLM hash; reaching one because a
/// config file said so hands it over on purpose.
#[must_use]
pub fn is_remote(path: &str) -> bool {
    let tidy = path.replace('/', "\\");
    tidy.starts_with("\\\\") || tidy.to_ascii_lowercase().starts_with("\\\\?\\unc\\")
}

#[must_use]
pub fn arms_a_launcher(arg: &str) -> bool {
    let named = arg
        .split('=')
        .next()
        .unwrap_or(arg)
        .trim()
        .to_ascii_lowercase();
    RUNS_SOMETHING_ELSE.contains(&named.as_str())
}

/// Strips what a browser entry must not carry, rather than dropping the entry:
/// a user who really did type an odd switch keeps their browser, and loses only
/// the part that could run something else.
pub fn disarm(browser: &mut Browser) -> bool {
    let mut touched = false;

    if is_remote(&browser.exe) {
        browser.exe = String::new();
        touched = true;
    }
    if browser.icon_path.as_deref().is_some_and(is_remote) {
        browser.icon_path = None;
        touched = true;
    }

    let before = browser.extra_args.len();
    browser.extra_args.retain(|arg| !arms_a_launcher(arg));
    touched |= browser.extra_args.len() != before;

    for profile in &mut browser.profiles {
        let before = profile.args.len();
        profile.args.retain(|arg| !arms_a_launcher(arg));
        touched |= profile.args.len() != before;
    }

    if browser.private_flag.as_deref().is_some_and(arms_a_launcher) {
        browser.private_flag = None;
        touched = true;
    }
    touched
}

/// Everything a browser id ends up naming is a file we create. Left alone, an
/// id of `..\..\x` or `\\host\share\x` decides where that file goes.
#[must_use]
pub fn as_file_name(id: &str) -> String {
    let tidy: String = id
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '-'
            }
        })
        .collect();
    if tidy.is_empty() {
        "unnamed".to_owned()
    } else {
        tidy
    }
}

#[cfg(test)]
mod tests {
    use super::{arms_a_launcher, as_file_name, disarm, is_remote};
    use crate::browser::{Browser, Profile};

    fn hostile() -> Browser {
        Browser {
            id: "chrome".to_owned(),
            name: "Google Chrome".to_owned(),
            exe: "chrome.exe".to_owned(),
            profiles: vec![Profile {
                id: "Default".to_owned(),
                name: "Personal".to_owned(),
                args: vec![
                    "--profile-directory=Default".to_owned(),
                    "--renderer-cmd-prefix=calc.exe".to_owned(),
                ],
            }],
            extra_args: vec![
                "--new-window".to_owned(),
                "--gpu-launcher=calc.exe".to_owned(),
            ],
            private_flag: Some("--incognito".to_owned()),
            icon_path: None,
            custom: false,
            hidden: false,
        }
    }

    /// The picker is what the shell runs for every link, so a switch that runs
    /// another binary turns one edited file into execution on the next click.
    #[test]
    fn a_switch_that_runs_another_binary_is_taken_away() {
        let mut browser = hostile();
        assert!(disarm(&mut browser));

        assert_eq!(browser.extra_args, vec!["--new-window".to_owned()]);
        assert_eq!(
            browser.profiles[0].args,
            vec!["--profile-directory=Default".to_owned()],
            "the profile keeps working, it just cannot launch anything"
        );
        assert_eq!(browser.private_flag.as_deref(), Some("--incognito"));
    }

    #[test]
    fn an_ordinary_browser_is_left_exactly_as_it_was() {
        let mut browser = hostile();
        browser.extra_args = vec!["--new-window".to_owned()];
        browser.profiles[0].args = vec!["--profile-directory=Default".to_owned()];
        let before = browser.clone();

        assert!(!disarm(&mut browser));
        assert_eq!(browser, before);
    }

    #[test]
    fn a_path_on_another_machine_is_refused_however_it_is_spelled() {
        for path in [
            r"\\attacker.test\share\evil.exe",
            r"//attacker.test/share/evil.exe",
            r"\\?\UNC\attacker.test\share\evil.exe",
        ] {
            assert!(is_remote(path), "{path}");
        }
        for path in [r"C:\Program Files\chrome.exe", "chrome.exe", "/usr/bin/x"] {
            assert!(!is_remote(path), "{path}");
        }
    }

    /// The icon path is read by the shell, so a remote one fetches over SMB and
    /// authenticates on the way. It is also the cheapest field to abuse: no
    /// traversal and no execution, just a string.
    #[test]
    fn a_remote_icon_is_dropped_rather_than_fetched() {
        let mut browser = hostile();
        browser.icon_path = Some(r"\\attacker.test\share\icon.ico".to_owned());
        assert!(disarm(&mut browser));
        assert!(browser.icon_path.is_none());
    }

    #[test]
    fn an_id_cannot_decide_where_a_file_lands() {
        assert_eq!(as_file_name(r"..\..\escaped"), "------escaped");
        assert_eq!(
            as_file_name(r"\\attacker.test\share\x"),
            "--attacker-test-share-x"
        );
        assert_eq!(as_file_name("custom-2"), "custom-2");
        assert_eq!(as_file_name("chrome"), "chrome");
        assert_eq!(as_file_name(""), "unnamed");
    }

    /// The whole attack, end to end: an edited file reaching the argv that a
    /// click would run. This is the boundary, so it is tested from the JSON.
    #[test]
    fn an_edited_file_cannot_put_another_binary_in_the_argv() {
        let raw = r#"{
            "schema_version": 2,
            "browsers": [{
                "id": "chrome",
                "name": "Google Chrome",
                "exe": "chrome.exe",
                "extraArgs": ["--gpu-launcher=calc.exe"],
                "extra_args": ["--gpu-launcher=calc.exe"],
                "custom": true
            }]
        }"#;

        let config = crate::config::read_browsers(raw).expect("should read");
        let argv = config.browsers[0]
            .launch(None, false, "https://github.com/a")
            .expect("should launch");

        assert!(
            !argv.iter().any(|a| a.contains("gpu-launcher")),
            "the switch must not survive into what gets run: {argv:?}"
        );
        assert_eq!(argv, vec!["https://github.com/a".to_owned()]);
    }

    #[test]
    fn a_switch_is_recognised_with_or_without_its_value() {
        assert!(arms_a_launcher("--gpu-launcher=calc.exe"));
        assert!(arms_a_launcher("--gpu-launcher"));
        assert!(arms_a_launcher("--GPU-Launcher=calc.exe"));
        assert!(!arms_a_launcher("--incognito"));
        assert!(!arms_a_launcher("--profile-directory=Default"));
    }
}
