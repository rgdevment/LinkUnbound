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

/// Guards the boundary between an untrusted inbound URL and spawning a process.
/// Browsers read any argument starting with `-` (or `/` on Windows) as a switch,
/// so a crafted `--gpu-launcher=calc.exe` would make one execute a binary.
#[must_use]
pub fn is_launchable(raw: &str) -> bool {
    if raw.is_empty() || raw.starts_with(['-', '/', '\\']) {
        return false;
    }
    parsed(raw).is_some_and(|url| LAUNCHABLE_SCHEMES.contains(&url.scheme()))
}

/// Everything an inbound link goes through before it may be shown or launched.
#[must_use]
pub fn normalise(raw: &str) -> Option<String> {
    let unwrapped = unwrap_safe_link(&unwrap_edge_protocol(raw));
    is_launchable(&unwrapped).then_some(unwrapped)
}

#[cfg(test)]
mod tests {
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
}
