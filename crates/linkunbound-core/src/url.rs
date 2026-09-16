use percent_encoding::percent_decode_str;
use url::Url;

const EDGE_HTTPS: &str = "microsoft-edge-https://";
const EDGE_PREFIXES: [&str; 2] = ["microsoft-edge://", "microsoft-edge:"];

/// Only what a browser is meant to receive from an untrusted source. `file:` is
/// deliberately absent: on Windows `file://host/share` is a UNC path, and the
/// browser resolving it over SMB hands the user's NTLM hash to that host.
const LAUNCHABLE_SCHEMES: [&str; 2] = ["http", "https"];

/// A wrapper is only trusted when Microsoft serves it. Matching on shape alone
/// turns the unwrapper into an open redirector: any page with `safelinks` in its
/// path could send the click somewhere else entirely.
const WRAPPER_HOSTS: [&str; 5] = [
    ".safelinks.protection.outlook.com",
    ".office.net",
    ".office.com",
    ".microsoft.com",
    ".microsoft",
];

/// Teams, Outlook, Widgets and Start menu search wrap links in a scheme wired to
/// Edge instead of handing out plain `https:`.
#[must_use]
pub fn unwrap_edge_protocol(raw: &str) -> String {
    let lower = raw.to_ascii_lowercase();
    if lower.starts_with(EDGE_HTTPS) {
        return format!("https://{}", &raw[EDGE_HTTPS.len()..]);
    }
    for prefix in EDGE_PREFIXES {
        if lower.starts_with(prefix) {
            return raw[prefix.len()..].to_owned();
        }
    }
    raw.to_owned()
}

fn parsed(raw: &str) -> Option<Url> {
    let url = Url::parse(raw).ok()?;
    url.has_host().then_some(url)
}

/// The canonical host as the browser will resolve it: punycode, lowercased, and
/// agreeing with WHATWG on backslashes, credentials and IPv6 literals. Anything
/// else routes a link by one host while the browser opens another.
#[must_use]
pub fn host_of(raw: &str) -> Option<String> {
    Some(parsed(raw)?.host_str()?.to_owned())
}

fn is_microsoft_wrapper(url: &Url) -> bool {
    let Some(host) = url.host_str() else {
        return false;
    };
    let host = host.to_ascii_lowercase();
    let served_by_microsoft = WRAPPER_HOSTS
        .iter()
        .any(|suffix| host.ends_with(suffix) || host == suffix.trim_start_matches('.'));
    if !served_by_microsoft {
        return false;
    }
    host.ends_with(".safelinks.protection.outlook.com")
        || url.path().to_ascii_lowercase().contains("safelinks")
}

/// Returns the destination wrapped inside a SafeLinks interstitial, or `raw`
/// when there is none to unwrap.
#[must_use]
pub fn unwrap_safe_link(raw: &str) -> String {
    let Some(url) = parsed(raw) else {
        return raw.to_owned();
    };
    if !is_microsoft_wrapper(&url) {
        return raw.to_owned();
    }
    let Some((_, inner)) = url.query_pairs().find(|(k, _)| k == "url") else {
        return raw.to_owned();
    };
    match Url::parse(&inner) {
        Ok(target) if LAUNCHABLE_SCHEMES.contains(&target.scheme()) && target.has_host() => {
            inner.into_owned()
        }
        _ => raw.to_owned(),
    }
}

/// True when the destination still looks like a wrapper after unwrapping, which
/// means Microsoft changed a shape this does not know yet.
#[must_use]
pub fn looks_unresolved(url: &str) -> bool {
    parsed(url).is_some_and(|u| is_microsoft_wrapper(&u))
}

#[cfg(target_os = "macos")]
const WEB_FILE_EXTENSIONS: [&str; 4] = ["html", "htm", "xhtml", "svg"];

#[cfg(target_os = "macos")]
#[must_use]
pub fn local_web_file_extensions() -> &'static [&'static str] {
    &WEB_FILE_EXTENSIONS
}

#[cfg(target_os = "macos")]
#[must_use]
pub fn local_web_file(path: &std::path::Path) -> Option<String> {
    let real = std::fs::canonicalize(path).ok()?;
    if !real.is_file() {
        return None;
    }
    let extension = real.extension()?.to_str()?.to_ascii_lowercase();
    if !WEB_FILE_EXTENSIONS.contains(&extension.as_str()) {
        return None;
    }
    Url::from_file_path(&real).ok().map(Into::into)
}

/// Read from the URL's own segments rather than a filesystem path: on Windows a
/// `file:///Users/...` URL has no drive and is not a path at all, and the picker
/// still has to name the document it was handed.
#[must_use]
pub fn local_file_parts(raw: &str) -> Option<(String, String)> {
    let url = Url::parse(raw).ok()?;
    if url.scheme() != "file" || url.has_host() {
        return None;
    }
    let mut segments: Vec<String> = url
        .path_segments()?
        .map(|segment| percent_decode_str(segment).decode_utf8_lossy().into_owned())
        .collect();
    let name = segments.pop().filter(|name| !name.is_empty())?;
    let folder = segments
        .pop()
        .filter(|folder| !folder.is_empty())
        .map(|folder| format!("…/{folder}"))
        .unwrap_or_default();
    Some((name, folder))
}

#[cfg(target_os = "macos")]
fn launchable_file(raw: &str) -> Option<String> {
    let url = Url::parse(raw).ok()?;
    if url.scheme() != "file" || url.has_host() {
        return None;
    }
    local_web_file(&url.to_file_path().ok()?)
}

#[cfg(not(target_os = "macos"))]
fn launchable_file(_raw: &str) -> Option<String> {
    None
}

/// Guards the boundary between an untrusted inbound URL and spawning a process.
/// Browsers read any argument starting with `-` (or `/` on Windows) as a switch,
/// so a crafted `--gpu-launcher=calc.exe` would make one execute a binary.
#[must_use]
pub fn is_launchable(raw: &str) -> bool {
    if raw.is_empty() || raw.starts_with(['-', '/', '\\']) {
        return false;
    }
    parsed(raw).is_some_and(|url| LAUNCHABLE_SCHEMES.contains(&url.scheme()))
        || launchable_file(raw).is_some()
}

/// Everything an inbound link goes through before it may be shown or launched.
#[must_use]
pub fn normalise(raw: &str) -> Option<String> {
    let unwrapped = unwrap_safe_link(&unwrap_edge_protocol(raw));
    if let Some(document) = launchable_file(&unwrapped) {
        return Some(document);
    }
    is_launchable(&unwrapped).then_some(unwrapped)
}

#[cfg(test)]
mod tests {

    /// A wrapper is a link somebody else wrote, and what it carries is not ours to trust. The
    /// scheme and the host are checked before anything is handed on, or a SafeLink could deliver
    /// `javascript:` or a path on somebody else's machine and this would pass it along as if
    /// Microsoft had vouched for it.
    #[test]
    fn a_wrapper_does_not_get_to_hand_over_whatever_it_likes() {
        for inner in [
            "javascript:alert(1)",
            "file:///C:/Windows/System32/calc.exe",
            "file://attacker.test/share/x",
            "vbscript:msgbox",
            "data:text/html,<script>x</script>",
            "ms-settings:defaultapps",
        ] {
            let wrapped = format!(
                "https://eur01.safelinks.protection.outlook.com/?url={}",
                percent(inner)
            );
            assert_eq!(
                unwrap_safe_link(&wrapped),
                wrapped,
                "«{inner}» came back out of a wrapper"
            );
        }
    }

    /// `file://` is the one that makes both halves of that check earn their place: it carries a
    /// host, so a check that asked only about hosts would hand it over — and a path on somebody
    /// else's machine is a login handed to whoever holds it.
    #[test]
    fn a_wrapper_carrying_a_path_on_another_machine_is_left_alone() {
        let wrapped = "https://eur01.safelinks.protection.outlook.com/?url=file%3A%2F%2Fattacker.test%2Fshare%2Fx";

        assert_eq!(unwrap_safe_link(wrapped), wrapped);
    }

    /// What it is for, so it has to still do it.
    #[test]
    fn an_ordinary_wrapped_link_is_handed_over() {
        let wrapped =
            "https://eur01.safelinks.protection.outlook.com/?url=https%3A%2F%2Fgithub.com%2Fa";

        assert_eq!(unwrap_safe_link(wrapped), "https://github.com/a");
    }

    /// The picker says a link is still wrapped so the person knows why the reaches are greyed.
    /// Answering no to everything takes the explanation away and leaves the greying unexplained.
    #[test]
    fn a_wrapper_is_recognised_as_one_and_a_plain_link_is_not() {
        assert!(looks_unresolved(
            "https://eur01.safelinks.protection.outlook.com/?url=x"
        ));
        assert!(!looks_unresolved("https://github.com/a"));
        assert!(!looks_unresolved("not a url at all"));
    }

    /// Either one is reason enough to refuse: nothing to open, or something that reads as a
    /// switch to the browser it would be handed to.
    #[test]
    fn a_link_is_refused_for_being_empty_or_for_looking_like_a_switch() {
        assert!(!is_launchable(""), "nothing to open");
        for said in ["--gpu-launcher=calc.exe", "/c calc.exe", r"\server\share"] {
            assert!(!is_launchable(said), "{said}");
        }
        assert!(is_launchable("https://github.com/a"));
    }

    fn percent(said: &str) -> String {
        said.chars()
            .map(|c| match c {
                'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
                _ => format!("%{:02X}", c as u32),
            })
            .collect()
    }
    use super::*;

    #[test]
    fn edge_wrapped_links_come_back_as_ordinary_urls() {
        assert_eq!(
            unwrap_edge_protocol("microsoft-edge:https://a.test"),
            "https://a.test"
        );
        assert_eq!(
            unwrap_edge_protocol("microsoft-edge-https://a.test/x"),
            "https://a.test/x"
        );
        assert_eq!(unwrap_edge_protocol("https://a.test"), "https://a.test");
    }

    #[test]
    fn a_safe_link_resolves_to_its_destination() {
        let wrapped = "https://eu01.safelinks.protection.outlook.com/?url=https%3A%2F%2Fgithub.com%2Fx&data=05";
        assert_eq!(unwrap_safe_link(wrapped), "https://github.com/x");
    }

    #[test]
    fn a_renamed_microsoft_cdn_is_still_recognised() {
        let old = "https://statics.teams.cdn.office.net/evergreen-assets/safelinks/2/atp.html?url=https%3A%2F%2Fgithub.com";
        let renamed = "https://teams.public.onecdn.static.microsoft/evergreen-assets/safelinks/2/atp.html?url=https%3A%2F%2Fgithub.com";
        for wrapped in [old, renamed] {
            assert_eq!(unwrap_safe_link(wrapped), "https://github.com");
        }
    }

    #[test]
    fn a_page_that_merely_mentions_safelinks_cannot_redirect_us() {
        let hostile = "https://intranet.corp/docs/safelinks-guide?url=https%3A%2F%2Fevil.test";
        assert_eq!(unwrap_safe_link(hostile), hostile);
        assert!(!looks_unresolved(hostile));
    }

    #[test]
    fn the_host_is_the_one_the_browser_will_open() {
        assert_eq!(
            host_of(r"https://good.test\@evil.test/").as_deref(),
            Some("good.test")
        );
        assert_eq!(host_of("https://[::1]:8080/x").as_deref(), Some("[::1]"));
        assert_eq!(
            host_of("https://user:pw@example.com:8443/x").as_deref(),
            Some("example.com")
        );
    }

    #[test]
    fn an_international_domain_has_one_canonical_form() {
        let punycode = Some("xn--mnchen-3ya.de");
        assert_eq!(host_of("https://münchen.de/x").as_deref(), punycode);
        assert_eq!(host_of("https://MÜNCHEN.de/x").as_deref(), punycode);
        assert_eq!(host_of("https://xn--mnchen-3ya.de/x").as_deref(), punycode);
    }

    #[test]
    fn a_file_url_from_outside_never_reaches_a_browser() {
        assert!(!is_launchable("file://evil.test/share/payload"));
        assert!(!is_launchable("file:///C:/Windows/System32/calc.exe"));
    }

    #[test]
    fn arguments_a_browser_would_read_as_switches_are_refused() {
        assert!(!is_launchable("--gpu-launcher=calc.exe"));
        assert!(!is_launchable("/c calc.exe"));
        assert!(!is_launchable("\\\\host\\share"));
        assert!(!is_launchable(""));
        assert!(!is_launchable("javascript:alert(1)"));
        assert!(is_launchable("https://github.com"));
    }

    #[test]
    fn a_teams_link_survives_both_wrappers_at_once() {
        let raw = "microsoft-edge:https://eu01.safelinks.protection.outlook.com/?url=https%3A%2F%2Fgithub.com%2Fa";
        assert_eq!(normalise(raw).as_deref(), Some("https://github.com/a"));
    }

    #[test]
    fn a_hostile_wrapper_never_reaches_a_browser() {
        assert!(normalise("--gpu-launcher=calc.exe").is_none());
        assert!(normalise("microsoft-edge:--gpu-launcher=calc.exe").is_none());
        assert!(normalise("microsoft-edge:file://evil.test/share").is_none());
    }

    #[test]
    fn a_local_file_is_named_by_itself_and_its_folder() {
        assert_eq!(
            local_file_parts("file:///Users/ana/Documents/My%20Page.html"),
            Some(("My Page.html".to_owned(), "…/Documents".to_owned()))
        );
        assert_eq!(
            local_file_parts("file:///page.svg"),
            Some(("page.svg".to_owned(), String::new()))
        );
        assert_eq!(
            local_file_parts("file:///C:/Users/ana/Desktop/Notas%20%C3%B1.htm"),
            Some(("Notas ñ.htm".to_owned(), "…/Desktop".to_owned()))
        );
        assert!(local_file_parts("https://example.com/page.html").is_none());
        assert!(local_file_parts("file://host/share/page.html").is_none());
        assert!(local_file_parts("file:///").is_none());
        assert!(local_file_parts("file:///Users/ana/Documents/").is_none());
    }

    #[cfg(target_os = "macos")]
    mod local_files {
        use super::super::{is_launchable, local_web_file, normalise};
        use std::path::{Path, PathBuf};

        fn scratch(name: &str) -> PathBuf {
            let dir = std::env::temp_dir().join(format!("linkunbound-url-{}", std::process::id()));
            std::fs::create_dir_all(&dir).expect("a scratch directory");
            dir.join(name)
        }

        #[test]
        fn a_web_document_on_disk_becomes_a_link_a_browser_opens() {
            for name in ["page.html", "page.HTM", "page.xhtml", "drawing.svg"] {
                let file = scratch(name);
                std::fs::write(&file, "<p>hi</p>").expect("a file");
                let link = local_web_file(&file).expect(name);
                assert!(link.starts_with("file:///"), "{link}");
                assert!(link.ends_with(name), "{link}");
                assert!(is_launchable(&link), "{link}");
                assert_eq!(normalise(&link).as_deref(), Some(link.as_str()));
            }
        }

        #[test]
        fn anything_that_is_not_a_web_document_is_refused() {
            let text = scratch("notes.txt");
            std::fs::write(&text, "x").expect("a file");
            assert!(local_web_file(&text).is_none());

            let folder = scratch("folder.html");
            std::fs::create_dir_all(&folder).expect("a directory");
            assert!(local_web_file(&folder).is_none());

            assert!(local_web_file(Path::new("/nowhere/at/all.html")).is_none());
            assert!(!is_launchable("file:///nowhere/at/all.html"));
            assert!(normalise("file:///nowhere/at/all.html").is_none());
        }

        #[test]
        fn a_symbolic_link_is_read_through_to_what_it_names() {
            let real = scratch("real.html");
            std::fs::write(&real, "x").expect("a file");
            let alias = scratch("alias.htm");
            let _ = std::fs::remove_file(&alias);
            std::os::unix::fs::symlink(&real, &alias).expect("a symlink");
            let link = local_web_file(&alias).expect("resolved");
            assert!(link.ends_with("real.html"), "{link}");

            let elsewhere = scratch("elsewhere.html");
            let _ = std::fs::remove_file(&elsewhere);
            std::os::unix::fs::symlink(scratch("notes.txt"), &elsewhere).expect("a symlink");
            std::fs::write(scratch("notes.txt"), "x").expect("a file");
            assert!(local_web_file(&elsewhere).is_none(), "the target decides");
        }

        #[test]
        fn a_path_on_another_machine_is_still_refused() {
            assert!(!is_launchable("file://host/share/page.html"));
            assert!(normalise("file://host/share/page.html").is_none());
        }
    }
}
