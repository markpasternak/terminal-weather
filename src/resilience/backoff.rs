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
