mod browser;
mod config;
mod private;
mod rule;
mod store;
mod url;

pub use browser::{Browser, LaunchRefused, Profile};
pub use config::{
    BrowserConfig, ConfigError, SCHEMA_VERSION, read_browsers, read_rules, write_browsers,
    write_rules,
};
pub use private::private_flag_for;
pub use rule::{Rule, RuleSet, Scope, Target, site_of};
pub use store::{Store, StoreError};
pub use url::{
    host_of, is_launchable, looks_unresolved, normalise, unwrap_edge_protocol, unwrap_safe_link,
};

#[cfg(test)]
mod tests {
    use super::host_of;

    #[test]
    fn a_host_is_lowercased_and_stripped_of_everything_around_it() {
        assert_eq!(
            host_of("https://GitHub.com/rgdevment").as_deref(),
            Some("github.com")
        );
        assert_eq!(
            host_of("https://example.com?q=1").as_deref(),
            Some("example.com")
        );
    }

    #[test]
    fn nothing_usable_yields_nothing() {
        assert!(host_of("not a url").is_none());
        assert!(host_of("https://").is_none());
    }

    /// WHATWG collapses the extra slash, so this is a host and the browser would
    /// treat it as one too. Agreeing with the browser is the whole point.
    #[test]
    fn an_extra_slash_is_read_the_way_a_browser_reads_it() {
        assert_eq!(host_of("https:///path").as_deref(), Some("path"));
    }
}
