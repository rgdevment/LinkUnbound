//! What the settings screen may change about a browser, checked before it is saved.

use std::collections::BTreeMap;

use linkunbound_core::{Browser, usable_key};

#[derive(serde::Serialize)]
pub struct BrowserView {
    pub id: String,
    pub name: String,
    pub exe: String,
    pub profiles: usize,
    /// Named, not just counted: a rule can point at one profile of a browser,
    /// so the screen has to be able to offer them apart.
    pub profile_names: Vec<(String, String)>,
    pub private: bool,
    pub custom: bool,
    pub hidden: bool,
    pub icon: Option<String>,
    /// Quoted back the way it has to be typed, or a value with a space in it would be split
    /// the next time the form was saved.
    pub args: String,
    pub private_flag: Option<String>,
    pub icon_path: Option<String>,
    /// The key chosen for each row, by profile id, or by "" for a browser with no profiles.
    pub keys: BTreeMap<String, char>,
    /// The key each row shows in the picker, chosen or not, so an empty box can say which it got.
    pub shown_keys: BTreeMap<String, char>,
}

pub fn seen(browser: &Browser, shown_keys: BTreeMap<String, char>) -> BrowserView {
    BrowserView {
        profiles: browser.profiles.len(),
        profile_names: browser
            .profiles
            .iter()
            .map(|p| (p.id.clone(), p.name.clone()))
            .collect(),
        private: browser.private_flag.is_some(),
        icon: crate::icon_data(browser.icon_source(), &browser.id),
        id: browser.id.clone(),
        name: browser.name.clone(),
        exe: browser.exe.clone(),
        custom: browser.custom,
        hidden: browser.hidden,
        args: linkunbound_core::join_args(&browser.extra_args),
        private_flag: browser.private_flag.clone(),
        icon_path: browser.icon_path.clone(),
        keys: browser.keys.clone(),
        shown_keys,
    }
}

pub fn views(all: &[Browser]) -> Vec<BrowserView> {
    let shown = linkunbound_core::picker_keys(all);
    all.iter()
        .zip(shown)
        .map(|(b, keys)| seen(b, keys))
        .collect()
}

/// What the settings screen can change about a browser it owns.
#[derive(serde::Deserialize)]
pub struct Edit {
    pub name: String,
    pub exe: String,
    /// The line as it was typed, not the words in it: a value in quotes is one argument, and
    /// deciding that in the form put the rule somewhere nothing could test it.
    pub args: String,
    pub private_flag: Option<String>,
    pub icon_path: Option<String>,
    /// What was typed for each row; an empty box leaves the row to the defaults.
    #[serde(default)]
    pub keys: BTreeMap<String, String>,
}

pub fn readable(path: &str) -> bool {
    let path = std::path::Path::new(path);
    path.is_file() || is_bundle(path)
}

#[cfg(target_os = "macos")]
fn is_bundle(path: &std::path::Path) -> bool {
    path.extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("app"))
        && path.join("Contents").join("Info.plist").is_file()
}

#[cfg(not(target_os = "macos"))]
fn is_bundle(_path: &std::path::Path) -> bool {
    false
}

pub fn hidden_in(mut all: Vec<Browser>, id: &str, hidden: bool) -> Result<Vec<Browser>, String> {
    // Hidden, its keys were free to give away; back in the picker, it lets go of those now taken.
    let taken: Vec<char> = all
        .iter()
        .filter(|b| b.id != id && !b.hidden)
        .flat_map(|b| {
            b.keys
                .values()
                .filter_map(|k| linkunbound_core::usable_key(*k))
        })
        .collect();
    let Some(found) = all.iter_mut().find(|b| b.id == id) else {
        return Err("browserGone".to_owned());
    };
    if found.hidden && !hidden {
        found
            .keys
            .retain(|_, k| linkunbound_core::usable_key(*k).is_none_or(|k| !taken.contains(&k)));
    }
    found.hidden = hidden;
    Ok(all)
}

/// A detected browser is hidden, never removed: the next scan would bring it
/// back and the deletion would look like it failed.
pub fn without(mut all: Vec<Browser>, id: &str) -> Result<Vec<Browser>, String> {
    if !all.iter().any(|b| b.id == id && b.custom) {
        return Err("browserNotYours".to_owned());
    }
    all.retain(|b| b.id != id);
    Ok(all)
}

/// Never reuses a number a live entry already holds, or editing one would
/// rename the other.
pub fn free_id(all: &[Browser]) -> String {
    (1..)
        .map(|n| format!("custom-{n}"))
        .find(|id| !all.iter().any(|b| &b.id == id))
        .unwrap_or_else(|| "custom-1".to_owned())
}

pub fn added(
    mut all: Vec<Browser>,
    edit: Edit,
    exists: impl Fn(&str) -> bool,
) -> Result<Vec<Browser>, String> {
    if !exists(&edit.exe) {
        return Err("browserNoProgram".to_owned());
    }
    let id = free_id(&all);
    let keys = keyed(&all, &id, &edit.keys)?;
    let private_flag = private_flag(edit.private_flag, &edit.exe);
    all.push(Browser {
        id,
        name: edit.name,
        exe: edit.exe,
        profiles: Vec::new(),
        extra_args: linkunbound_core::split_args(&edit.args),
        private_flag,
        icon_path: edit.icon_path.filter(|p| !p.is_empty()),
        custom: true,
        hidden: false,
        keys,
    });
    Ok(all)
}

pub fn edited(
    mut all: Vec<Browser>,
    id: &str,
    edit: Edit,
    exists: impl Fn(&str) -> bool,
) -> Result<Vec<Browser>, String> {
    let keys = keyed(&all, id, &edit.keys)?;
    let Some(found) = all.iter_mut().find(|b| b.id == id) else {
        return Err("browserGone".to_owned());
    };
    found.keys = keys;
    if found.custom {
        if !exists(&edit.exe) {
            return Err("browserNoProgram".to_owned());
        }
        found.exe = edit.exe;
    }
    found.name = edit.name;
    found.extra_args = linkunbound_core::split_args(&edit.args);
    // Detection knows how a browser it found opens privately, and keeps that; where it found one
    // it does not know — a family off its list — the flag is the person's to say, as for a
    // browser they added themselves.
    if found.custom || linkunbound_core::private_flag_for(&found.exe).is_none() {
        found.private_flag = private_flag(edit.private_flag, &found.exe);
    }
    found.icon_path = edit.icon_path.filter(|p| !p.is_empty());
    Ok(all)
}

/// The keys typed for one browser's rows. One that another row already holds is refused rather
/// than kept: the picker would give it to whichever row comes first, and the other would
/// silently lose it.
pub fn keyed(
    all: &[Browser],
    id: &str,
    typed: &BTreeMap<String, String>,
) -> Result<BTreeMap<String, char>, String> {
    let rows: Vec<String> = all.iter().find(|b| b.id == id).map_or_else(
        || vec![String::new()],
        |b| {
            b.rows()
                .iter()
                .map(|p| p.map(|p| p.id.clone()).unwrap_or_default())
                .collect()
        },
    );
    let mut keys = BTreeMap::new();
    for (row, said) in typed {
        let said = said.trim();
        if said.is_empty() || !rows.contains(row) {
            continue;
        }
        let mut chars = said.chars();
        let key = chars
            .next()
            .and_then(usable_key)
            .filter(|_| chars.next().is_none())
            .ok_or_else(|| "keyUnusable".to_owned())?;
        if keys.values().any(|k| *k == key) {
            return Err("keyTwice".to_owned());
        }
        if let Some(holder) = holder_of(all, id, key) {
            return Err(format!("keyTaken:{holder}"));
        }
        keys.insert(row.clone(), key);
    }
    Ok(keys)
}

/// The switch typed, or when the box was left empty the one the browser's family is known to use:
/// few people know `--incognito` exists, let alone that Edge spells it `-inprivate`.
fn private_flag(typed: Option<String>, exe: &str) -> Option<String> {
    typed
        .filter(|f| !f.trim().is_empty())
        .or_else(|| linkunbound_core::private_flag_for(exe).map(str::to_owned))
}

/// Who else holds a key: the browser's name, with the profile's when it is one of several.
fn holder_of(all: &[Browser], id: &str, key: char) -> Option<String> {
    all.iter()
        .filter(|b| b.id != id && !b.hidden)
        .find_map(|b| {
            let held = b
                .rows()
                .into_iter()
                .find(|p| b.key_for(p.map(|p| p.id.as_str())) == Some(key))?;
            Some(held.map_or_else(|| b.name.clone(), |p| format!("{} · {}", b.name, p.name)))
        })
}

#[cfg(test)]
mod tests {
    use super::{Edit, added, hidden_in, keyed};
    use linkunbound_core::{Browser, Profile};
    use std::collections::BTreeMap;

    fn browser(id: &str, profiles: &[(&str, &str)]) -> Browser {
        Browser {
            id: id.to_owned(),
            name: "Google Chrome".to_owned(),
            exe: "chrome.exe".to_owned(),
            profiles: profiles
                .iter()
                .map(|(id, name)| Profile {
                    id: (*id).to_owned(),
                    name: (*name).to_owned(),
                    args: Vec::new(),
                })
                .collect(),
            extra_args: Vec::new(),
            private_flag: None,
            icon_path: None,
            custom: false,
            hidden: false,
            keys: BTreeMap::new(),
        }
    }

    fn typed(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
        pairs
            .iter()
            .map(|(row, key)| ((*row).to_owned(), (*key).to_owned()))
            .collect()
    }

    /// Most people have never heard of `--incognito`: a browser added by hand whose family is
    /// known opens privately without anyone typing it, and a switch typed on purpose still wins.
    #[test]
    fn a_known_browser_brings_its_own_private_switch() {
        let add = |exe: &str, flag: &str| {
            let edit = Edit {
                name: "Mío".to_owned(),
                exe: exe.to_owned(),
                args: String::new(),
                private_flag: Some(flag.to_owned()),
                icon_path: None,
                keys: BTreeMap::new(),
            };
            added(Vec::new(), edit, |_| true).expect("there")[0]
                .private_flag
                .clone()
        };
        let chrome = "C:/Program Files/Google/Chrome/Application/chrome.exe";
        assert_eq!(add(chrome, "").as_deref(), Some("--incognito"));
        assert_eq!(
            add("C:/Edge/msedge.exe", " ").as_deref(),
            Some("-inprivate")
        );
        assert_eq!(add(chrome, "--otra").as_deref(), Some("--otra"));
        assert_eq!(add("C:/Apps/raro.exe", "").as_deref(), None);
    }

    /// A hidden browser holds no key, so another may take its key meanwhile; shown again, it lets
    /// go of that one rather than share it, and keeps the rest.
    #[test]
    fn a_browser_shown_again_lets_go_of_a_key_given_away() {
        let mut edge = browser("edge", &[]);
        edge.hidden = true;
        edge.keys.insert(String::new(), 'H');
        let mut firefox = browser("firefox", &[]);
        firefox.keys.insert(String::new(), 'H');
        let mut kept = browser("brave", &[]);
        kept.hidden = true;
        kept.keys.insert(String::new(), 'B');

        let shown = hidden_in(vec![edge, firefox, kept], "edge", false).expect("there");
        assert!(shown[0].keys.is_empty(), "Firefox has H now");
        assert_eq!(shown[1].key_for(None), Some('H'));
        let back = hidden_in(shown, "brave", false).expect("there");
        assert_eq!(back[2].key_for(None), Some('B'), "nobody took B");
    }

    /// A key is one digit from 1 or one letter, kept in capitals, and only while no other row holds
    /// it: the second row would never be reached by it.
    #[test]
    fn a_key_is_kept_only_while_no_other_row_holds_it() {
        let mut chrome = browser(
            "chrome",
            &[("Default", "Personal"), ("Profile 1", "Trabajo")],
        );
        chrome.keys.insert("Profile 1".to_owned(), 'W');
        let all = vec![chrome, browser("custom-1", &[])];

        let kept = keyed(&all, "custom-1", &typed(&[("", "p"), ("other", " ")])).expect("free");
        assert_eq!(
            kept,
            BTreeMap::from([(String::new(), 'P')]),
            "a blank box is no key"
        );
        assert_eq!(
            keyed(&all, "custom-1", &typed(&[("", "w")])).unwrap_err(),
            "keyTaken:Google Chrome · Trabajo"
        );
        assert_eq!(
            keyed(&all, "custom-1", &typed(&[("", "0")])).unwrap_err(),
            "keyUnusable"
        );
        assert_eq!(
            keyed(&all, "custom-1", &typed(&[("", "ab")])).unwrap_err(),
            "keyUnusable"
        );
        assert_eq!(
            keyed(
                &all,
                "chrome",
                &typed(&[("Default", "Q"), ("Profile 1", "q")])
            )
            .unwrap_err(),
            "keyTwice",
            "two rows of one browser cannot share one either"
        );
        assert_eq!(
            keyed(&all, "chrome", &typed(&[("Profile 1", "w")])).expect("its own key"),
            BTreeMap::from([("Profile 1".to_owned(), 'W')])
        );

        let mut hidden = browser("edge", &[]);
        hidden.hidden = true;
        hidden.keys.insert(String::new(), 'H');
        let mut lower = browser("brave", &[]);
        lower.keys.insert(String::new(), 'l');
        let others = [hidden, lower];
        assert!(
            keyed(&others, "custom-1", &typed(&[("", "h")])).is_ok(),
            "a hidden browser is not in the picker and holds nothing"
        );
        assert!(
            keyed(&others, "custom-1", &typed(&[("", "L")])).is_err(),
            "a key written in lowercase by hand still holds"
        );

        assert_eq!(
            keyed(&all, "custom-1", &typed(&[("Profile 9", "z")])).expect("ignored"),
            BTreeMap::new(),
            "a row the browser does not have keeps no key"
        );

        let mut plain = browser("firefox", &[]);
        plain.name = "Firefox".to_owned();
        plain.keys.insert(String::new(), 'F');
        assert_eq!(
            keyed(&[plain], "custom-1", &typed(&[("", "F")])).unwrap_err(),
            "keyTaken:Firefox",
            "a browser with no profiles is named alone"
        );
    }
}
