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
    use std::path::Path;
    use std::process::Command;

    #[derive(Debug, PartialEq, Eq)]
    pub enum How {
        Itself,
        Bundle,
        Instance,
    }

    /// `open` drops everything after `--args` when the app is already running,
    /// but asked for an instance of its own it hands the whole command line to
    /// the running copy, which Chromium and Firefox both take and then leave.
    /// Either way the browser is started by Launch Services, so it answers for
    /// its own permissions rather than for ours, and Safari's launch
    /// constraints are never tripped by running its executable by hand.
    pub fn how(app: &Path, args: &[String]) -> How {
        if !app.is_dir() {
            return How::Itself;
        }
        if args.len() > 1 {
            How::Instance
        } else {
            How::Bundle
        }
    }

    pub fn command_for(app: &Path, args: &[String]) -> Command {
        match how(app, args) {
            How::Itself => {
                let mut run = Command::new(app);
                run.args(args);
                run
            }
            How::Bundle => {
                let mut run = Command::new("/usr/bin/open");
                run.arg("-a").arg(app).args(args);
                run
            }
            How::Instance => {
                let mut run = Command::new("/usr/bin/open");
                run.arg("-n").arg("-a").arg(app).arg("--args").args(args);
                run
            }
        }
    }

    pub fn spawn(exe: &str, args: &[String]) -> Result<(), io::Error> {
        let app = Path::new(exe);
        // `open` reports a missing app on its own exit code, which nothing here
        // waits for: the failure would reach the picker as success.
        if !app.exists() {
            return Err(io::Error::new(io::ErrorKind::NotFound, exe.to_owned()));
        }
        super::spawn_and_forget(&mut command_for(app, args))
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
        use std::time::{Duration, Instant};

        fn zombies_of(me: &str) -> usize {
            let listed = Command::new("ps")
                .args(["-e", "-o", "ppid=,stat="])
                .output()
                .expect("ps runs");
            String::from_utf8_lossy(&listed.stdout)
                .lines()
                .filter(|line| {
                    let mut parts = line.split_whitespace();
                    parts.next() == Some(me) && parts.next().is_some_and(|s| s.contains('Z'))
                })
                .count()
        }

        fn settles(patience: Duration, done: impl Fn() -> bool) -> bool {
            let started = Instant::now();
            while started.elapsed() < patience {
                if done() {
                    return true;
                }
                std::thread::sleep(Duration::from_millis(25));
            }
            done()
        }

        let mark = std::env::temp_dir().join(format!("linkunbound-ran-{}", std::process::id()));
        let _ = std::fs::remove_file(&mark);
        spawn_and_forget(Command::new("/usr/bin/touch").arg(&mark)).expect("touch runs");
        for _ in 0..3 {
            spawn_and_forget(&mut Command::new("/usr/bin/true")).expect("true runs");
        }
        assert!(
            settles(Duration::from_secs(10), || mark.is_file()),
            "the child never ran"
        );
        let _ = std::fs::remove_file(&mark);

        let me = std::process::id().to_string();
        assert!(
            settles(Duration::from_secs(10), || zombies_of(&me) == 0),
            "{} children left as zombies",
            zombies_of(&me)
        );
    }

    #[cfg(target_os = "macos")]
    mod macos {
        use super::super::mac::{How, command_for, how};
        use std::path::Path;

        fn args(list: &[&str]) -> Vec<String> {
            list.iter().map(|a| (*a).to_owned()).collect()
        }

        fn spelled(command: &std::process::Command) -> Vec<String> {
            std::iter::once(command.get_program())
                .chain(command.get_args())
                .map(|a| a.to_string_lossy().into_owned())
                .collect()
        }

        /// The bundle is a directory, so handing it to a process is handing over a directory.
        /// A link with nothing but the URL goes through `open`, which is the only way Safari
        /// receives one at all and what brings the browser to the front.
        #[test]
        fn a_bundle_with_nothing_but_a_link_is_opened_the_way_the_system_opens_one() {
            let app = Path::new("/Applications");
            assert_eq!(how(app, &args(&["https://a.test"])), How::Bundle);
            assert_eq!(
                spelled(&command_for(app, &args(&["https://a.test"]))),
                ["/usr/bin/open", "-a", "/Applications", "https://a.test"]
            );
        }

        /// `open` drops everything past `--args` when the app is already running; an instance
        /// of its own hands the command line to the running copy, and the browser is still
        /// started by Launch Services rather than as a child answering for our permissions.
        #[test]
        fn a_link_carrying_a_switch_is_opened_as_an_instance_with_the_link_last() {
            let app = Path::new("/Applications");
            let carried = args(&[
                "--profile-directory=Work",
                "--incognito",
                "https://a.test/--x",
            ]);
            assert_eq!(how(app, &carried), How::Instance);
            assert_eq!(
                spelled(&command_for(app, &carried)),
                [
                    "/usr/bin/open",
                    "-n",
                    "-a",
                    "/Applications",
                    "--args",
                    "--profile-directory=Work",
                    "--incognito",
                    "https://a.test/--x"
                ]
            );
        }

        /// Somebody who added a browser by hand named a program, not a bundle.
        #[test]
        fn a_path_that_is_already_a_program_is_run_as_one() {
            let echo = Path::new("/bin/echo");
            assert_eq!(how(echo, &args(&["https://a.test"])), How::Itself);
            assert_eq!(
                spelled(&command_for(echo, &args(&["--x", "https://a.test"]))),
                ["/bin/echo", "--x", "https://a.test"]
            );
        }
    }
}
