#![windows_subsystem = "windows"]

//! `cargo run -p linkunbound-shell --example preview -- [--classic] [--light] [--six]`

use linkunbound_core::Language;
use linkunbound_shell::{HALO, ICON_SIDE, Listed, Picker, TILE_ICON_SIDE, dress, paint};
use slint::ComponentHandle;

fn main() -> Result<(), slint::PlatformError> {
    let args: Vec<String> = std::env::args().collect();
    let flag = |name: &str| args.iter().any(|a| a == name);
    let classic = flag("--classic");
    let side = if classic { ICON_SIDE } else { TILE_ICON_SIDE };
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
            .and_then(|b| linkunbound_mac::icon_for(b.icon_source(), &b.id, &icons, side * 2))
            .map(|p| p.to_string_lossy().into_owned())
    };
    #[cfg(not(any(windows, target_os = "macos")))]
    let icon = |_id: &str| -> Option<String> {
        let _ = (&icons, side);
        None
    };

    let rows = vec![
        Listed {
            browser_id: "google-chrome".to_owned(),
            profile_id: Some("Default".to_owned()),
            name: "Google Chrome".to_owned(),
            profile: "Tu Chrome".to_owned(),
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
            profile: "Trabajo".to_owned(),
            icon: icon("vivaldi-xzttkvpr7ou6s6fgtka6s5fohm"),
            can_private: false,
        },
    ];

    let mut rows = rows;
    if flag("--six") {
        let mut more = rows.clone();
        more[0].profile = "Cliente".to_owned();
        more[2].profile = "Personal".to_owned();
        rows.extend(more);
    }

    let window = Picker::new()?;
    let notice = linkunbound_shell::Notice::new()?;
    paint(&window, &notice, flag("--light"));
    window.set_classic(classic);
    window.set_halo(if classic { 0.0 } else { HALO });
    dress(
        &window,
        &Language::Spanish.strings(),
        "https://docs.google.com/document/d/1a9F/edit",
        Some("tisty-gui"),
        &rows,
    );

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

    window.run()
}
