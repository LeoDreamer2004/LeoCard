use leocard_protocol::PlayerInteractionKind;
use std::time::Duration;

const EGG_COUNT: u16 = PlayerInteractionKind::EggBarrage.egg_count() as u16;
const PHRASE_HITS: u16 = 12;
const FILL_START: u16 = 84;
const STEP_MICROS: u64 = 48_000;
const PICKUP_MICROS: u64 = 80_000;

// 4/4 phrases on a thirty-second-note grid. Gaps are rests, close notes are flams.
const PHRASES: [[u8; 12]; 6] = [
    [0, 4, 6, 8, 10, 12, 16, 18, 20, 24, 27, 30],
    [0, 3, 4, 8, 11, 12, 16, 20, 22, 24, 28, 31],
    [0, 4, 7, 8, 12, 14, 16, 19, 20, 24, 28, 30],
    [0, 2, 4, 8, 12, 15, 16, 20, 24, 26, 28, 31],
    [0, 2, 4, 6, 8, 10, 12, 16, 18, 20, 22, 24],
    [0, 4, 6, 8, 10, 12, 16, 20, 22, 24, 25, 26],
];
const FINAL_FILL: [u8; 16] = [0, 4, 8, 12, 16, 18, 20, 22, 24, 25, 26, 27, 28, 29, 30, 31];

pub(super) struct BarrageBeat {
    pub at: Duration,
    pub kind: PlayerInteractionKind,
    pub gain: f32,
}

pub(super) struct EggBarrageTimeline {
    pub emitted: u16,
    started_at: Duration,
    seed: u32,
}

impl EggBarrageTimeline {
    pub fn new(now: Duration, seed: u32) -> Self {
        Self {
            emitted: 0,
            started_at: now,
            seed,
        }
    }

    pub fn next_beat(&self) -> BarrageBeat {
        if self.emitted == EGG_COUNT {
            return BarrageBeat {
                at: Duration::from_micros(PICKUP_MICROS + 255 * STEP_MICROS)
                    + Duration::from_millis(500),
                kind: PlayerInteractionKind::Shoe,
                gain: 1.3,
            };
        }
        let fill = self.emitted >= FILL_START;
        let (bar, note, position) = if fill {
            let note = usize::from(self.emitted - FILL_START);
            (7, note, FINAL_FILL[note])
        } else {
            let bar = self.emitted / PHRASE_HITS;
            let note = usize::from(self.emitted % PHRASE_HITS);
            let phrase = (self.seed.rotate_left(u32::from(bar) * 5) ^ (u32::from(bar) * 17))
                as usize
                % PHRASES.len();
            (bar, note, PHRASES[phrase][note])
        };
        let gain = if fill && note > 0 {
            0.40 + 0.70 * note as f32 / 15.0
        } else if position % 8 == 0 {
            [1.15, 0.60, 0.92, 0.72][usize::from(position / 8)]
        } else if position % 4 == 0 {
            0.48
        } else if position % 2 == 0 {
            0.34
        } else {
            0.24
        };
        let swing = if !fill && position % 8 == 4 { 8_000 } else { 0 };
        BarrageBeat {
            at: Duration::from_micros(
                PICKUP_MICROS + (u64::from(bar) * 32 + u64::from(position)) * STEP_MICROS + swing,
            ),
            kind: PlayerInteractionKind::Egg,
            gain,
        }
    }

    pub fn ready(&self, now: Duration) -> bool {
        !self.finished() && now.saturating_sub(self.started_at) >= self.next_beat().at
    }

    pub fn advance(&mut self, now: Duration) {
        let scheduled = self.next_beat().at;
        self.emitted += 1;
        // Stay on the score's grid during normal frames. After a hitch, pause the
        // score rather than dumping overdue sounds or permanently losing its groove.
        let elapsed = now.saturating_sub(self.started_at);
        if !self.finished() && elapsed >= self.next_beat().at {
            self.started_at += elapsed.saturating_sub(scheduled);
        }
    }

    pub fn finished(&self) -> bool {
        self.emitted > EGG_COUNT
    }
}
