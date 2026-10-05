use std::sync::Mutex;
use std::time::{Duration, Instant};

use linkunbound_core::Browser;

const FRESH: Duration = Duration::from_secs(30);

static LAST: Mutex<Remembered> = Mutex::new(Remembered { at: None });

struct Remembered {
    at: Option<(Instant, Vec<Browser>)>,
}

impl Remembered {
    fn take(&mut self, now: Instant, look: impl FnOnce() -> Vec<Browser>) -> Vec<Browser> {
        if let Some((when, found)) = &self.at
            && now.saturating_duration_since(*when) < FRESH
        {
            return found.clone();
        }
        let found = look();
        self.at = Some((now, found.clone()));
        found
    }
}

pub fn catalogue() -> Vec<Browser> {
    let installed = LAST
        .lock()
        .map(|mut last| last.take(Instant::now(), crate::host::browsers))
        .unwrap_or_else(|_| crate::host::browsers());
    let saved = crate::store()
        .browsers()
        .map(|c| c.browsers)
        .unwrap_or_default();
    linkunbound_core::merge(installed, &saved)
}

#[cfg(test)]
mod tests {
    use super::{FRESH, Remembered};
    use std::cell::Cell;
    use std::time::{Duration, Instant};

    #[test]
    fn the_installed_browsers_are_looked_for_again_only_once_they_are_stale() {
        let looked = Cell::new(0);
        let look = || {
            looked.set(looked.get() + 1);
            Vec::new()
        };
        let mut last = Remembered { at: None };
        let start = Instant::now();

        last.take(start, look);
        last.take(start + Duration::from_secs(1), look);
        assert_eq!(looked.get(), 1);

        last.take(start + FRESH, look);
        assert_eq!(looked.get(), 2);
    }
}
