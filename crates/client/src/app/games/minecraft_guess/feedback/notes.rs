use super::state::FeedbackSound;

#[derive(Clone, Copy)]
pub(super) enum Instrument {
    Guitar,
    Harp,
    Pling,
    Bass,
    Bell,
}

#[derive(Clone, Copy)]
pub(super) struct Note {
    pub instrument: Instrument,
    pub delay: f32,
    pub pitch: u8,
    pub volume: f32,
}

impl Note {
    const fn new(instrument: Instrument, delay: f32, pitch: u8, volume: f32) -> Self {
        Self {
            instrument,
            delay,
            pitch,
            volume,
        }
    }

    pub fn speed(self) -> f32 {
        // Match the note block's 0..=24 tuning; 12 plays the original sample.
        2.0_f32.powf((f32::from(self.pitch) - 12.0) / 12.0)
    }
}

impl FeedbackSound {
    pub(super) fn notes(self) -> &'static [Note] {
        match self {
            Self::Submit => const { &[Note::new(Instrument::Guitar, 0.0, 6, 0.50)] },
            Self::Reveal => const { &[Note::new(Instrument::Harp, 0.0, 13, 0.32)] },
            Self::Success => {
                const {
                    &[
                        Note::new(Instrument::Pling, 0.0, 6, 0.42),
                        Note::new(Instrument::Pling, 0.13, 10, 0.46),
                        Note::new(Instrument::Pling, 0.26, 13, 0.50),
                    ]
                }
            }
            Self::Skip => const { &[Note::new(Instrument::Bass, 0.0, 6, 0.38)] },
            Self::Turn => {
                const {
                    &[
                        Note::new(Instrument::Bell, 0.0, 6, 0.22),
                        Note::new(Instrument::Bell, 0.12, 13, 0.28),
                    ]
                }
            }
        }
    }
}
