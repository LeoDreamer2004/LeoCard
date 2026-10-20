use fastrand::Rng;

/// Sampling policy, separate from the lobby's fixed-point representation.
#[derive(Clone, Copy)]
pub(super) struct Temperature(f32);

impl Temperature {
    pub fn new(tenths: u8) -> Self {
        Self(f32::from(tenths) / 10.0)
    }

    pub fn blend(self, cold: f32, hot: f32) -> f32 {
        cold + (hot - cold) * self.0
    }

    pub fn degree_limit(self) -> usize {
        self.blend(4.0, 10.0).round() as usize
    }

    pub fn path_limit(self) -> usize {
        self.blend(2.0, 6.0).round() as usize
    }

    pub fn leaf_limit(self, size: usize) -> usize {
        (size as f32 * 0.25 * self.0 * self.0).floor() as usize
    }

    pub fn branch(self, random: &mut Rng) -> bool {
        random.f32() < self.0 * self.0 * 0.4
    }

    pub fn ring_size(self, random: &mut Rng) -> usize {
        self.choose(3..=12, |size| -(*size as f32 - 4.0).powi(2) * 0.25, random)
            .expect("ring sizes")
    }

    pub fn density(self, random: &mut Rng) -> f32 {
        self.blend(1.6, 1.15 + random.f32() * 0.65)
    }

    /// Gumbel-max is a bounded, single-pass Boltzmann draw without exponent overflow.
    pub fn choose<T>(
        self,
        values: impl IntoIterator<Item = T>,
        score: impl Fn(&T) -> f32,
        random: &mut Rng,
    ) -> Option<T> {
        let mut best = None;
        let mut maximum = f32::NEG_INFINITY;
        for value in values {
            let uniform = random.f32().clamp(f32::EPSILON, 1.0 - f32::EPSILON);
            let perturbed = score(&value) - self.blend(0.08, 3.5) * (-uniform.ln()).ln();
            if perturbed > maximum {
                best = Some(value);
                maximum = perturbed;
            }
        }
        best
    }
}
