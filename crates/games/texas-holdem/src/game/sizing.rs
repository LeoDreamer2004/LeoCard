/// Ordinary raises advance from the current bet in multiples of the last full raise.
/// An all-in is a separate action and may fall between these steps.
#[derive(Clone, Copy, Debug)]
pub struct RaiseSizing {
    current: u32,
    increment: u32,
    limit: u32,
}

impl RaiseSizing {
    pub fn new(current: u32, increment: u32, limit: u32) -> Self {
        Self {
            current,
            increment: increment.max(1),
            limit,
        }
    }

    pub fn minimum(self) -> u32 {
        self.current.saturating_add(self.increment)
    }

    pub fn maximum(self) -> Option<u32> {
        let steps = self.limit.saturating_sub(self.current) / self.increment;
        (steps > 0).then(|| self.current + steps * self.increment)
    }

    pub fn contains(self, target: u32) -> bool {
        target >= self.minimum()
            && target <= self.limit
            && (target - self.current).is_multiple_of(self.increment)
    }

    /// Round up to a step, stopping at the largest affordable ordinary raise.
    pub fn normalize(self, target: u32) -> Option<u32> {
        let maximum = self.maximum()?;
        let steps = target
            .clamp(self.minimum(), maximum)
            .saturating_sub(self.current)
            .div_ceil(self.increment);
        Some((self.current + steps * self.increment).min(maximum))
    }
}
