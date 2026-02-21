#[derive(Debug, Clone)]
pub struct Backoff {
    current: u64,
    base: u64,
    max: u64,
}

impl Backoff {
    #[must_use]
    pub fn new(base: u64, max: u64) -> Self {
        Self {
            current: base,
            base,
            max,
        }
    }

    pub fn next_delay(&mut self) -> u64 {
        let delay = self.current;
        self.current = (self.current * 2).min(self.max);
        delay
    }

    pub fn reset(&mut self) {
        self.current = self.base;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state() {
        let backoff = Backoff::new(100, 1000);
        assert_eq!(backoff.current, 100);
        assert_eq!(backoff.base, 100);
        assert_eq!(backoff.max, 1000);
    }

    #[test]
    fn test_exponential_increase() {
        let mut backoff = Backoff::new(100, 1000);
        assert_eq!(backoff.next_delay(), 100);
        assert_eq!(backoff.next_delay(), 200);
        assert_eq!(backoff.next_delay(), 400);
        assert_eq!(backoff.next_delay(), 800);
    }

    #[test]
    fn test_max_cap() {
        let mut backoff = Backoff::new(100, 500);
        assert_eq!(backoff.next_delay(), 100);
        assert_eq!(backoff.next_delay(), 200);
        assert_eq!(backoff.next_delay(), 400);
        assert_eq!(backoff.next_delay(), 500); // Capped at 500
        assert_eq!(backoff.next_delay(), 500);
    }

    #[test]
    fn test_reset() {
        let mut backoff = Backoff::new(100, 1000);
        backoff.next_delay();
        backoff.next_delay();
        assert_ne!(backoff.current, 100);

        backoff.reset();
        assert_eq!(backoff.current, 100);
        assert_eq!(backoff.next_delay(), 100);
    }

    #[test]
    fn test_mixed_flow() {
        let mut backoff = Backoff::new(100, 400);

        // Start -> Increase
        assert_eq!(backoff.next_delay(), 100);
        assert_eq!(backoff.next_delay(), 200);

        // Cap
        assert_eq!(backoff.next_delay(), 400);
        assert_eq!(backoff.next_delay(), 400);

        // Reset
        backoff.reset();
        assert_eq!(backoff.next_delay(), 100);

        // Increase again
        assert_eq!(backoff.next_delay(), 200);
    }
}
