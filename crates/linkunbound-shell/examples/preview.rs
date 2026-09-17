#![windows_subsystem = "windows"]

//! `cargo run -p linkunbound-shell --example preview -- [--sheet] [--light] [--en] [--six | --twelve]
//! [--scale=N] [--update | --store | --stage=starting|getting|installing|failed] [--notice]`

use linkunbound_core::Language;
use linkunbound_shell::{HALO, ICON_SIDE, Listed, Picker, TILE_ICON_SIDE, dress, paint};
use slint::ComponentHandle;

fn main() -> Result<(), slint::PlatformError> {
    let args: Vec<String> = std::env::args().collect();
    let flag = |name: &str| args.iter().any(|a| a == name);
    let classic = !flag("--sheet");
    let scale: u32 = args
        .iter()
        .find_map(|a| a.strip_prefix("--scale="))
        .and_then(|v| v.parse().ok())
        .unwrap_or(1);
    let side = if classic { ICON_SIDE } else { TILE_ICON_SIDE } * scale;
    let icons = linkunbound_core::data_dir().join("icons");
    // Real icons at the side the chosen style draws them, so what the preview shows is what
    // the resident will: a 24 px icon stretched to a tile is exactly the blur being checked for.
    #[cfg(windows)]
    let icon = |id: &str| {
        linkunbound_win::installed_browsers()
            .into_iter()
            .find(|b| b.id == id)
            .and_then(|b| linkunbound_win::icon_for(b.icon_source(), &b.id, &icons, side))
            .map(|p| p.to_string_lossy().into_owned())
    };
    #[cfg(target_os = "macos")]
    let icon = |id: &str| {
        linkunbound_mac::installed_browsers()
            .into_iter()
            .find(|b| b.id == id)
            .and_then(|b| linkunbound_mac::icon_for(b.icon_source(), &b.id, &icons, side))
            .map(|p| p.to_string_lossy().into_owned())
    };
    #[cfg(not(any(windows, target_os = "macos")))]
    let icon = |_id: &str| -> Option<String> {
        let _ = (&icons, side);
        None
    };

    let english = flag("--en");
    let named = |es: &str, en: &str| if english { en } else { es }.to_owned();
    let rows = vec![
        Listed {
            browser_id: "google-chrome".to_owned(),
            profile_id: Some("Default".to_owned()),
            name: "Google Chrome".to_owned(),
            profile: named("Tu Chrome", "Personal"),
            icon: icon("google-chrome"),
            can_private: true,
        },
        Listed {
            browser_id: "firefox".to_owned(),
            profile_id: None,
            name: "Mozilla Firefox".to_owned(),
            profile: String::new(),
            icon: icon("firefox-308046b0af4a39cb"),
            can_private: true,
        },
        Listed {
            browser_id: "vivaldi".to_owned(),
            profile_id: None,
            name: "Vivaldi".to_owned(),
            profile: named("Trabajo", "Work"),
            icon: icon("vivaldi-xzttkvpr7ou6s6fgtka6s5fohm"),
            can_private: false,
        },
    ];

    let mut rows = rows;
    if flag("--six") || flag("--twelve") {
        let mut more = rows.clone();
        more[0].profile = named("Cliente", "Client");
        more[2].profile = named("Personal", "Home");
        rows.extend(more);
    }
    if flag("--twelve") {
        let more = rows.clone();
        rows.extend(more);
    }

    let words = if english {
        Language::English
    } else {
        Language::Spanish
    }
    .strings();
    let window = Picker::new()?;
    let notice = linkunbound_shell::Notice::new()?;
    paint(&window, &notice, flag("--light"));
    window.set_classic(classic);
    window.set_halo(if classic { 0.0 } else { HALO });
    dress(
        &window,
        &words,
        "https://docs.google.com/document/d/1a9F/edit",
        Some("slack"),
        &rows,
    );

    let stage = args
        .iter()
        .find_map(|a| a.strip_prefix("--stage="))
        .map(str::to_owned)
        .or_else(|| flag("--installing").then(|| "getting".to_owned()));
    if flag("--update") || flag("--store") || stage.is_some() {
        let looked = linkunbound_core::update::Looked {
            checked_at: Some(1),
            found_version: Some("2.0.1".to_owned()),
            found_route: Some(if flag("--store") { "store" } else { "download" }.to_owned()),
            found_installs: Some(!flag("--store")),
            candidates: None,
        };
        let progress = stage.map(|stage| linkunbound_core::update::Progress {
            version: "2.0.1".to_owned(),
            far: if stage == "getting" { 42 } else { 0 },
            stage,
        });
        let strip = linkunbound_shell::strip_for(&words, &looked, progress.as_ref(), "2.0.0", None);
        linkunbound_shell::show_strip(&window, strip.as_ref());
    }

    let handle = window.as_weak();
    window.on_dismissed(move || {
        if let Some(w) = handle.upgrade() {
            let _ = w.hide();
        }
    });
    window.on_open(|index, private| {
        println!("abrir fila {index}, privada: {private}");
    });
    window.on_copy(|| println!("copiar"));
    window.on_update_asked(|| println!("actualizar"));
    window.on_update_dismissed(|| println!("ahora no"));
    // The keys as the resident reads them: a digit opens its row, the arrows walk the reaches.
    {
        let handle = window.as_weak();
        window.on_typed(move |typed, private| {
            let Some(w) = handle.upgrade() else {
                return false;
            };
            let Some(digit) = typed.chars().next().and_then(|c| c.to_digit(10)) else {
                return false;
            };
            let index = i32::try_from(digit).unwrap_or(0) - 1;
            if index < 0 {
                return false;
            }
            w.invoke_open(index, private);
            true
        });
    }
    {
        let handle = window.as_weak();
        window.on_step_reach(move |delta| {
            use slint::Model;
            let Some(w) = handle.upgrade() else { return 0 };
            let dead: Vec<bool> = w.get_all_reaches().iter().map(|r| r.dead).collect();
            linkunbound_shell::next_live(&dead, w.get_reach_index(), delta)
        });
    }

    if flag("--notice") {
        use linkunbound_core::Strings;
        notice.set_headline(Strings::fill(words.notice_opened, "Mozilla Firefox").into());
        notice.set_reason(Strings::fill(words.notice_by_rule, "docs.google.com").into());
        notice.set_undo_label(words.notice_undo.into());
        notice.set_left(6);
        notice.show()?;
    }

    window.run()
}
