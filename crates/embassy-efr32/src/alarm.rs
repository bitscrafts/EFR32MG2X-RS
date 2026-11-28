//! The pending alarm: which deadline is the earliest, and the waker parked on it.
//!
//! Embassy calls `schedule_wake` once per pending timer, so several deadlines can be asked for; only
//! the earliest can be armed in one comparator. [`earliest`] decides which, and it is pure, so the
//! decision has host tests instead of a board.

use core::cell::RefCell;
use core::task::Waker;
use critical_section::Mutex;

/// The one deadline currently armed, with the waker to wake when it passes.
static PENDENTE: Mutex<RefCell<Option<(u64, Waker)>>> = Mutex::new(RefCell::new(None));

/// Which deadline should be armed, given what is armed and what was just asked for.
///
/// The earliest wins, and a later request never replaces an earlier one -- replacing it would make
/// the earlier timer fire late by however much the two differ.
pub fn earliest(armed: Option<u64>, requested: u64) -> u64 {
    match armed {
        Some(a) if a <= requested => a,
        _ => requested,
    }
}

/// Parks `waker` for `at`. Returns the deadline to arm when the armed one has to change.
///
/// A request that does not move the deadline returns `None`: the comparator already points at
/// something earlier or equal, and re-arming it would only risk losing the interrupt in the window
/// between writing the value and enabling it.
pub fn store(at: u64, waker: &Waker) -> Option<u64> {
    critical_section::with(|cs| {
        let celula = PENDENTE.borrow(cs);
        let mut guarda = celula.borrow_mut();
        let actual = guarda.as_ref().map(|(d, _)| *d);
        let escolhido = earliest(actual, at);
        if Some(escolhido) == actual {
            // A deadline at least as early already exists: keep the new waker only if it replaces the old one.
            if let Some((d, w)) = guarda.as_mut() {
                if *d > at {
                    *d = at;
                    *w = waker.clone();
                    return Some(at);
                }
            } else {
                *guarda = Some((at, waker.clone()));
                return Some(at);
            }
            return None;
        }
        *guarda = Some((escolhido, waker.clone()));
        Some(escolhido)
    })
}

/// Wakes the parked task if its deadline has passed. Returns true when it woke something.
pub fn take_expired<F: FnOnce() -> u64>(now: F) -> bool {
    let waker = critical_section::with(|cs| {
        let celula = PENDENTE.borrow(cs);
        let mut guarda = celula.borrow_mut();
        match guarda.as_ref() {
            Some((d, _)) if now() >= *d => guarda.take().map(|(_, w)| w),
            _ => None,
        }
    });
    match waker {
        Some(w) => {
            w.wake();
            true
        }
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::earliest;

    #[test]
    fn nothing_armed_takes_the_request() {
        assert_eq!(earliest(None, 500), 500);
    }

    #[test]
    fn an_earlier_armed_deadline_wins() {
        assert_eq!(earliest(Some(100), 500), 100);
    }

    #[test]
    fn an_earlier_request_replaces_a_later_one() {
        assert_eq!(earliest(Some(500), 100), 100);
    }

    #[test]
    fn the_same_moment_does_not_move() {
        assert_eq!(earliest(Some(300), 300), 300);
    }
}
