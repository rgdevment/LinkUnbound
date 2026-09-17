/// Order matters: Edge is Chromium-based but spells the switch differently, so
/// it has to be tested before the generic Chromium markers.
const FAMILIES: [(&[&str], &str); 4] = [
    (&["msedge", "microsoft edge"], "-inprivate"),
    (
        &["firefox", "librewolf", "waterfox", "zen"],
        "-private-window",
    ),
    (&["opera"], "--private"),
    (
        &[
            "chrome",
            "chromium",
            "brave",
            "vivaldi",
            "thorium",
            "ungoogled",
            "arc",
        ],
        "--incognito",
    ),
];

/// There is no cross-browser standard here, and a wrong switch is not ignored:
/// Chromium reads an unknown `--flag` as a URL and Firefox opens an error page.
/// Anything unrecognised opts out rather than guessing.
#[must_use]
pub fn private_flag_for(executable: &str) -> Option<&'static str> {
    let needle = tail_of(executable).to_ascii_lowercase();
    FAMILIES
        .iter()
        .find(|(markers, _)| markers.iter().any(|m| needle.contains(m)))
        .map(|(_, flag)| *flag)
}

/// The executable and the folder it sits in, and nothing above: a marker as short as `arc` or
/// `zen` matched the user's own name in `C:\Users\Marcos`, and the folder is kept because Opera
/// once launched through a `launcher.exe` that only its folder names.
fn tail_of(executable: &str) -> String {
    let mut parts = executable
        .rsplit(['\\', '/'])
        .filter(|part| !part.is_empty());
    let name = parts.next().unwrap_or_default();
    match parts.next() {
        Some(folder) => format!("{folder}/{name}"),
        None => name.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::private_flag_for;

    #[test]
    fn edge_is_recognised_before_the_chromium_markers_it_also_matches() {
        assert_eq!(
            private_flag_for(r"C:\Program Files\Microsoft\Edge\msedge.exe"),
            Some("-inprivate")
        );
    }

    #[test]
    fn each_family_gets_the_switch_it_actually_understands() {
        assert_eq!(
            private_flag_for("/usr/bin/firefox"),
            Some("-private-window")
        );
        assert_eq!(private_flag_for("brave.exe"), Some("--incognito"));
        assert_eq!(private_flag_for("opera.exe"), Some("--private"));
    }

    #[test]
    fn an_unknown_browser_opts_out_instead_of_guessing() {
        assert!(private_flag_for("safari").is_none());
        assert!(private_flag_for(r"C:\Apps\some-browser.exe").is_none());
    }

    /// The path above the program is the person's, not the browser's: a user called Marcos had
    /// every browser of theirs read as Arc.
    #[test]
    fn the_folders_above_the_program_do_not_name_it() {
        assert!(private_flag_for(r"C:\Users\Marcos\Programs\Viewer\viewer.exe").is_none());
        assert!(private_flag_for(r"D:\Operations\Tools\tool.exe").is_none());
        assert_eq!(
            private_flag_for(r"C:\Users\Marcos\AppData\Local\Programs\Opera\launcher.exe"),
            Some("--private"),
            "the folder right above still counts"
        );
        assert_eq!(
            private_flag_for(r"C:\Users\Marcos\AppData\Local\Google\Chrome\Application\chrome.exe"),
            Some("--incognito")
        );
    }
}
