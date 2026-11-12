use super::super::major::MajorShowcaseSequence;
use super::super::timeline::WinShowcaseTiming;
use bevy::prelude::*;
use leocard_protocol::MatchId;
use std::collections::HashSet;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ShowcaseClip {
    High,
    Background,
    Tiles,
    Glyph,
}

impl ShowcaseClip {
    pub fn sustained(self) -> bool {
        matches!(self, Self::High | Self::Background)
    }
}

#[derive(Clone, Copy)]
pub(super) struct ShowcaseCue {
    pub at: f32,
    pub clip: ShowcaseClip,
    pub volume: f32,
}

#[derive(Component)]
pub(crate) struct ShowcaseAudioStage {
    pub(super) hand: (MatchId, u8),
    pub(super) timing: WinShowcaseTiming,
    cues: Vec<ShowcaseCue>,
}

impl ShowcaseAudioStage {
    pub(in super::super) fn high(hand: (MatchId, u8), timing: WinShowcaseTiming) -> Self {
        Self {
            hand,
            timing,
            cues: vec![ShowcaseCue {
                at: 0.0,
                clip: ShowcaseClip::High,
                volume: 0.88,
            }],
        }
    }

    pub(in super::super) fn major(
        hand: (MatchId, u8),
        timing: WinShowcaseTiming,
        glyphs: usize,
    ) -> Self {
        let mut cues = vec![
            ShowcaseCue {
                at: 0.0,
                clip: ShowcaseClip::Background,
                volume: 0.48,
            },
            ShowcaseCue {
                at: MajorShowcaseSequence::WINNING_TILE_START + MajorShowcaseSequence::TILE_FLIGHT,
                clip: ShowcaseClip::Tiles,
                volume: 0.90,
            },
        ];
        cues.extend((0..glyphs).map(|index| ShowcaseCue {
            at: MajorShowcaseSequence::FIRST_GLYPH
                + index as f32 * MajorShowcaseSequence::GLYPH_INTERVAL
                + MajorShowcaseSequence::GLYPH_FALL,
            clip: ShowcaseClip::Glyph,
            volume: if index + 1 == glyphs { 0.92 } else { 0.78 },
        }));
        Self { hand, timing, cues }
    }

    pub(super) fn due(
        &self,
        local: f32,
        played: &mut HashSet<(u32, usize)>,
    ) -> Vec<(ShowcaseCue, f32)> {
        if !self.timing.active(local) {
            return Vec::new();
        }
        self.cues
            .iter()
            .enumerate()
            .filter_map(|(index, &cue)| {
                let age = local - cue.at;
                if age < 0.0 || !played.insert((self.timing.start.to_bits(), index)) {
                    return None;
                }
                // 断线重入或卡帧时，背景从当前进度续播，过期的撞击不集中补放。
                (cue.clip.sustained() || age <= 0.12).then_some((cue, age))
            })
            .collect()
    }
}
