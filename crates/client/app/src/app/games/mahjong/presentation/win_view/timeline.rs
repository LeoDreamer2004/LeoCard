#[derive(Clone, Copy)]
pub(super) struct WinShowcaseTiming {
    pub reveal_duration: f32,
    pub start: f32,
    pub duration: f32,
}

impl WinShowcaseTiming {
    pub fn local(self, summary_elapsed: f32) -> f32 {
        summary_elapsed + self.reveal_duration - self.start
    }

    pub fn active(self, local: f32) -> bool {
        (0.0..self.duration).contains(&local)
    }
}

pub(super) fn smooth(value: f32) -> f32 {
    let value = value.clamp(0.0, 1.0);
    value * value * (3.0 - 2.0 * value)
}
