use std::process::Command;

use linkunbound_core::{Browser, LaunchRefused};

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
    Command::new(&browser.exe).args(&args).spawn()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use linkunbound_core::Profile;

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
}
