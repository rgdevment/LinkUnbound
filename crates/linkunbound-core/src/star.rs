pub const SETTLED_IN: u64 = 60 * 60 * 24 * 14;
pub const RULES_ENOUGH: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Asking {
    Start,
    Wait,
    Now,
}

#[must_use]
pub fn asking(asked: bool, since: Option<u64>, now: u64, rules: impl FnOnce() -> usize) -> Asking {
    if asked {
        return Asking::Wait;
    }
    let Some(since) = since.filter(|at| *at <= now) else {
        return Asking::Start;
    };
    if now - since < SETTLED_IN {
        return Asking::Wait;
    }
    if rules() < RULES_ENOUGH {
        return Asking::Wait;
    }
    Asking::Now
}

#[cfg(test)]
mod tests {
    use super::{Asking, RULES_ENOUGH, SETTLED_IN, asking};

    const NOW: u64 = 1_800_000_000;

    #[test]
    fn a_copy_that_was_asked_once_is_never_asked_again() {
        assert_eq!(asking(true, Some(0), NOW, || 1000), Asking::Wait);
    }

    #[test]
    fn a_copy_with_no_mark_lays_one_and_says_nothing_yet() {
        assert_eq!(asking(false, None, NOW, || 1000), Asking::Start);
    }

    #[test]
    fn a_mark_from_a_clock_that_was_ahead_is_laid_again_rather_than_waited_on_for_ever() {
        assert_eq!(asking(false, Some(NOW + 1), NOW, || 1000), Asking::Start);
    }

    #[test]
    fn a_fortnight_is_the_floor_and_the_second_before_it_is_not() {
        assert_eq!(
            asking(false, Some(NOW - SETTLED_IN + 1), NOW, || 1000),
            Asking::Wait
        );
        assert_eq!(
            asking(false, Some(NOW - SETTLED_IN), NOW, || 1000),
            Asking::Now
        );
    }

    #[test]
    fn somebody_who_taught_it_nothing_is_not_asked_for_anything() {
        let long_ago = Some(NOW - SETTLED_IN * 2);
        assert_eq!(
            asking(false, long_ago, NOW, || RULES_ENOUGH - 1),
            Asking::Wait
        );
        assert_eq!(asking(false, long_ago, NOW, || RULES_ENOUGH), Asking::Now);
    }

    #[test]
    fn the_rules_are_not_read_when_the_answer_is_known_without_them() {
        let counted = std::cell::Cell::new(0);
        let count = || {
            counted.set(counted.get() + 1);
            1000
        };
        assert_eq!(asking(true, None, NOW, count), Asking::Wait);
        assert_eq!(counted.get(), 0);

        let count = || {
            counted.set(counted.get() + 1);
            1000
        };
        assert_eq!(asking(false, Some(NOW), NOW, count), Asking::Wait);
        assert_eq!(counted.get(), 0);
    }
}
