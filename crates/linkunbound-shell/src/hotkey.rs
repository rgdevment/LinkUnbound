use global_hotkey::hotkey::HotKey;
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};

pub struct Hotkey {
    manager: GlobalHotKeyManager,
    held: Option<HotKey>,
}

impl Hotkey {
    #[must_use]
    pub fn new() -> Option<Self> {
        GlobalHotKeyManager::new().ok().map(|manager| Self {
            manager,
            held: None,
        })
    }

    pub fn claim(&mut self, wanted: Option<&str>) -> Option<String> {
        if let Some(held) = self.held.take() {
            let _ = self.manager.unregister(held);
        }
        let spelled = wanted?;
        let parsed: HotKey = spelled.parse().ok()?;
        self.manager.register(parsed).ok()?;
        self.held = Some(parsed);
        Some(spelled.to_owned())
    }

    pub fn pressed(&self) -> bool {
        let mut asked = false;
        while let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
            if event.state() == HotKeyState::Pressed
                && self.held.is_some_and(|held| held.id() == event.id())
            {
                asked = true;
            }
        }
        asked
    }
}

impl Drop for Hotkey {
    fn drop(&mut self) {
        if let Some(held) = self.held.take() {
            let _ = self.manager.unregister(held);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::HotKey;

    #[test]
    fn the_spellings_the_core_can_produce_all_parse() {
        for said in [
            "Alt+Shift+L",
            "Ctrl+Shift+L",
            "Control+Alt+Digit1",
            "F9",
            "Super+L",
        ] {
            assert!(said.parse::<HotKey>().is_ok(), "{said} should parse");
        }
    }

    #[test]
    fn nonsense_is_refused_rather_than_claiming_something_else() {
        assert!("".parse::<HotKey>().is_err());
        assert!("Shift".parse::<HotKey>().is_err());
    }
}
