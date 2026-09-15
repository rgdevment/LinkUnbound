use crate::{Browser, LaunchRefused};

#[derive(Debug, thiserror::Error)]
pub enum LaunchError {
    #[error("no browser is registered under {0}")]
    UnknownBrowser(String),
    #[error(transparent)]
    Refused(#[from] LaunchRefused),
    #[error("the browser would not start: {0}")]
    Spawn(#[from] std::io::Error),
}

/// The URL is never taken from the webview: it comes from the state the core
/// already validated, so the argument-injection guard stays on the path.
pub fn open(
    browsers: &[Browser],
    browser_id: &str,
    profile_id: Option<&str>,
    private: bool,
    url: &str,
) -> Result<(), LaunchError> {
    let browser = browsers
        .iter()
        .find(|b| b.id == browser_id)
        .ok_or_else(|| LaunchError::UnknownBrowser(browser_id.to_owned()))?;

    let args = browser.launch(profile_id, private, url)?;
    spawn(&browser.exe, &args)?;
    Ok(())
}

/// A child nobody waits for stays a zombie on Unix for as long as this process lives.
pub fn spawn_and_forget(command: &mut std::process::Command) -> Result<(), std::io::Error> {
    let child = command.spawn()?;
    forget(child);
    Ok(())
}

#[cfg(unix)]
fn forget(mut child: std::process::Child) {
    std::thread::spawn(move || {
        let _ = child.wait();
    });
}

#[cfg(not(unix))]
fn forget(child: std::process::Child) {
    drop(child);
}

#[cfg(not(target_os = "macos"))]
fn spawn(exe: &str, args: &[String]) -> Result<(), std::io::Error> {
    spawn_and_forget(std::process::Command::new(exe).args(args))
}

#[cfg(target_os = "macos")]
use mac::spawn;

#[cfg(target_os = "macos")]
mod mac {
    use std::io;
    use std::path::{Path, PathBuf};
    use std::process::Command;

    #[derive(Debug, PartialEq, Eq)]
    pub enum How {
        Itself,
        Bundle,
        Inside,
    }

    /// `open` drops everything after `--args` when the app is already running, so
    /// a link carrying a switch has to reach the executable directly. A link
    /// carrying none goes through `open`, which is what puts the browser in front
    /// and what reaches Safari at all.
    pub fn how(app: &Path, args: &[String]) -> How {
        if !app.is_dir() {
            return How::Itself;
        }
        if args.len() > 1 {
            How::Inside
        } else {
            How::Bundle
        }
    }

    /// Firefox keeps twenty other files beside it in there and spells itself in
    /// lower case, so the bundle's own name is not the executable's.
    pub fn inside(app: &Path) -> Result<PathBuf, io::Error> {
        let info = plist::Value::from_file(app.join("Contents").join("Info.plist"))
            .map_err(|why| io::Error::new(io::ErrorKind::InvalidData, why.to_string()))?;
        let named = info
            .as_dictionary()
            .and_then(|d| d.get("CFBundleExecutable"))
            .and_then(plist::Value::as_string)
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("{} names no executable", app.display()),
                )
            })?;
        Ok(app.join("Contents").join("MacOS").join(named))
    }

    pub fn spawn(exe: &str, args: &[String]) -> Result<(), io::Error> {
        let app = Path::new(exe);
        // `open` reports a missing app on its own exit code, which nothing here
        // waits for: the failure would reach the picker as success.
        if !app.exists() {
            return Err(io::Error::new(io::ErrorKind::NotFound, exe.to_owned()));
        }
        match how(app, args) {
            How::Itself => super::spawn_and_forget(Command::new(app).args(args)),
            How::Bundle => {
                super::spawn_and_forget(Command::new("/usr/bin/open").arg("-a").arg(app).args(args))
            }
            How::Inside => super::spawn_and_forget(Command::new(inside(app)?).args(args)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Profile;

    fn catalogue() -> Vec<Browser> {
        vec![Browser {
            id: "chrome".to_owned(),
            name: "Chrome".to_owned(),
            exe: "definitely-not-a-real-binary".to_owned(),
            profiles: vec![Profile {
                id: "Profile 2".to_owned(),
                name: "Work".to_owned(),
                args: vec!["--profile-directory=Profile 2".to_owned()],
            }],
            private_flag: Some("--incognito".to_owned()),
            extra_args: Vec::new(),
            icon_path: None,
            custom: false,
            hidden: false,
        }]
    }

    #[test]
    fn a_browser_that_is_not_in_the_catalogue_is_refused_by_name() {
        let err = open(&catalogue(), "ghost", None, false, "https://a.test").unwrap_err();
        assert!(matches!(err, LaunchError::UnknownBrowser(id) if id == "ghost"));
    }

    #[test]
    fn a_promise_that_cannot_be_kept_never_reaches_the_process() {
        let err = open(
            &catalogue(),
            "chrome",
            Some("gone"),
            false,
            "https://a.test",
        )
        .unwrap_err();
        assert!(matches!(
            err,
            LaunchError::Refused(LaunchRefused::ProfileGone(_))
        ));
    }

    /// A path that names nothing has to fail here rather than at the browser, which nothing
    /// waits for: `open` would exit non-zero into a void and the picker would report success.
    #[test]
    fn a_browser_whose_path_names_nothing_is_reported_rather_than_handed_over() {
        let err = open(&catalogue(), "chrome", None, false, "https://a.test").unwrap_err();
        assert!(matches!(err, LaunchError::Spawn(_)));
    }

    #[cfg(unix)]
    #[test]
    fn a_child_that_has_exited_is_not_left_as_a_zombie() {
        use std::process::Command;
        for _ in 0..3 {
            spawn_and_forget(&mut Command::new("/usr/bin/true")).expect("true runs");
        }
        std::thread::sleep(std::time::Duration::from_millis(300));

        let me = std::process::id().to_string();
        let listed = Command::new("ps")
            .args(["-e", "-o", "ppid=,stat="])
            .output()
            .expect("ps runs");
        let zombies = String::from_utf8_lossy(&listed.stdout)
            .lines()
            .filter(|line| {
                let mut parts = line.split_whitespace();
                parts.next() == Some(me.as_str()) && parts.next().is_some_and(|s| s.contains('Z'))
            })
            .count();
        assert_eq!(zombies, 0);
    }

    #[cfg(target_os = "macos")]
    mod macos {
        use super::super::mac::{How, how, inside};
        use std::path::Path;

        fn args(list: &[&str]) -> Vec<String> {
            list.iter().map(|a| (*a).to_owned()).collect()
        }

        /// The bundle is a directory, so handing it to a process is handing over a directory.
        /// A link with nothing but the URL goes through `open`, which is the only way Safari
        /// receives one at all and what brings the browser to the front.
        #[test]
        fn a_bundle_with_nothing_but_a_link_is_opened_the_way_the_system_opens_one() {
            let app = Path::new("/Applications");
            assert_eq!(how(app, &args(&["https://a.test"])), How::Bundle);
        }

        /// `open` drops everything past `--args` when the app is already running, and the
        /// link would never arrive: a private window or a profile has to reach the executable.
        #[test]
        fn a_link_carrying_a_switch_reaches_the_executable_instead() {
            let app = Path::new("/Applications");
            assert_eq!(
                how(app, &args(&["--incognito", "https://a.test"])),
                How::Inside
            );
            assert_eq!(
                how(
                    app,
                    &args(&["--profile-directory=Profile 2", "https://a.test"])
                ),
                How::Inside
            );
        }

        /// Somebody who added a browser by hand named a program, not a bundle.
        #[test]
        fn a_path_that_is_already_a_program_is_run_as_one() {
            assert_eq!(
                how(Path::new("/bin/echo"), &args(&["https://a.test"])),
                How::Itself
            );
        }

        /// The bundle's own name is not the executable's: Firefox spells itself in lower case
        /// and keeps twenty other files beside it, so a guess from the folder name misses.
        #[test]
        fn the_executable_is_read_from_the_bundle_rather_than_guessed_from_its_name() {
            let firefox = Path::new("/Applications/Firefox.app");
            if !firefox.exists() {
                return;
            }
            let found = inside(firefox).expect("Firefox names its executable");
            assert!(found.is_file(), "{found:?}");
            assert_eq!(found.file_name().and_then(|n| n.to_str()), Some("firefox"));
        }

        #[test]
        fn a_bundle_that_is_not_one_is_reported_rather_than_launched() {
            assert!(inside(Path::new("/Applications")).is_err());
        }
    }
}
