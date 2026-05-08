use chrono::{DateTime, Utc};

/// Source of "now". Token-source caches depend on this so tests can advance
/// time deterministically (via `TestClock`) instead of sleeping.
pub trait Clock: Send + Sync {
    fn now(&self) -> DateTime<Utc>;
}

/// `Clock` backed by `chrono::Utc::now()`. The default for production use.
#[derive(Debug, Default, Clone, Copy)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}

/// Manually advanced clock for tests. Stores a `DateTime<Utc>`; `now()`
/// returns it, `advance()` mutates it.
#[cfg(test)]
pub(crate) struct TestClock {
    now: std::sync::Mutex<DateTime<Utc>>,
}

#[cfg(test)]
impl TestClock {
    pub(crate) fn new(start: DateTime<Utc>) -> Self {
        Self {
            now: std::sync::Mutex::new(start),
        }
    }

    pub(crate) fn advance(&self, delta: chrono::Duration) {
        let mut g = self.now.lock().expect("test clock poisoned");
        *g += delta;
    }
}

#[cfg(test)]
impl Clock for TestClock {
    fn now(&self) -> DateTime<Utc> {
        *self.now.lock().expect("test clock poisoned")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use pretty_assertions::assert_eq;

    #[test]
    fn system_clock_returns_a_recent_utc_now() {
        let before = Utc::now();
        let clock = SystemClock;
        let observed = clock.now();
        let after = Utc::now();
        assert!(observed >= before && observed <= after);
    }

    #[test]
    fn test_clock_advance_moves_now_forward() {
        let start = Utc.with_ymd_and_hms(2026, 5, 4, 12, 0, 0).unwrap();
        let clock = TestClock::new(start);
        assert_eq!(clock.now(), start);

        clock.advance(chrono::Duration::seconds(120));
        assert_eq!(clock.now(), start + chrono::Duration::seconds(120));
    }
}
