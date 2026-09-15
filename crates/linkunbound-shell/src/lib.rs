//! The resident half: the picker and the notice, drawn without a browser.

slint::include_modules!();

pub mod place;
pub mod single;
pub mod tray;

use std::rc::Rc;

use linkunbound_core::{Browser, Scope, Strings, local_file_parts, looks_unresolved, site_of};

/// Both windows read the same palette, so the choice is applied once per window
/// rather than threaded through every component that draws.
pub fn paint(picker: &Picker, notice: &Notice, light: bool) {
    picker.global::<Palette>().set_light(light);
    notice.global::<Palette>().set_light(light);
}

/// Extracted and drawn at this same side: any other ratio scales, and blurs.
pub const ICON_SIDE: u32 = 24;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reaches {
    Once,
    Url,
    Host,
    Site,
    /// Every link from the app the click came from, whatever the address. What
    /// the user means by ticking it on a link that arrived from Slack.
    FromApp,
}

impl Reaches {
    #[must_use]
    pub fn scope(self, url: &str, host: &str) -> Option<Scope> {
        match self {
            Self::Once => None,
            Self::Url => Some(Scope::Url(url.to_owned())),
            Self::Host => Some(Scope::Host(host.to_owned())),
            Self::Site => Some(Scope::Site(site_of(host))),
            Self::FromApp => Some(Scope::Any),
        }
    }

    /// Only this reach binds the rule to where the click came from; the others
    /// answer the same whatever app produced the link.
    #[must_use]
    pub const fn binds_to_the_source(self) -> bool {
        matches!(self, Self::FromApp)
    }

    #[must_use]
    pub fn at(index: i32) -> Self {
        match index {
            1 => Self::Url,
            2 => Self::Host,
            3 => Self::Site,
            4 => Self::FromApp,
            _ => Self::Once,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Listed {
    pub browser_id: String,
    pub profile_id: Option<String>,
    pub name: String,
    pub profile: String,
    pub icon: Option<String>,
    pub can_private: bool,
}

#[must_use]
pub fn destinations(browsers: &[Browser]) -> Vec<Listed> {
    browsers
        .iter()
        .filter(|b| !b.hidden)
        .flat_map(|browser| {
            let base = Listed {
                browser_id: browser.id.clone(),
                profile_id: None,
                name: browser.name.clone(),
                profile: String::new(),
                icon: None,
                can_private: browser.supports_private(),
            };
            if browser.profiles.is_empty() {
                return vec![base];
            }
            browser
                .profiles
                .iter()
                .map(|profile| Listed {
                    profile_id: Some(profile.id.clone()),
                    profile: profile.name.clone(),
                    ..base.clone()
                })
                .collect()
        })
        .collect()
}

/// A wrapper left unwrapped must not be remembered: the rule would key off
/// Microsoft's redirector and answer for every link it ever carries.
#[must_use]
pub fn reaches(
    words: &Strings,
    url: &str,
    host: &str,
    site: &str,
    source: Option<&str>,
) -> Vec<(String, bool, bool)> {
    let wrapped = looks_unresolved(url) || local_file_parts(url).is_some();
    let mut offered = vec![
        (words.reach_once.to_owned(), false, false),
        (words.reach_url.to_owned(), true, wrapped),
        (
            words.reach_subdomain.to_owned(),
            true,
            wrapped || host == site || host.is_empty(),
        ),
        (words.reach_site.to_owned(), true, wrapped),
    ];
    // Left out rather than greyed when there is no origin: a dead label would cost the live
    // ones their space.
    if let Some(source) = source {
        offered.push((Strings::fill(words.reach_from_app, source), true, false));
    }
    offered
}

/// The two the row shows. Everything else goes behind «More», because five labels never fit a
/// width the rows can also live at — and two of the five are dead half the time: a URL reach on
/// a wrapped link, a subdomain reach on a host that has none.
pub const IN_THE_ROW: [i32; 2] = [0, 3];

#[must_use]
pub fn in_the_row(index: i32) -> bool {
    IN_THE_ROW.contains(&index)
}

#[must_use]
pub fn split(url: &str) -> (String, String) {
    if let Some(parts) = local_file_parts(url) {
        return parts;
    }
    let Some(rest) = url.split_once("://").map(|(_, r)| r) else {
        return (url.to_owned(), String::new());
    };
    match rest.find(['/', '?', '#']) {
        Some(at) if rest[at..] != *"/" => (rest[..at].to_owned(), rest[at..].to_owned()),
        _ => (rest.trim_end_matches('/').to_owned(), String::new()),
    }
}

fn image_for(path: Option<&str>) -> slint::Image {
    path.and_then(|p| slint::Image::load_from_path(std::path::Path::new(p)).ok())
        .unwrap_or_default()
}

pub fn dress(window: &Picker, words: &Strings, url: &str, source: Option<&str>, rows: &[Listed]) {
    window.set_icon_side(f32::from(u16::try_from(ICON_SIDE).unwrap_or(24)));
    let (host, trail) = split(url);
    let site = site_of(&host);

    window.set_host(host.clone().into());
    window.set_trail(trail.into());
    window.set_source_line(
        source
            .map(|from| Strings::fill(words.picker_from, from))
            .unwrap_or_default()
            .into(),
    );
    window.set_remember_label(words.picker_remember.into());
    window.set_private_label(words.picker_private.into());
    window.set_private_on(false);
    window.set_copied(false);
    window.set_reach_index(0);
    let wrapped = looks_unresolved(url);
    window.set_alarming(!wrapped);
    window.set_problem(if wrapped {
        words.wrapper_unresolved.into()
    } else {
        slint::SharedString::new()
    });

    let listed: Vec<Destination> = rows
        .iter()
        .enumerate()
        .map(|(i, row)| Destination {
            key: (i + 1).to_string().into(),
            name: row.name.clone().into(),
            profile: row.profile.clone().into(),
            icon: image_for(row.icon.as_deref()),
            can_private: row.can_private,
        })
        .collect();
    window.set_rows(Rc::new(slint::VecModel::from(listed)).into());

    let said = reaches(words, url, &host, &site, source);
    let all: Vec<Reach> = said
        .into_iter()
        .enumerate()
        .map(|(at, (label, keeps, dead))| Reach {
            label: label.into(),
            keeps,
            dead,
            index: i32::try_from(at).unwrap_or(0),
        })
        .collect();

    window.set_reach_labels(
        Rc::new(slint::VecModel::from(
            all.iter().map(|one| one.label.clone()).collect::<Vec<_>>(),
        ))
        .into(),
    );
    window.set_in_the_row(
        Rc::new(slint::VecModel::from(
            all.iter()
                .map(|one| in_the_row(one.index))
                .collect::<Vec<_>>(),
        ))
        .into(),
    );
    window.set_reach_more_label(words.reach_more.into());

    // What the row draws, so the window can be measured against the words it will actually show
    // rather than against every reach there is.
    window.set_reaches_line(
        all.iter()
            .filter(|one| in_the_row(one.index))
            .map(|one| one.label.to_string())
            .chain(std::iter::once(words.reach_more.to_owned()))
            .collect::<Vec<_>>()
            .join(" · ")
            .into(),
    );

    let (row, rest): (Vec<Reach>, Vec<Reach>) =
        all.into_iter().partition(|one| in_the_row(one.index));
    window.set_reaches(Rc::new(slint::VecModel::from(row)).into());
    window.set_reaches_more(Rc::new(slint::VecModel::from(rest)).into());
}

#[cfg(test)]
mod tests {
    use super::dress;
    use super::{Listed, Palette, Reaches, destinations, image_for, paint, reaches, split};
    use linkunbound_core::{Browser, Language, Profile, Scope, Strings, looks_unresolved};
    use slint::ComponentHandle;

    const SPOKEN: Strings = Language::Spanish.strings();

    fn browser(id: &str, profiles: &[&str], private: bool) -> Browser {
        Browser {
            id: id.to_owned(),
            name: id.to_owned(),
            exe: format!("{id}.exe"),
            profiles: profiles
                .iter()
                .map(|p| Profile {
                    id: (*p).to_owned(),
                    name: (*p).to_owned(),
                    args: Vec::new(),
                })
                .collect(),
            extra_args: Vec::new(),
            private_flag: private.then(|| "--incognito".to_owned()),
            icon_path: None,
            custom: false,
            hidden: false,
        }
    }

    #[test]
    fn a_browser_with_profiles_becomes_one_row_each() {
        let rows = destinations(&[browser("chrome", &["Default", "Work"], true)]);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[1].profile, "Work");
        assert_eq!(rows[1].profile_id.as_deref(), Some("Work"));
    }

    #[test]
    fn a_browser_without_profiles_is_a_single_row() {
        let rows = destinations(&[browser("firefox", &[], true)]);
        assert_eq!(rows.len(), 1);
        assert!(rows[0].profile_id.is_none());
    }

    #[test]
    fn a_hidden_browser_never_reaches_the_picker() {
        let hidden = Browser {
            hidden: true,
            ..browser("edge", &[], true)
        };
        assert!(destinations(&[hidden]).is_empty());
    }

    #[test]
    fn a_browser_that_cannot_open_privately_says_so_up_front() {
        let rows = destinations(&[browser("odd", &[], false)]);
        assert!(!rows[0].can_private);
    }

    const PLAIN: &str = "https://github.com/a";

    fn dressed() -> Vec<Listed> {
        vec![
            Listed {
                browser_id: "firefox".to_owned(),
                profile_id: None,
                name: "Mozilla Firefox".to_owned(),
                profile: String::new(),
                icon: None,
                can_private: true,
            },
            Listed {
                browser_id: "chrome".to_owned(),
                profile_id: Some("Default".to_owned()),
                name: "Google Chrome".to_owned(),
                profile: "Personal".to_owned(),
                icon: None,
                can_private: false,
            },
        ]
    }

    /// The reaches used to grow the window: five labels needed more room than the rows do, and
    /// the fifth carries an app name no fixed width can be drawn for. Two live in the row now and
    /// the rest behind «More», so the window is the same size whatever the link or its origin.
    #[test]
    fn the_window_is_the_same_width_whatever_the_link_came_from() {
        headless();
        let window = crate::Picker::new().expect("a window");
        let words = Language::Spanish.strings();
        let rows = dressed();

        dress(&window, &words, "https://docs.google.com/a", None, &rows);
        let alone = window.get_wanted_width();
        assert_eq!(alone, 348.0);

        for from in ["ms-teams", "Microsoft Teams Classic"] {
            dress(
                &window,
                &words,
                "https://docs.google.com/a",
                Some(from),
                &rows,
            );
            assert_eq!(
                window.get_wanted_width(),
                alone,
                "«{from}» moved the window's width"
            );
        }
    }

    /// The arrows step through this list rather than through the row, and the row is now two
    /// entries long. Were the walk bounded by the row, the three reaches behind the opener would
    /// be out of the keyboard's reach entirely.
    #[test]
    fn the_keyboard_still_has_every_reach_to_walk() {
        use slint::Model;

        headless();
        let window = crate::Picker::new().expect("a window");
        let words = Language::Spanish.strings();
        let rows = dressed();

        dress(
            &window,
            &words,
            "https://docs.google.com/a",
            Some("ms-teams"),
            &rows,
        );
        assert_eq!(
            window.get_reach_labels().row_count(),
            5,
            "the arrows walk all five, not the two on show"
        );

        dress(&window, &words, "https://docs.google.com/a", None, &rows);
        assert_eq!(window.get_reach_labels().row_count(), 4);
    }

    /// The index is the whole contract between the window and the rule that gets written: the row
    /// hands one back and `Reaches::at` turns it into a scope. Splitting the reaches into a row
    /// and a menu must not renumber them, or choosing «the whole site» writes something else.
    #[test]
    fn splitting_the_reaches_leaves_every_index_where_it_was() {
        use slint::Model;

        headless();
        let window = crate::Picker::new().expect("a window");
        let words = Language::Spanish.strings();
        let rows = dressed();

        dress(
            &window,
            &words,
            "https://docs.google.com/a",
            Some("ms-teams"),
            &rows,
        );

        let row = window.get_reaches();
        let menu = window.get_reaches_more();
        assert_eq!(row.row_count(), 2);
        assert_eq!(menu.row_count(), 3);

        let mut seen: Vec<(i32, String)> = row
            .iter()
            .chain(menu.iter())
            .map(|one| (one.index, one.label.to_string()))
            .collect();
        seen.sort_by_key(|(at, _)| *at);

        assert_eq!(
            seen,
            vec![
                (0, words.reach_once.to_owned()),
                (1, words.reach_url.to_owned()),
                (2, words.reach_subdomain.to_owned()),
                (3, words.reach_site.to_owned()),
                (4, Strings::fill(words.reach_from_app, "ms-teams")),
            ],
            "every reach kept the number Reaches::at reads"
        );

        assert_eq!(
            row.iter().map(|one| one.index).collect::<Vec<_>>(),
            vec![0, 3],
            "the row shows «just this time» and «the whole site»"
        );
    }

    /// With a reach the row does not show, the opener has to name it: left saying «More», the
    /// choice is made and nothing on screen says what it was.
    #[test]
    fn the_opener_names_the_reach_when_it_is_one_the_row_hides() {
        headless();
        let window = crate::Picker::new().expect("a window");
        let words = Language::Spanish.strings();
        let rows = dressed();

        dress(
            &window,
            &words,
            "https://docs.google.com/a",
            Some("ms-teams"),
            &rows,
        );

        for at in [0, 3] {
            window.set_reach_index(at);
            assert!(!window.get_more_holds(), "{at} is drawn in the row");
        }
        for at in [1, 2, 4] {
            window.set_reach_index(at);
            assert!(window.get_more_holds(), "{at} is only behind the opener");
        }
    }

    /// Slint's testing backend, so the window needs neither a screen nor the main thread: winit
    /// wants both, and a runner has neither.
    fn headless() {
        i_slint_backend_testing::init_no_event_loop();
    }

    #[test]
    fn what_the_window_is_told_about_a_link() {
        use slint::Model;

        headless();
        let window = crate::Picker::new().expect("the software renderer must build a window");
        let words = Language::Spanish.strings();
        let rows = dressed();

        dress(
            &window,
            &words,
            "https://docs.google.com/d/1a9F/edit",
            None,
            &rows,
        );

        assert_eq!(window.get_host(), "docs.google.com");
        assert_eq!(window.get_trail(), "/d/1a9F/edit");

        let listed = window.get_rows();
        assert_eq!(listed.row_count(), 2);
        assert_eq!(
            listed.row_data(0).expect("first row").key,
            "1",
            "the keys are what the user presses, counted from one"
        );
        assert_eq!(listed.row_data(1).expect("second row").key, "2");
        assert!(listed.row_data(0).expect("firefox").can_private);
        assert!(
            !listed.row_data(1).expect("chrome").can_private,
            "a browser with no private switch says so, or the row lies"
        );
        assert_eq!(listed.row_data(1).expect("chrome").profile, "Personal");

        let offered = window.get_reaches();
        assert_eq!(
            offered.row_count(),
            2,
            "the row shows two and keeps the rest behind More"
        );
        assert_eq!(offered.row_data(0).expect("first").label, words.reach_once);
        assert_eq!(
            window.get_reaches_more().row_count(),
            2,
            "no origin, so no origin reach"
        );
        assert_eq!(window.get_problem(), "", "an ordinary link raises nothing");
        assert_eq!(window.get_source_line(), "");

        dress(&window, &words, PLAIN, Some("teams"), &rows);
        assert!(
            window.get_source_line().contains("teams"),
            "it names the app: {}",
            window.get_source_line()
        );
        assert_eq!(
            window.get_reaches().row_count() + window.get_reaches_more().row_count(),
            5,
            "the origin reach joins the ones behind the opener"
        );

        dress(
            &window,
            &words,
            "https://eu01.safelinks.protection.outlook.com/?whatever=1",
            None,
            &rows,
        );
        assert_eq!(window.get_problem(), words.wrapper_unresolved);
        assert!(!window.get_alarming(), "a wrapper is not a failure");

        window.set_private_on(true);
        window.set_copied(true);
        window.set_reach_index(3);
        dress(&window, &words, "https://gitlab.com/b", None, &rows);
        assert!(!window.get_private_on(), "private must not carry over");
        assert!(!window.get_copied(), "nor the copied tick");
        assert_eq!(window.get_reach_index(), 0, "nor the reach that was chosen");
        assert_eq!(window.get_problem(), "", "nor the previous link's notice");
    }

    #[test]
    fn the_subdomain_reach_is_dead_when_the_host_is_already_its_site() {
        let same = reaches(&SPOKEN, PLAIN, "github.com", "github.com", None);
        assert!(same[2].2);
        let sub = reaches(&SPOKEN, PLAIN, "docs.google.com", "google.com", None);
        assert!(!sub[2].2);
    }

    #[test]
    fn the_reaches_are_worded_in_the_language_they_are_asked_for() {
        let spanish = reaches(&SPOKEN, PLAIN, "github.com", "github.com", None);
        let english = reaches(
            &Language::English.strings(),
            PLAIN,
            "github.com",
            "github.com",
            None,
        );
        assert_eq!(spanish[0].0, "Solo esta vez");
        assert_eq!(english[0].0, "Just this time");
        assert_eq!(english[3].0, "The whole site");
    }

    /// What the user means by ticking it on a link that came from Slack: every
    /// link from that app, whatever the address. 1.x built exactly this rule.
    #[test]
    fn the_origin_reach_covers_any_address_and_only_that_app() {
        assert_eq!(
            Reaches::at(4).scope("https://github.com/a", "github.com"),
            Some(Scope::Any)
        );
        assert!(Reaches::at(4).binds_to_the_source());
        for other in [0, 1, 2, 3] {
            assert!(
                !Reaches::at(other).binds_to_the_source(),
                "reach {other} must answer whatever app produced the link"
            );
        }
    }

    /// A wrapper we could not unwrap is fine for this one: the rule keys off the
    /// app, not the address. Without a known origin there is nothing to key on.
    #[test]
    fn the_origin_reach_appears_only_when_the_origin_is_known() {
        let wrapped = "https://eu01.safelinks.protection.outlook.com/?whatever=1";
        let known = reaches(
            &SPOKEN,
            wrapped,
            "outlook.com",
            "outlook.com",
            Some("teams"),
        );
        assert_eq!(known.len(), 5);
        assert!(!known[4].2, "a wrapper does not disable the origin reach");
        assert!(
            known[4].0.contains("teams"),
            "it names the app: {}",
            known[4].0
        );

        let unknown = reaches(&SPOKEN, PLAIN, "github.com", "github.com", None);
        assert_eq!(
            unknown.len(),
            4,
            "five labels do not fit the row, so with no origin it is left out"
        );
    }

    /// Greying the reaches without a word leaves the user with no reason why.
    #[test]
    fn a_wrapped_link_is_named_as_such_and_not_as_a_failure() {
        let wrapped = "https://eu01.safelinks.protection.outlook.com/?whatever=1";
        assert!(looks_unresolved(wrapped));
        assert!(!looks_unresolved(PLAIN));

        let spanish = Language::Spanish.strings();
        assert!(spanish.wrapper_unresolved.contains("envuelto"));
        assert!(
            !spanish.wrapper_unresolved.is_empty(),
            "the picker shows this in place of nothing"
        );
    }

    #[test]
    fn a_wrapper_that_stayed_wrapped_cannot_be_remembered() {
        let wrapped = "https://eu01.safelinks.protection.outlook.com/?whatever=1";
        let offered = reaches(
            &SPOKEN,
            wrapped,
            "eu01.safelinks.protection.outlook.com",
            "outlook.com",
            None,
        );
        assert!(!offered[0].2, "solo esta vez sigue disponible");
        assert!(offered[1].2 && offered[2].2 && offered[3].2);
    }

    #[test]
    fn the_header_shows_the_host_apart_from_where_it_leads() {
        assert_eq!(
            split("https://docs.google.com/document/d/1a9F/edit"),
            (
                "docs.google.com".to_owned(),
                "/document/d/1a9F/edit".to_owned()
            )
        );
        assert_eq!(
            split("https://example.test/"),
            ("example.test".to_owned(), String::new())
        );
        assert_eq!(
            split("https://example.test"),
            ("example.test".to_owned(), String::new())
        );
    }

    #[test]
    fn a_local_file_is_headed_by_its_name_and_remembered_by_nothing() {
        let document = "file:///Users/ana/Documents/My%20Page.html";
        assert_eq!(
            split(document),
            ("My Page.html".to_owned(), "…/Documents".to_owned())
        );
        let offered = reaches(
            &SPOKEN,
            document,
            "My Page.html",
            "my page.html",
            Some("finder"),
        );
        assert!(!offered[0].2, "once is the one reach a file has");
        assert!(offered[1].2 && offered[2].2 && offered[3].2);
    }

    #[test]
    fn a_query_belongs_to_the_trail_not_the_host() {
        let (host, trail) = split("https://intranet.corp/x?f=a|b");
        assert_eq!(host, "intranet.corp");
        assert_eq!(trail, "/x?f=a|b");
    }

    #[test]
    fn choosing_once_writes_no_rule_at_all() {
        assert!(Reaches::at(0).scope("https://x.test/", "x.test").is_none());
    }

    #[test]
    fn each_reach_saves_what_its_label_promises() {
        let url = "https://docs.google.com/a";
        assert_eq!(
            Reaches::at(1).scope(url, "docs.google.com"),
            Some(Scope::Url(url.to_owned()))
        );
        assert_eq!(
            Reaches::at(2).scope(url, "docs.google.com"),
            Some(Scope::Host("docs.google.com".to_owned()))
        );
        assert_eq!(
            Reaches::at(3).scope(url, "docs.google.com"),
            Some(Scope::Site("google.com".to_owned()))
        );
    }

    #[test]
    fn an_index_out_of_range_falls_back_to_writing_nothing() {
        assert!(Reaches::at(99).scope("https://x.test/", "x.test").is_none());
        assert!(Reaches::at(-1).scope("https://x.test/", "x.test").is_none());
    }

    #[test]
    fn a_destination_carries_the_profile_the_launcher_will_need() {
        let rows: Vec<Listed> = destinations(&[browser("chrome", &["Work"], true)]);
        assert_eq!(rows[0].browser_id, "chrome");
        assert_eq!(rows[0].profile_id.as_deref(), Some("Work"));
    }

    /// Both windows are told once, and each has its own copy of the palette: telling only the
    /// picker leaves the notice painted in the other theme.
    #[test]
    fn both_windows_are_told_which_palette_to_paint() {
        headless();
        let picker = crate::Picker::new().expect("a picker");
        let notice = crate::Notice::new().expect("a notice");

        paint(&picker, &notice, true);
        assert!(picker.global::<Palette>().get_light());
        assert!(notice.global::<Palette>().get_light());

        paint(&picker, &notice, false);
        assert!(!picker.global::<Palette>().get_light());
        assert!(!notice.global::<Palette>().get_light());
    }

    /// An icon that fails to load leaves a blank square rather than stopping the picker, so
    /// nothing complains when loading quietly stops working — only the rows go empty.
    #[test]
    fn an_icon_on_disk_is_loaded_and_a_missing_one_is_not_fatal() {
        headless();
        let real = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../app/src-tauri/icons/32x32.png");

        let loaded = image_for(real.to_str());
        assert!(
            loaded.size().width > 0,
            "an icon that is there has to reach the row"
        );

        assert_eq!(image_for(None).size().width, 0);
        assert_eq!(image_for(Some("nothing/like/it.png")).size().width, 0);
    }
}
