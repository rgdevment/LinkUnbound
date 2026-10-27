use linkunbound_core::{Origin, Rule, Target, host_of};
use linkunbound_shell::{Listed, Reaches};

use crate::{notice, store};

pub fn forget(rule_id: &str) {
    if let Err(why) = store().edit_rules(|rules| rules.remove(rule_id)) {
        notice::noted("rules", &why.to_string());
    }
}

/// Reports rather than swallows: the window says the choice was remembered, and
/// a failed write would leave that claim false with nothing on screen to correct
/// it.
pub fn remember(
    url: &str,
    chosen: &Listed,
    private: bool,
    reach: Reaches,
    source: Option<Origin>,
) -> bool {
    let key = source.as_ref().map(|origin| origin.key.clone());
    let Some(rule) = rule_for(url, chosen, private, reach, key) else {
        return true;
    };
    store()
        .edit_rules(|rules| {
            rules.upsert_from(rule.clone(), source.as_ref());
            true
        })
        .map_err(|why| notice::noted("rules", &why.to_string()))
        .is_ok()
}

/// `None` when there is nothing to remember. Only the origin reach carries the
/// app: the others answer the same link whichever one produced it, and without
/// a known origin that reach has nothing to bind to.
pub fn rule_for(
    url: &str,
    chosen: &Listed,
    private: bool,
    reach: Reaches,
    source_app: Option<String>,
) -> Option<Rule> {
    let host = host_of(url)?;
    let scope = reach.scope(url, &host)?;
    if reach.binds_to_the_source() && source_app.is_none() {
        return None;
    }
    Some(Rule {
        id: String::new(),
        scope,
        source_app: reach.binds_to_the_source().then_some(source_app).flatten(),
        target: Target {
            browser_id: chosen.browser_id.clone(),
            profile_id: chosen.profile_id.clone(),
        },
        private,
    })
}

#[cfg(test)]
mod tests {
    use super::rule_for;
    use linkunbound_core::Scope;
    use linkunbound_shell::{Listed, Reaches};

    fn chosen() -> Listed {
        Listed {
            browser_id: "chrome".to_owned(),
            profile_id: Some("Default".to_owned()),
            name: "Google Chrome".to_owned(),
            profile: "Personal".to_owned(),
            icon: None,
            can_private: true,
            key: None,
        }
    }

    /// The origin is read when the link arrives, because by the time the user
    /// picks, the picker itself is the foreground window.
    #[test]
    fn only_the_origin_reach_binds_the_rule_to_an_app() {
        let url = "https://github.com/a";

        let bound = rule_for(
            url,
            &chosen(),
            false,
            Reaches::FromApp,
            Some("teams".to_owned()),
        )
        .expect("an origin reach with a known origin makes a rule");
        assert_eq!(bound.source_app.as_deref(), Some("teams"));
        assert_eq!(bound.scope, Scope::Any, "it covers any address");

        let site = rule_for(
            url,
            &chosen(),
            false,
            Reaches::Site,
            Some("teams".to_owned()),
        )
        .expect("a site reach makes a rule too");
        assert_eq!(
            site.source_app, None,
            "but it answers whichever app produced the link"
        );
        assert_eq!(site.scope, Scope::Site("github.com".to_owned()));
    }

    #[test]
    fn an_origin_rule_is_not_written_without_an_origin() {
        assert!(
            rule_for(
                "https://github.com/a",
                &chosen(),
                false,
                Reaches::FromApp,
                None
            )
            .is_none()
        );
    }

    #[test]
    fn just_this_time_leaves_nothing_behind() {
        assert!(
            rule_for(
                "https://github.com/a",
                &chosen(),
                false,
                Reaches::Once,
                None
            )
            .is_none()
        );
    }

    #[test]
    fn the_rule_carries_the_profile_and_the_private_choice() {
        let rule = rule_for("https://github.com/a", &chosen(), true, Reaches::Site, None)
            .expect("should make a rule");
        assert_eq!(rule.target.browser_id, "chrome");
        assert_eq!(rule.target.profile_id.as_deref(), Some("Default"));
        assert!(rule.private);
    }
}
