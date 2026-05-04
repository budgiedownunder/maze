use std::time::Duration;

/// Bounded retry policy applied to a single send when the provider returns a
/// transient error. This is intentionally not a job queue — it covers the
/// "blip" case (5xx response, connection reset). Anything more durable belongs
/// in the consumer's own retry/queue layer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetryPolicy {
    /// Total attempts including the first one. `1` disables retry.
    pub max_attempts: u32,
    /// Backoff before the second attempt. Doubles up to `max_backoff`.
    pub initial_backoff: Duration,
    /// Cap on backoff between attempts.
    pub max_backoff: Duration,
    /// Multiplier applied to the previous backoff on each retry.
    pub backoff_multiplier: u32,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_backoff: Duration::from_millis(100),
            max_backoff: Duration::from_secs(10),
            backoff_multiplier: 2,
        }
    }
}

impl RetryPolicy {
    /// A policy that performs no retries (the first attempt is the only attempt).
    pub fn no_retry() -> Self {
        Self {
            max_attempts: 1,
            initial_backoff: Duration::ZERO,
            max_backoff: Duration::ZERO,
            backoff_multiplier: 1,
        }
    }

    /// Compute the backoff duration before attempt number `attempt` (1-indexed).
    /// Attempt 1 is always immediate (returns `Duration::ZERO`).
    pub fn backoff_before_attempt(&self, attempt: u32) -> Duration {
        if attempt <= 1 {
            return Duration::ZERO;
        }
        let mut backoff = self.initial_backoff;
        for _ in 2..attempt {
            backoff = backoff.saturating_mul(self.backoff_multiplier);
            if backoff > self.max_backoff {
                backoff = self.max_backoff;
                break;
            }
        }
        backoff.min(self.max_backoff)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn default_policy_has_three_attempts_and_exponential_backoff() {
        let p = RetryPolicy::default();
        assert_eq!(p.max_attempts, 3);
        assert_eq!(p.initial_backoff, Duration::from_millis(100));
        assert_eq!(p.max_backoff, Duration::from_secs(10));
        assert_eq!(p.backoff_multiplier, 2);
    }

    #[test]
    fn no_retry_policy_disables_retries() {
        let p = RetryPolicy::no_retry();
        assert_eq!(p.max_attempts, 1);
        assert_eq!(p.backoff_before_attempt(1), Duration::ZERO);
    }

    #[test]
    fn backoff_grows_exponentially_then_caps() {
        let p = RetryPolicy::default();
        assert_eq!(p.backoff_before_attempt(1), Duration::ZERO);
        assert_eq!(p.backoff_before_attempt(2), Duration::from_millis(100));
        assert_eq!(p.backoff_before_attempt(3), Duration::from_millis(200));
        // Far-future attempt caps at max_backoff.
        assert_eq!(p.backoff_before_attempt(50), Duration::from_secs(10));
    }
}
