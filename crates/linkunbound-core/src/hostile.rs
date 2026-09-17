use crate::browser::Browser;

/// Switches that make a browser run another binary. The file lives in the
/// user's profile, and this app is what the shell runs for a link.
const RUNS_SOMETHING_ELSE: [&str; 8] = [
    "gpu-launcher",
    "utility-cmd-prefix",
    "renderer-cmd-prefix",
    "browser-subprocess-path",
    "load-extension",
    "disable-extensions-except",
    "remote-debugging-port",
    "headless",
];

/// Chromium reads all three on Windows, so a list that only knew `--` let `/gpu-launcher` and
/// `-gpu-launcher` through to a browser that honours them.
const PREFIXES: [&str; 3] = ["--", "-", "/"];

/// Reaching a network path authenticates the user against whoever holds it.
#[must_use]
pub fn is_remote(path: &str) -> bool {
    let tidy = path.replace('/', "\\");
    tidy.starts_with("\\\\") || tidy.to_ascii_lowercase().starts_with("\\\\?\\unc\\")
}

#[must_use]
pub fn arms_a_launcher(arg: &str) -> bool {
    let (named, value) = arg.split_once('=').unwrap_or((arg, ""));
    let named = named.trim().to_ascii_lowercase();
    // Longest prefix first: `--x` also starts with `-`, and stripping the short one would leave
    // a name with a dash on it that matches nothing.
    PREFIXES
        .iter()
        .find_map(|prefix| named.strip_prefix(prefix))
        .is_some_and(|name| RUNS_SOMETHING_ELSE.contains(&name))
        // `--user-data-dir=\\host\share` makes the browser reach the same host the executable
        // check keeps it away from.
        || is_remote(value.trim().trim_matches('"'))
}

/// Strips rather than rejects: an odd switch costs the switch, not the browser.
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

/// An id with `..` or a UNC prefix would otherwise decide where our own
/// files land.
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

    /// `disarm` answers whether it took anything away, and each place it looks has to be able to
    /// say yes on its own — an `and` in place of an `or` would let one clean field speak for the
    /// rest.
    #[test]
    fn it_reports_a_change_wherever_the_change_was() {
        let mut only_extra = hostile();
        only_extra.profiles[0].args = vec!["--profile-directory=Default".to_owned()];
        only_extra.private_flag = Some("--incognito".to_owned());
        assert!(disarm(&mut only_extra), "the extra args alone");

        let mut only_profile = hostile();
        only_profile.extra_args = vec!["--new-window".to_owned()];
        only_profile.private_flag = Some("--incognito".to_owned());
        assert!(disarm(&mut only_profile), "the profile args alone");

        let mut only_private = hostile();
        only_private.extra_args = vec!["--new-window".to_owned()];
        only_private.profiles[0].args = vec!["--profile-directory=Default".to_owned()];
        only_private.private_flag = Some("--headless".to_owned());
        assert!(disarm(&mut only_private), "the private flag alone");
    }

    /// Letters, digits, dash and underscore each stay; everything else becomes a dash. Reading
    /// that as one condition rather than three would turn every name into dashes.
    #[test]
    fn a_name_keeps_what_is_safe_to_keep() {
        assert_eq!(as_file_name("chrome-2_beta"), "chrome-2_beta");
        assert_eq!(as_file_name("Edge9"), "Edge9");
        assert_eq!(as_file_name("a.b c"), "a-b-c");
    }
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

    /// The executable is kept off the network, and so is anything the browser is told to read or
    /// write there: a profile or a cache on `\\host\share` reaches that host all the same.
    #[test]
    fn a_switch_pointing_the_browser_at_another_machine_is_taken_away() {
        for said in [
            r"--user-data-dir=\\evil\share\profile",
            r#"--disk-cache-dir="\\evil\share\cache""#,
            "--user-data-dir=//evil/share/profile",
        ] {
            assert!(arms_a_launcher(said), "{said}");
        }
        assert!(!arms_a_launcher(r"--user-data-dir=C:\Users\ana\profile"));
        assert!(!arms_a_launcher("--profile-directory=Profile 1"));
    }

    /// Chromium takes a switch with any of the three prefixes on Windows, so a list that only
    /// knew the long one was a list the browser did not agree with.
    #[test]
    fn a_switch_is_recognised_however_it_is_prefixed() {
        for said in [
            "/gpu-launcher=calc.exe",
            "-gpu-launcher=calc.exe",
            "--gpu-launcher=calc.exe",
            "/RENDERER-CMD-PREFIX=calc.exe",
        ] {
            assert!(arms_a_launcher(said), "{said}");
        }
        for said in ["gpu-launcher=calc.exe", "--incognito", "/new-window", "-P"] {
            assert!(!arms_a_launcher(said), "{said}");
        }
    }

    /// The line a person types is split before it is judged, so what the filter sees has to be
    /// what the browser would see — quotes and all.
    #[test]
    fn a_hostile_switch_does_not_survive_being_quoted() {
        for line in [
            r#""--gpu-launcher=calc.exe""#,
            r#""/gpu-launcher"="calc.exe""#,
            r#"--gpu-launcher" "=calc.exe"#,
            r#"" --gpu-launcher=calc.exe""#,
        ] {
            let said = crate::split_args(line);
            assert!(
                said.iter().any(|one| arms_a_launcher(one)),
                "{line} became {said:?}"
            );
        }
    }
}
