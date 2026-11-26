//! Rule commands: the list, the form, editing in place, and the probe.

use std::cmp::Reverse;

use linkunbound_core::{
    Browser, Origin, Replaced, Rule, RuleSet, Scope, SeenApp, Target, Why, host_of,
    is_public_suffix, normalise, same_name, site_of, why_first,
};
use serde::Serialize;

use crate::{app_icon_data, catalogue, icon_data, store};

/// The rule as the settings window shows it: what it covers, where it opens and
/// whether the browser it names is still installed.
#[derive(Serialize)]
pub struct RuleView {
    id: String,
    /// As stored, so the screen can offer to change it without matching names.
    browser_id: String,
    profile_id: Option<String>,
    kind: &'static str,
    covers: String,
    browser: String,
    profile: Option<String>,
    icon: Option<String>,
    private: bool,
    source_app: Option<String>,
    resolved: bool,
    /// The browser is there and only the profile the rule names is not.
    profile_gone: bool,
}

fn is_scheme(said: &str) -> bool {
    said.starts_with(|c: char| c.is_ascii_alphabetic())
        && said
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.'))
}

/// What the picker would have saved for the app typed: on a Mac its bundle id when one is
/// installed by that name, so the rule outlives a change of language; otherwise the name.
fn app_key(said: &str) -> String {
    #[cfg(target_os = "macos")]
    if let Some(id) = linkunbound_mac::bundle_id_named(said) {
        return id;
    }
    said.to_lowercase().trim_end_matches(".exe").to_owned()
}

/// An app typed by its name is the app a rule already saved by bundle id, when that id goes by
/// the same name: the form would otherwise add a second rule the first one hides.
fn named_alike<'a>(rules: impl IntoIterator<Item = &'a Rule>, rule: &Rule) -> bool {
    let Some(typed) = rule.source_app.as_deref() else {
        return false;
    };
    let names = |said: &str| [said.to_owned(), shown_origin(said)];
    let wanted = names(typed);
    rules.into_iter().any(|r| {
        r.scope == rule.scope
            && r.source_app.as_deref().is_some_and(|saved| {
                names(saved)
                    .iter()
                    .any(|name| wanted.iter().any(|typed| same_name(name, typed)))
            })
    })
}

/// A Mac picker saves the bundle id, which no language changes; the person reads the app's name.
pub(crate) fn shown_origin(saved: &str) -> String {
    #[cfg(target_os = "macos")]
    if let Some(name) = linkunbound_mac::app_named(saved) {
        return name;
    }
    saved.to_owned()
}

fn describe(rule: &Rule, browsers: &[Browser]) -> RuleView {
    let found = browsers.iter().find(|b| b.id == rule.target.browser_id);
    let profile = rule.target.profile_id.as_ref().and_then(|id| {
        found
            .and_then(|b| b.profiles.iter().find(|p| &p.id == id))
            .map(|p| p.name.clone())
    });
    let (kind, covers) = match &rule.scope {
        Scope::Any => ("any", String::new()),
        Scope::Url(u) => ("url", u.clone()),
        Scope::Host(h) => ("host", h.clone()),
        Scope::Site(d) => ("site", d.clone()),
    };
    // A profile that is gone is a rule that never fires, however present its browser.
    let resolved = found.is_some() && (rule.target.profile_id.is_none() || profile.is_some());
    let profile_gone = found.is_some() && rule.target.profile_id.is_some() && profile.is_none();
    RuleView {
        id: rule.id.clone(),
        browser_id: rule.target.browser_id.clone(),
        profile_id: rule.target.profile_id.clone(),
        kind,
        covers,
        browser: found.map_or_else(|| rule.target.browser_id.clone(), |b| b.name.clone()),
        profile,
        icon: found.and_then(|b| icon_data(&b.exe, &b.id)),
        private: rule.private,
        source_app: rule.source_app.as_deref().map(shown_origin),
        resolved,
        profile_gone,
    }
}

#[tauri::command]
pub fn rules_list() -> Result<Vec<RuleView>, String> {
    let browsers = catalogue();
    Ok(store()
        .rules()
        .map_err(|e| e.to_string())?
        .rules
        .iter()
        .map(|r| describe(r, &browsers))
        .collect())
}

#[tauri::command]
pub fn rules_remove(id: String) -> Result<Vec<RuleView>, String> {
    store()
        .edit_rules(|rules| rules.remove(&id))
        .map_err(|e| e.to_string())?;
    rules_list()
}

/// What the form said, turned into the rule the picker would have written for the same choice:
/// the same scopes, the same origin binding, the same target. Refused, by name, when the value
/// is not what the kind needs or the destination is gone.
fn rule_from(
    kind: &str,
    value: &str,
    target: Target,
    private: bool,
    browsers: &[Browser],
) -> Result<Rule, &'static str> {
    let Some(browser) = browsers.iter().find(|b| b.id == target.browser_id) else {
        return Err("browserGone");
    };
    if let Some(profile) = target.profile_id.as_deref()
        && !browser.profiles.iter().any(|p| p.id == profile)
    {
        return Err("browserGone");
    }
    // Saved, such a rule would wear the badge and never fire: the launch refuses it and the link
    // falls through to the picker, which does not offer that browser privately either.
    if private && !browser.supports_private() {
        return Err("rulePrivateImpossible");
    }
    let said = value.trim();
    if said.is_empty() {
        return Err("ruleValueEmpty");
    }
    // Read the way an arriving link's host is read, so `user@`, a port or a trailing dot come out
    // as the host a rule is matched against. A wildcard never matches; a lone name like
    // `intranet` is a host when no path follows it, as the picker already treats it.
    let host_named = || -> Result<String, &'static str> {
        // Only a scheme at the very start makes it a link: `a.com/?next=https://b.com` is a.com.
        let bare = match said.split_once("://") {
            Some((scheme, rest)) if is_scheme(scheme) => rest,
            _ => said,
        }
        .trim_end_matches('/');
        let host = host_of(&format!("https://{bare}")).ok_or("ruleValueNotHost")?;
        let single = !host.contains('.') && !host.starts_with('[');
        if host.contains('*') || (single && bare.contains(['/', '\\', '?', '#'])) {
            return Err("ruleValueNotHost");
        }
        Ok(host)
    };
    // A site is a registrable domain: a lone name is either a whole country's ending (`np`) or
    // a word that no address carries as its site (`github`).
    let site_named = || -> Result<String, &'static str> {
        let site = site_of(&host_named()?);
        if !site.contains('.') && !site.starts_with('[') {
            return Err("ruleValueNotSite");
        }
        if is_public_suffix(&site) {
            return Err("ruleValueIsSuffix");
        }
        Ok(site)
    };
    let (scope, source_app) = match kind {
        "url" => {
            let url = normalise(said).ok_or("ruleValueNotUrl")?;
            (Scope::Url(url), None)
        }
        "host" => (Scope::Host(host_named()?), None),
        "site" => (Scope::Site(site_named()?), None),
        // «Slack» and «Slack.exe» both mean the app somebody sees in «Desde …».
        "app" => (Scope::Any, Some(app_key(said))),
        _ => return Err("ruleKindUnknown"),
    };
    Ok(Rule {
        id: String::new(),
        scope,
        source_app,
        target,
        private,
    })
}

// Async so it runs off the main thread: on a Mac an app typed by name may be looked for among
// every installed one.
#[tauri::command(async)]
pub fn rules_add(
    kind: String,
    value: String,
    browser_id: String,
    profile_id: Option<String>,
    private: bool,
) -> Result<Vec<RuleView>, String> {
    let target = Target {
        browser_id,
        profile_id,
    };
    let rule = rule_from(&kind, &value, target, private, &catalogue()).map_err(str::to_owned)?;
    // The picker replaces on purpose — «always here» again is a change of mind — but a form
    // that overwrote a rule the person forgot they had would do it without a word.
    let mut standing = false;
    store()
        .edit_rules(|rules| {
            if rules.standing_in_for(&rule).is_some() || named_alike(&rules.rules, &rule) {
                standing = true;
                return false;
            }
            rules.upsert(rule);
            true
        })
        .map_err(|e| e.to_string())?;
    if standing {
        return Err("ruleExists".to_owned());
    }
    rules_list()
}

/// In place: remaking the rule would send it to the end of the order.
#[tauri::command]
pub fn rules_retarget(
    id: String,
    browser_id: String,
    profile_id: Option<String>,
) -> Result<Vec<RuleView>, String> {
    let target = Target {
        browser_id,
        profile_id,
    };
    let browsers = catalogue();
    let opens_privately = browsers
        .iter()
        .find(|b| b.id == target.browser_id)
        .is_some_and(Browser::supports_private);
    let mut refused = false;
    let changed = store()
        .edit_rules(|rules| {
            if rules
                .rules
                .iter()
                .any(|r| r.id == id && r.private && !opens_privately)
            {
                refused = true;
                return false;
            }
            rules.retarget(&id, target.clone())
        })
        .map_err(|e| e.to_string())?;
    if refused {
        return Err("rulePrivateImpossible".to_owned());
    }
    if !changed {
        return Err("ruleGone".to_owned());
    }
    rules_list()
}

/// A rule the picker bound to an app and a site keeps its app: the form has no way to say both.
fn edited_in(rules: &mut RuleSet, id: &str, mut rule: Rule) -> Replaced {
    let Some(old) = rules.rules.iter().find(|r| r.id == id) else {
        return Replaced::Gone;
    };
    if rule.source_app.is_none() && old.scope != Scope::Any {
        rule.source_app.clone_from(&old.source_app);
    }
    if named_alike(rules.rules.iter().filter(|r| r.id != id), &rule) {
        return Replaced::Taken;
    }
    rules.replace(id, rule)
}

#[tauri::command(async)]
pub fn rules_edit(
    id: String,
    kind: String,
    value: String,
    browser_id: String,
    profile_id: Option<String>,
    private: bool,
) -> Result<Vec<RuleView>, String> {
    let target = Target {
        browser_id,
        profile_id,
    };
    let rule = rule_from(&kind, &value, target, private, &catalogue()).map_err(str::to_owned)?;
    let mut outcome = Replaced::Gone;
    store()
        .edit_rules(|rules| {
            outcome = edited_in(rules, &id, rule);
            outcome == Replaced::Done
        })
        .map_err(|e| e.to_string())?;
    match outcome {
        Replaced::Done => rules_list(),
        Replaced::Gone => Err("ruleGone".to_owned()),
        Replaced::Taken => Err("ruleExists".to_owned()),
    }
}

#[derive(Serialize)]
pub struct Probe {
    winner: Option<RuleView>,
    why: Option<Why>,
    also: Vec<RuleView>,
    /// The site a new rule would cover, offered when nothing matches.
    site: String,
}

fn probed(
    rules: &RuleSet,
    url: &str,
    from_app: Option<&str>,
    browsers: &[Browser],
) -> Result<Probe, &'static str> {
    let said = url.trim();
    let written = match said.split_once("://") {
        Some((scheme, _)) if is_scheme(scheme) => said.to_owned(),
        _ => format!("https://{said}"),
    };
    let link = normalise(&written).ok_or("ruleValueNotUrl")?;
    let host = host_of(&link).ok_or("ruleValueNotUrl")?;
    let origin = from_app.map(|app| Origin {
        key: app_key(app),
        label: app.to_owned(),
    });
    let found = rules.matching_from(&link, &host, origin.as_ref());
    let why = found.first().map(|w| why_first(w, found.get(1).copied()));
    let mut views = found.iter().map(|r| describe(r, browsers));
    Ok(Probe {
        winner: views.next(),
        why,
        also: views.collect(),
        site: site_of(&host),
    })
}

// Async for the same reason as `rules_add`: the app may be looked for by name.
#[tauri::command(async)]
pub fn rules_probe(url: String, from_app: Option<String>) -> Result<Probe, String> {
    let rules = store().rules().map_err(|e| e.to_string())?;
    probed(&rules, &url, from_app.as_deref(), &catalogue()).map_err(str::to_owned)
}

/// An app the «From» field offers: one links came from, or one a rule names and none came from yet.
#[derive(Serialize)]
pub struct SeenAppView {
    name: String,
    links: u64,
    last_seen: Option<u64>,
    icon: Option<String>,
    /// The «From» rule that already holds it, so the form can tell it from the one being edited.
    rule_id: Option<String>,
}

#[derive(Debug, PartialEq, Eq)]
struct Offered {
    origin: Origin,
    links: u64,
    last_seen: Option<u64>,
    program: Option<String>,
    rule_id: Option<String>,
}

/// Matched on what the rule saved, as `named_alike` does: on a Mac that is a bundle id, which
/// only the record's key carries, while the record's label is the name in lower case.
fn offered(seen: Vec<SeenApp>, rules: &[Rule], shown: impl Fn(&str) -> String) -> Vec<Offered> {
    let mut all: Vec<Offered> = seen
        .into_iter()
        .map(|app| Offered {
            origin: Origin {
                key: app.key,
                label: app.label,
            },
            links: app.links,
            last_seen: Some(app.last_seen),
            program: app.program,
            rule_id: None,
        })
        .collect();
    all.sort_by_key(|app| Reverse(app.last_seen));
    for rule in rules {
        let Some(saved) = rule.source_app.as_deref() else {
            continue;
        };
        let name = shown(saved);
        let at = all
            .iter()
            .position(|app| app.origin.answers_to(saved) || app.origin.answers_to(&name))
            .unwrap_or_else(|| {
                all.push(Offered {
                    origin: Origin {
                        key: saved.to_owned(),
                        label: name,
                    },
                    links: 0,
                    last_seen: None,
                    program: None,
                    rule_id: None,
                });
                all.len() - 1
            });
        if rule.scope == Scope::Any && all[at].rule_id.is_none() {
            all[at].rule_id = Some(rule.id.clone());
        }
    }
    all
}

#[cfg(target_os = "macos")]
fn program_of(app: &Offered) -> Option<String> {
    linkunbound_mac::app_path(&app.origin.key)
}

#[cfg(not(target_os = "macos"))]
fn program_of(app: &Offered) -> Option<String> {
    app.program.clone()
}

// Async: icons are extracted, and on a Mac each app is looked up by its bundle id.
#[tauri::command(async)]
pub fn apps_seen() -> Vec<SeenAppView> {
    let store = store();
    let rules = store.rules().map(|set| set.rules).unwrap_or_default();
    offered(store.apps_seen(), &rules, shown_origin)
        .into_iter()
        .map(|app| SeenAppView {
            icon: program_of(&app).and_then(|program| app_icon_data(&program, &app.origin.key)),
            name: app.origin.label,
            links: app.links,
            last_seen: app.last_seen,
            rule_id: app.rule_id,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{describe, edited_in, named_alike, offered, probed, rule_from};
    use crate::tests::detected;
    use linkunbound_core::{Profile, Replaced, Rule, RuleSet, Scope, SeenApp, Target, Why};

    fn with_profiles(id: &str, profiles: &[&str]) -> linkunbound_core::Browser {
        linkunbound_core::Browser {
            profiles: profiles
                .iter()
                .map(|p| Profile {
                    id: (*p).to_owned(),
                    name: (*p).to_owned(),
                    args: vec![],
                })
                .collect(),
            ..detected(id)
        }
    }

    fn aimed(browser: &str, profile: Option<&str>) -> Target {
        Target {
            browser_id: browser.to_owned(),
            profile_id: profile.map(str::to_owned),
        }
    }

    /// The form writes the same rule the picker would for the same choice, so the two ways in
    /// cannot disagree about what a kind means.
    #[test]
    fn a_rule_from_the_form_covers_what_its_kind_says() {
        let all = vec![with_profiles("chrome", &["Default"]), detected("firefox")];

        let url = rule_from(
            "url",
            " https://docs.google.com/a?b=1 ",
            aimed("firefox", None),
            false,
            &all,
        )
        .expect("a url");
        assert_eq!(
            url.scope,
            Scope::Url("https://docs.google.com/a?b=1".to_owned())
        );
        assert!(url.source_app.is_none());

        let host = rule_from(
            "host",
            "https://docs.google.com/x",
            aimed("firefox", None),
            true,
            &all,
        )
        .expect("a host, read off a url");
        assert_eq!(host.scope, Scope::Host("docs.google.com".to_owned()));
        assert!(host.private);

        let site = rule_from(
            "site",
            "Docs.Google.com",
            aimed("chrome", Some("Default")),
            false,
            &all,
        )
        .expect("a site, read off a bare host");
        assert_eq!(site.scope, Scope::Site("google.com".to_owned()));

        let app =
            rule_from("app", "Slack.EXE", aimed("firefox", None), false, &all).expect("an app");
        assert_eq!(app.scope, Scope::Any);
        assert_eq!(
            app.source_app.as_deref(),
            Some("slack"),
            "as the picker records an origin: lowercased, without the executable's suffix"
        );

        let odd = rule_from(
            "host",
            "User@Docs.Google.com:8443/x.",
            aimed("firefox", None),
            false,
            &all,
        )
        .expect("a host, read the way a link's host is read");
        assert_eq!(odd.scope, Scope::Host("docs.google.com".to_owned()));
    }

    #[test]
    fn an_app_typed_by_name_is_the_one_a_rule_already_names() {
        let saved = |app: &str| Rule {
            id: String::new(),
            scope: Scope::Any,
            source_app: Some(app.to_owned()),
            target: aimed("chrome", None),
            private: false,
        };
        let typed = saved("slack");
        assert!(named_alike(&[saved("Slack")], &typed));
        assert!(!named_alike(&[saved("teams")], &typed));
        #[cfg(target_os = "macos")]
        assert!(named_alike(&[saved("com.apple.Safari")], &saved("safari")));
    }

    /// The picker saves a rule for `intranet` or `localhost`; the form has to accept what the
    /// picker writes, or the same rule can be made one way and not the other.
    #[test]
    fn a_single_name_is_a_host_in_the_form_as_in_the_picker() {
        let all = vec![with_profiles("chrome", &["Default"])];
        for (said, host) in [
            ("localhost", "localhost"),
            ("intranet", "intranet"),
            ("https://intranet/", "intranet"),
            ("localhost:3000", "localhost"),
            ("https://intranet:8080/", "intranet"),
            ("intranet.", "intranet"),
            ("ana@intranet", "intranet"),
        ] {
            let rule = rule_from("host", said, aimed("chrome", None), false, &all).expect(said);
            assert_eq!(rule.scope, Scope::Host(host.to_owned()), "{said}");
        }
        let site =
            rule_from("site", "bbc.co.uk", aimed("chrome", None), false, &all).expect("a site");
        assert_eq!(site.scope, Scope::Site("bbc.co.uk".to_owned()));
    }

    /// Every refusal names what to fix; none of them writes a rule that would match nothing or
    /// open nowhere.
    #[test]
    fn a_rule_from_the_form_is_refused_by_name() {
        let all = vec![with_profiles("chrome", &["Default"])];
        let chrome = || aimed("chrome", None);

        assert_eq!(
            rule_from("url", "   ", chrome(), false, &all),
            Err("ruleValueEmpty")
        );
        assert_eq!(
            rule_from("url", "not a url", chrome(), false, &all),
            Err("ruleValueNotUrl")
        );
        assert_eq!(
            rule_from("host", "just words", chrome(), false, &all),
            Err("ruleValueNotHost")
        );
        assert_eq!(
            rule_from("host", "*.example.com", chrome(), false, &all),
            Err("ruleValueNotHost")
        );
        assert_eq!(
            rule_from("site", "co.uk", chrome(), false, &all),
            Err("ruleValueIsSuffix")
        );
        for said in [
            "github.com/login?return_to=https://gitlab.com/",
            "example.com/redirect/https://foo",
        ] {
            let rule = rule_from("host", said, chrome(), false, &all).expect(said);
            assert_ne!(rule.scope, Scope::Host("gitlab.com".to_owned()), "{said}");
            assert_ne!(rule.scope, Scope::Host("foo".to_owned()), "{said}");
        }
        for said in [r"a\b.com", r"intranet\reports"] {
            assert_eq!(
                rule_from("host", said, chrome(), false, &all),
                Err("ruleValueNotHost"),
                "{said}"
            );
        }
        for said in ["np", "github", "intranet"] {
            assert_eq!(
                rule_from("site", said, chrome(), false, &all),
                Err("ruleValueNotSite"),
                "{said}"
            );
        }
        assert_eq!(
            rule_from("site", "https://github.io/", chrome(), false, &all),
            Err("ruleValueIsSuffix")
        );
        assert_eq!(
            rule_from("site", "a/b.com", chrome(), false, &all),
            Err("ruleValueNotHost")
        );
        assert_eq!(
            rule_from("when", "x.com", chrome(), false, &all),
            Err("ruleKindUnknown")
        );
        assert_eq!(
            rule_from("site", "x.com", aimed("edge", None), false, &all),
            Err("browserGone")
        );
        assert_eq!(
            rule_from("site", "x.com", aimed("chrome", Some("Work")), false, &all),
            Err("browserGone"),
            "a profile the browser does not have is a destination that is gone"
        );
    }

    fn rule_for(browser: &str, profile: Option<&str>) -> Rule {
        Rule {
            id: "r1".to_owned(),
            scope: Scope::Site("github.com".to_owned()),
            source_app: None,
            target: Target {
                browser_id: browser.to_owned(),
                profile_id: profile.map(str::to_owned),
            },
            private: false,
        }
    }

    #[test]
    fn a_rule_pointing_at_a_browser_that_is_gone_is_not_called_resolved() {
        let installed = [detected("chrome")];

        let live = describe(&rule_for("chrome", Some("Default")), &installed);
        assert!(live.resolved);
        assert_eq!(live.browser, "Google Chrome");
        assert_eq!(live.profile.as_deref(), Some("Personal"));

        let gone = describe(&rule_for("brave", None), &installed);
        assert!(
            !gone.resolved,
            "an uninstalled browser must not read as live"
        );
        assert_eq!(gone.browser, "brave", "and it still names what it was");
    }

    #[test]
    fn a_rule_whose_profile_is_gone_says_so_apart_from_a_gone_browser() {
        let installed = [detected("chrome")];
        let profile = describe(&rule_for("chrome", Some("Work")), &installed);
        assert!(!profile.resolved);
        assert!(profile.profile_gone);
        let browser = describe(&rule_for("brave", Some("Work")), &installed);
        assert!(!browser.resolved);
        assert!(
            !browser.profile_gone,
            "the browser itself is what is missing"
        );
        assert!(!describe(&rule_for("chrome", Some("Default")), &installed).profile_gone);
    }

    #[test]
    fn a_rule_carries_the_destination_it_was_saved_with() {
        let view = describe(&rule_for("chrome", Some("Default")), &[detected("chrome")]);
        assert_eq!(view.browser_id, "chrome");
        assert_eq!(view.profile_id.as_deref(), Some("Default"));
    }

    fn rule_of(id: &str, scope: Scope, app: Option<&str>, browser: &str) -> Rule {
        Rule {
            id: id.to_owned(),
            scope,
            source_app: app.map(str::to_owned),
            target: aimed(browser, None),
            private: false,
        }
    }

    fn site(domain: &str) -> Scope {
        Scope::Site(domain.to_owned())
    }

    fn set_of(rules: Vec<Rule>) -> RuleSet {
        RuleSet {
            schema_version: 2,
            rules,
        }
    }

    #[test]
    fn editing_a_rule_rewrites_it_where_it_stood() {
        let mut rules = set_of(vec![
            rule_of("site:a.test", site("a.test"), None, "chrome"),
            rule_of("site:b.test", site("b.test"), None, "chrome"),
            rule_of("site:c.test", site("c.test"), None, "chrome"),
        ]);
        let edited = rule_of("", Scope::Host("b2.test".to_owned()), None, "firefox");

        assert_eq!(edited_in(&mut rules, "site:b.test", edited), Replaced::Done);

        let ids: Vec<&str> = rules.rules.iter().map(|r| r.id.as_str()).collect();
        assert_eq!(ids, ["site:a.test", "host:b2.test", "site:c.test"]);
        assert_eq!(rules.rules[1].target.browser_id, "firefox");
    }

    #[test]
    fn a_site_rule_for_an_app_keeps_its_app_when_the_form_edits_it() {
        let mut rules = set_of(vec![rule_of(
            "site:a.test@slack",
            site("a.test"),
            Some("slack"),
            "chrome",
        )]);
        let from_the_form = rule_of("", site("b.test"), None, "firefox");

        assert_eq!(
            edited_in(&mut rules, "site:a.test@slack", from_the_form),
            Replaced::Done
        );

        assert_eq!(rules.rules[0].id, "site:b.test@slack");
        assert_eq!(rules.rules[0].source_app.as_deref(), Some("slack"));
    }

    #[test]
    fn a_rule_for_an_app_alone_edited_to_a_site_has_no_origin_to_carry() {
        let mut rules = set_of(vec![rule_of(
            "any@slack",
            Scope::Any,
            Some("slack"),
            "chrome",
        )]);
        let to_a_site = rule_of("", site("b.test"), None, "chrome");

        assert_eq!(
            edited_in(&mut rules, "any@slack", to_a_site),
            Replaced::Done
        );

        assert_eq!(rules.rules[0].source_app, None);
    }

    #[test]
    fn editing_onto_what_another_rule_covers_is_refused_and_leaves_both() {
        let mut rules = set_of(vec![
            rule_of("site:a.test", site("a.test"), None, "chrome"),
            rule_of("site:b.test", site("b.test"), None, "edge"),
        ]);
        let onto_a = rule_of("", site("a.test"), None, "firefox");

        assert_eq!(
            edited_in(&mut rules, "site:b.test", onto_a),
            Replaced::Taken
        );

        assert_eq!(rules.rules[0].target.browser_id, "chrome");
        assert_eq!(rules.rules[1].id, "site:b.test");
        assert_eq!(rules.rules[1].target.browser_id, "edge");
    }

    #[test]
    fn editing_onto_an_app_another_rule_saved_under_a_different_case_is_refused() {
        let mut rules = set_of(vec![
            rule_of("any@Slack", Scope::Any, Some("Slack"), "chrome"),
            rule_of("site:b.test", site("b.test"), None, "edge"),
        ]);
        let typed = rule_of("", Scope::Any, Some("slack"), "firefox");

        assert_eq!(edited_in(&mut rules, "site:b.test", typed), Replaced::Taken);
        assert_eq!(rules.rules[1].id, "site:b.test");
    }

    #[test]
    fn editing_a_rule_that_is_gone_says_so() {
        let mut rules = set_of(vec![rule_of("site:a.test", site("a.test"), None, "chrome")]);
        let edited = rule_of("", site("c.test"), None, "chrome");

        assert_eq!(
            edited_in(&mut rules, "site:zz.test", edited),
            Replaced::Gone
        );
        assert_eq!(rules.rules.len(), 1);
    }

    #[test]
    fn a_link_is_answered_by_the_best_rule_and_the_others_that_also_match() {
        let rules = set_of(vec![
            rule_of("site:github.com", site("github.com"), None, "chrome"),
            rule_of(
                "host:gist.github.com",
                Scope::Host("gist.github.com".to_owned()),
                None,
                "firefox",
            ),
            rule_of("site:gitlab.com", site("gitlab.com"), None, "chrome"),
        ]);
        let browsers = [detected("chrome")];

        let probe = probed(&rules, "https://gist.github.com/x", None, &browsers).expect("a link");

        let winner = probe.winner.expect("a winner");
        assert_eq!(winner.id, "host:gist.github.com");
        assert!(!winner.resolved, "firefox is not among the installed");
        assert_eq!(probe.why, Some(Why::Narrower));
        let also: Vec<&str> = probe.also.iter().map(|r| r.id.as_str()).collect();
        assert_eq!(also, ["site:github.com"]);
        assert_eq!(probe.site, "github.com");
    }

    #[test]
    fn a_link_nothing_covers_offers_the_site_a_rule_would_cover() {
        let rules = set_of(vec![rule_of("site:a.test", site("a.test"), None, "chrome")]);

        let probe = probed(&rules, "https://docs.example.co.uk/a", None, &[]).expect("a link");

        assert!(probe.winner.is_none());
        assert!(probe.why.is_none());
        assert!(probe.also.is_empty());
        assert_eq!(probe.site, "example.co.uk");
    }

    #[test]
    fn a_link_typed_without_its_scheme_is_read_as_https() {
        let rules = set_of(vec![rule_of(
            "url:https://a.test/x",
            Scope::Url("https://a.test/x".to_owned()),
            None,
            "chrome",
        )]);

        let probe = probed(&rules, "  a.test/x ", None, &[detected("chrome")]).expect("a link");

        assert_eq!(probe.winner.expect("a winner").id, "url:https://a.test/x");
        assert_eq!(probe.why, Some(Why::Alone));
    }

    #[test]
    fn a_link_without_its_scheme_may_carry_another_link_in_its_query() {
        let rules = set_of(vec![rule_of(
            "site:a.test",
            Scope::Site("a.test".to_owned()),
            None,
            "chrome",
        )]);

        let probe = probed(&rules, "a.test/login?next=https://b.test/", None, &[]).expect("a link");

        assert_eq!(probe.winner.expect("a winner").id, "site:a.test");
    }

    #[test]
    fn a_link_from_an_app_is_answered_by_the_rule_for_that_app_by_name() {
        let rules = set_of(vec![
            rule_of("site:a.test", site("a.test"), None, "chrome"),
            rule_of("site:a.test@slack", site("a.test"), Some("slack"), "edge"),
        ]);

        let from_slack = probed(&rules, "https://a.test/", Some("Slack.exe"), &[]).expect("a link");
        assert_eq!(from_slack.winner.expect("a winner").id, "site:a.test@slack");
        assert_eq!(from_slack.why, Some(Why::Origin));

        let from_nowhere = probed(&rules, "https://a.test/", None, &[]).expect("a link");
        assert_eq!(from_nowhere.winner.expect("a winner").id, "site:a.test");
        assert!(from_nowhere.also.is_empty());
    }

    fn sighted(key: &str, label: &str, links: u64, last_seen: u64) -> SeenApp {
        SeenApp {
            key: key.to_owned(),
            label: label.to_owned(),
            links,
            last_seen,
            program: None,
        }
    }

    fn naming(id: &str, scope: Scope, app: &str) -> Rule {
        Rule {
            id: id.to_owned(),
            scope,
            source_app: Some(app.to_owned()),
            target: aimed("chrome", None),
            private: false,
        }
    }

    fn as_is(saved: &str) -> String {
        saved.to_owned()
    }

    #[test]
    fn the_apps_seen_come_most_recent_first_whatever_order_they_were_stored_in() {
        let seen = vec![
            sighted("olk", "olk", 1, 10),
            sighted("slack", "slack", 5, 30),
            sighted("teams", "teams", 2, 20),
        ];
        let all = offered(seen, &[], as_is);
        let names: Vec<&str> = all.iter().map(|a| a.origin.key.as_str()).collect();
        assert_eq!(names, ["slack", "teams", "olk"]);
        assert_eq!(all[0].links, 5);
        assert_eq!(all[0].last_seen, Some(30));
        assert!(all.iter().all(|a| a.rule_id.is_none()));
    }

    #[test]
    fn an_app_only_a_rule_names_goes_last_with_no_links_and_no_date() {
        let rules = [naming("any@zoom", Scope::Any, "zoom")];
        let all = offered(vec![sighted("slack", "slack", 1, 5)], &rules, as_is);
        assert_eq!(all.len(), 2);
        assert_eq!(all[1].origin.label, "zoom");
        assert_eq!(all[1].links, 0);
        assert_eq!(all[1].last_seen, None);
        assert!(all[1].program.is_none());
        assert_eq!(all[1].rule_id.as_deref(), Some("any@zoom"));
    }

    #[test]
    fn rules_naming_the_same_app_add_it_once_and_the_first_from_rule_holds_it() {
        let rules = [
            naming("site:a.test@zoom", Scope::Site("a.test".to_owned()), "zoom"),
            naming("any@zoom", Scope::Any, "Zoom"),
            naming("any@zoom-again", Scope::Any, "zoom"),
        ];
        let all = offered(Vec::new(), &rules, as_is);
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].rule_id.as_deref(), Some("any@zoom"));
    }

    #[test]
    fn a_rule_for_a_site_from_an_app_does_not_hold_the_app() {
        let rules = [naming(
            "site:a.test@slack",
            Scope::Host("a.test".to_owned()),
            "slack",
        )];
        let all = offered(vec![sighted("slack", "slack", 3, 9)], &rules, as_is);
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].links, 3, "the sighting is kept, not duplicated");
        assert!(all[0].rule_id.is_none());
    }

    #[test]
    fn a_rule_that_names_no_app_changes_nothing() {
        let mut anyone = naming("any", Scope::Any, "x");
        anyone.source_app = None;
        let all = offered(vec![sighted("slack", "slack", 1, 1)], &[anyone], as_is);
        assert_eq!(all.len(), 1);
        assert!(all[0].rule_id.is_none());
    }

    #[test]
    fn a_rule_matches_an_app_by_its_key_or_its_label_in_any_case() {
        let seen = vec![
            sighted("ms-teams", "Microsoft Teams", 1, 2),
            sighted("olk", "olk", 1, 1),
        ];
        let rules = [
            naming("any@label", Scope::Any, "microsoft teams"),
            naming("any@key", Scope::Any, "OLK"),
        ];
        let all = offered(seen, &rules, as_is);
        assert_eq!(all.len(), 2);
        assert_eq!(all[0].rule_id.as_deref(), Some("any@label"));
        assert_eq!(all[1].rule_id.as_deref(), Some("any@key"));
    }

    /// On a Mac a rule saves the bundle id, which the record's key carries and its label does not.
    #[test]
    fn a_rule_saved_with_a_bundle_id_finds_the_app_by_its_key_or_by_the_name_shown() {
        let seen = vec![sighted("com.tinyspeck.slackmacgap", "slack", 4, 7)];
        let by_id = [naming("any@id", Scope::Any, "com.tinyspeck.slackmacgap")];
        let all = offered(seen.clone(), &by_id, |_| "Slack".to_owned());
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].rule_id.as_deref(), Some("any@id"));

        let by_name = [naming("any@name", Scope::Any, "com.slack.unknown")];
        let all = offered(seen, &by_name, |_| "slack".to_owned());
        assert_eq!(all.len(), 1, "found by the name it is shown under");
        assert_eq!(all[0].rule_id.as_deref(), Some("any@name"));
    }

    #[test]
    fn an_app_a_rule_names_but_was_never_seen_takes_the_name_it_is_shown_under() {
        let rules = [naming("any@id", Scope::Any, "com.apple.Safari")];
        let all = offered(Vec::new(), &rules, |_| "Safari".to_owned());
        assert_eq!(all[0].origin.key, "com.apple.Safari");
        assert_eq!(all[0].origin.label, "Safari");
    }

    #[test]
    fn a_probe_that_is_not_a_link_is_refused() {
        let rules = set_of(vec![]);
        for said in ["", "   ", "not a link"] {
            assert!(
                matches!(probed(&rules, said, None, &[]), Err("ruleValueNotUrl")),
                "{said:?}"
            );
        }
    }
}
