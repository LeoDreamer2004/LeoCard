use super::state::FeedbackSound;
use crate::app::games::GameFeedback;
use std::iter::Copied;
use std::slice::Iter;

#[derive(Clone, Copy)]
pub(crate) enum Instrument {
    Guitar,
    Harp,
    Pling,
    Bass,
    Bell,
}

#[derive(Clone, Copy)]
pub(crate) struct Note {
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

impl GameFeedback for FeedbackSound {
    type Cue = FeedbackNotes;

    fn feedback(&self, _context: ()) -> Self::Cue {
        let notes: &'static [Note] = match self {
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
            Self::NodeSelect => const { &[Note::new(Instrument::Guitar, 0.0, 18, 0.20)] },
            Self::Combo(_) => {
                const {
                    &[
                        Note::new(Instrument::Pling, 0.12, 6, 0.26),
                        Note::new(Instrument::Pling, 0.22, 10, 0.29),
                        Note::new(Instrument::Pling, 0.32, 13, 0.32),
                    ]
                }
            }
            Self::ComboBlocked => {
                const {
                    &[
                        Note::new(Instrument::Bass, 0.0, 6, 0.22),
                        Note::new(Instrument::Bass, 0.08, 3, 0.16),
                    ]
                }
            }
        };
        let shift = match self {
            Self::Combo(count) => count.saturating_sub(1).min(5) as u8 * 2,
            _ => 0,
        };
        FeedbackNotes {
            notes: notes.iter().copied(),
            shift,
        }
    }
}

pub(crate) struct FeedbackNotes {
    notes: Copied<Iter<'static, Note>>,
    shift: u8,
}

impl Iterator for FeedbackNotes {
    type Item = Note;

    fn next(&mut self) -> Option<Self::Item> {
        let mut note = self.notes.next()?;
        note.pitch += self.shift;
        Some(note)
    }
}
