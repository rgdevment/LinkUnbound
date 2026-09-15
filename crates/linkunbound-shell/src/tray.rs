use std::sync::mpsc::Sender;

use linkunbound_core::Strings;
use muda::{Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder, TrayIconEvent};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Asked {
    Settings,
    Quit,
}

/// The taskbar never recolours what it is handed, so both variants ship. The
/// menu bar does, from the alpha alone, and the dark drawing is the one that
/// carries it: handed the light one it would recolour a picture of nothing.
fn artwork(light_taskbar: bool) -> Option<Icon> {
    let bytes: &[u8] = if light_taskbar || cfg!(target_os = "macos") {
        include_bytes!("../../../app/src-tauri/icons/tray-light-32.png")
    } else {
        include_bytes!("../../../app/src-tauri/icons/tray-dark-32.png")
    };
    let decoded = image_from_png(bytes)?;
    Icon::from_rgba(decoded.2, decoded.0, decoded.1).ok()
}

fn image_from_png(bytes: &[u8]) -> Option<(u32, u32, Vec<u8>)> {
    let decoder = png::Decoder::new(std::io::Cursor::new(bytes));
    let mut reader = decoder.read_info().ok()?;
    let mut buffer = vec![0; reader.output_buffer_size()?];
    let info = reader.next_frame(&mut buffer).ok()?;
    buffer.truncate(info.buffer_size());
    Some((info.width, info.height, buffer))
}

pub struct Tray {
    _icon: TrayIcon,
    settings: MenuItem,
    quit: MenuItem,
}

impl Tray {
    pub fn install(light_taskbar: bool, words: &Strings) -> Option<Self> {
        let settings = MenuItem::new(words.tray_settings, true, None);
        let quit = MenuItem::new(words.tray_quit, true, None);
        let separator = PredefinedMenuItem::separator();
        let menu = Menu::new();
        menu.append_items(&[&settings, &separator, &quit]).ok()?;

        let icon = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip("LinkUnbound")
            .with_icon(artwork(light_taskbar)?)
            .with_icon_as_template(cfg!(target_os = "macos"))
            .with_menu_on_left_click(false)
            .build()
            .ok()?;

        Some(Self {
            _icon: icon,
            settings,
            quit,
        })
    }

    pub fn relabel(&self, words: &Strings) {
        self.settings.set_text(words.tray_settings);
        self.quit.set_text(words.tray_quit);
    }

    pub fn show(&self, visible: bool) {
        let _ = self._icon.set_visible(visible);
    }

    /// A left click means the same as the menu entry, so both drain together.
    pub fn drain(&self, asks: &Sender<Asked>) {
        while let Ok(event) = MenuEvent::receiver().try_recv() {
            let asked = if event.id == self.settings.id() {
                Asked::Settings
            } else if event.id == self.quit.id() {
                Asked::Quit
            } else {
                continue;
            };
            if asks.send(asked).is_err() {
                return;
            }
        }
        while let Ok(event) = TrayIconEvent::receiver().try_recv() {
            if let TrayIconEvent::Click {
                button: tray_icon::MouseButton::Left,
                button_state: tray_icon::MouseButtonState::Up,
                ..
            } = event
                && asks.send(Asked::Settings).is_err()
            {
                return;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::image_from_png;

    #[test]
    fn both_tray_variants_decode_to_something_the_shell_can_show() {
        for bytes in [
            include_bytes!("../../../app/src-tauri/icons/tray-dark-32.png").as_slice(),
            include_bytes!("../../../app/src-tauri/icons/tray-light-32.png").as_slice(),
        ] {
            let (width, height, pixels) = image_from_png(bytes).expect("should decode");
            assert_eq!((width, height), (32, 32));
            assert_eq!(pixels.len(), (width * height * 4) as usize);
        }
    }

    /// A template is recoloured from its alpha, so the drawing handed over has to be the one
    /// that has any: the light-taskbar picture is the dark glyph, and the other is a white one
    /// that would arrive as a silhouette of nothing.
    #[cfg(target_os = "macos")]
    #[test]
    fn the_menu_bar_is_handed_the_drawing_that_carries_the_glyph() {
        let template = super::artwork(false).is_some();
        assert!(template, "the menu bar picture has to decode");

        let light = include_bytes!("../../../app/src-tauri/icons/tray-light-32.png").as_slice();
        let (_, _, pixels) = image_from_png(light).expect("should decode");
        let ink = pixels
            .as_chunks::<4>()
            .0
            .iter()
            .filter(|p| p[3] > 0)
            .count();
        assert!(ink > 0, "a template with no alpha is an empty menu bar");
    }

    #[test]
    fn the_two_variants_are_not_the_same_picture() {
        let dark = include_bytes!("../../../app/src-tauri/icons/tray-dark-32.png").as_slice();
        let light = include_bytes!("../../../app/src-tauri/icons/tray-light-32.png").as_slice();
        assert_ne!(dark, light);
    }
}
