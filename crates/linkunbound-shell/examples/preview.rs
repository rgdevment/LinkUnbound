#![windows_subsystem = "windows"]

//! `cargo run -p linkunbound-shell --example preview`

use linkunbound_core::Language;
use linkunbound_shell::{Listed, Picker, dress};
use slint::ComponentHandle;

fn main() -> Result<(), slint::PlatformError> {
    let icons = std::env::var_os("LOCALAPPDATA")
        .map(std::path::PathBuf::from)
        .unwrap_or_default()
        .join("LinkUnbound")
        .join("icons");
    let icon = |id: &str| {
        let path = icons.join(format!("{id}.png"));
        path.is_file().then(|| path.to_string_lossy().into_owned())
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

    let window = Picker::new()?;
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
