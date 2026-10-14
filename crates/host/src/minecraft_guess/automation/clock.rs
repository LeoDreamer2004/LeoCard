use std::time::Duration;

#[derive(Clone, Debug, Default)]
pub(in super::super) struct GuessClock {
    elapsed: Duration,
    automatic_wait: Duration,
}

impl GuessClock {
    pub(super) fn remaining(&self, limit: u16) -> Option<u16> {
        if limit == 0 {
            return None;
        }
        let remaining = Duration::from_secs(u64::from(limit)).saturating_sub(self.elapsed);
        Some((remaining.as_secs() + u64::from(remaining.subsec_nanos() != 0)) as u16)
    }

    pub(super) fn automatic_ready(&self, delay: Duration) -> bool {
        self.automatic_wait >= delay
    }

    pub(super) fn restart_automatic_wait(&mut self) {
        self.automatic_wait = Duration::ZERO;
    }

    pub(super) fn advance(&mut self, elapsed: Duration, automatic: bool) {
        self.elapsed = self.elapsed.saturating_add(elapsed);
        self.automatic_wait = if automatic {
            self.automatic_wait.saturating_add(elapsed)
        } else {
            Duration::ZERO
        };
    }
}
