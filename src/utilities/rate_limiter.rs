use std::time::{Duration, Instant};

/// A rate limiter for controlling the frequency of actions.
///
/// This struct is used to limit the rate at which a particular section of code is executed.
/// It tracks the number of times an action is performed within a specified duration
/// and allows controlling whether an action should wait based on this rate limit.
///
/// # Examples
///
/// ```
/// use std::time::Duration;
/// let mut rate_limiter = RateLimiter::new(4, Duration::new(1, 0));
///
/// for _ in 0..4 {
///     assert_eq!(rate_limiter.should_wait(), false);
/// }
/// assert_eq!(rate_limiter.should_wait(), true);
/// ```
struct RateLimiter {
    last_call: Instant,
    count: usize,
    rate_limit: usize,
    duration: Duration
}

impl RateLimiter {
    /// Creates a new instance of `RateLimiter`.
    ///
    /// This method initializes a rate limiter with a specified rate limit and duration.
    ///
    /// # Arguments
    ///
    /// * `rate_limit` - The maximum number of allowed calls to `should_wait` within the duration.
    /// * `duration` - The time window in which the number of calls to `should_wait` is limited.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    /// let rate_limiter = RateLimiter::new(4, Duration::new(1, 0));
    /// ```
    pub fn new(rate_limit: usize, duration: Duration) -> Self {
        RateLimiter {
            last_call: Instant::now(),
            count: 0,
            rate_limit,
            duration
        }
    }

    /// Determines if a call should wait based on the rate limit.
    ///
    /// This method checks if the number of calls made to `should_wait` has reached the rate limit
    /// within the specified duration. If the rate limit is reached, it returns `true`, indicating
    /// that the call should wait. Otherwise, it returns `false`.
    ///
    /// The count and last call time are reset if the duration has elapsed since the last call.
    ///
    /// # Returns
    ///
    /// Returns `true` if the call should wait, or `false` if it can proceed immediately.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::{Duration, Instant};
    /// let mut rate_limiter = RateLimiter::new(4, Duration::new(1, 0));
    ///
    /// for _ in 0..4 {
    ///     assert_eq!(rate_limiter.should_wait(), false);
    /// }
    /// assert_eq!(rate_limiter.should_wait(), true); // Now it should wait
    /// ```
    pub fn should_wait(&mut self) -> bool {
        if self.last_call.elapsed() >= self.duration {
            self.last_call = Instant::now();
            self.count = 1;
            false
        } else if self.count < self.rate_limit {
            self.count += 1;
            false
        } else {
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_initial_call() {
        let mut rate_limiter = RateLimiter::new(4, Duration::new(1, 0));
        assert_eq!(rate_limiter.should_wait(), false);
    }

    #[test]
    fn test_calls_within_limit_and_duration() {
        let mut rate_limiter = RateLimiter::new(4, Duration::new(1, 0));
        for _ in 0..3 {
            assert_eq!(rate_limiter.should_wait(), false);
        }
    }

    #[test]
    fn test_reaching_rate_limit() {
        let mut rate_limiter = RateLimiter::new(4, Duration::new(1, 0));

        for _ in 0..4 {
            assert_eq!(rate_limiter.should_wait(), false);
        }
        assert_eq!(rate_limiter.should_wait(), true);
    }

    #[test]
    fn test_reset_after_duration() {
        let mut rate_limiter = RateLimiter::new(4, Duration::new(1, 0));
        for _ in 0..4 {
            rate_limiter.should_wait();
        }

        thread::sleep(Duration::new(1, 10));

        assert_eq!(rate_limiter.should_wait(), false);
    }
}