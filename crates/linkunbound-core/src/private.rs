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
    let needle = executable.to_ascii_lowercase();
    FAMILIES
        .iter()
        .find(|(markers, _)| markers.iter().any(|m| needle.contains(m)))
        .map(|(_, flag)| *flag)
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
}
