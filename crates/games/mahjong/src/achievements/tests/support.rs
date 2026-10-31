use super::super::{MahjongAchievementEvent, MahjongHandFacts, MahjongWinFacts, register};
use crate::{
    MahjongFan, MahjongFanValue, MahjongMatchLength, MahjongMatchProgress, MahjongScoreResult,
};
use leocard_achievements::{
    AchievementBook, AchievementContext, AchievementRegistry, AchievementTrigger,
};
use leocard_game_common::PlayerId;

use std::sync::LazyLock;

pub(super) static REGISTRY: LazyLock<AchievementRegistry> = LazyLock::new(|| {
    let mut registry = AchievementRegistry::default();
    register(&mut registry);
    registry
});

#[derive(Default)]
pub(super) struct Progress {
    book: AchievementBook,
    sequence: u128,
}

impl Progress {
    pub(super) fn apply(&mut self, event: MahjongAchievementEvent) {
        self.apply_for(PlayerId(0), event);
    }

    pub(super) fn apply_for(&mut self, player: PlayerId, event: MahjongAchievementEvent) {
        self.sequence += 1;
        self.book.trigger_with_registry(
            &AchievementTrigger::Game {
                player,
                event: event.into(),
            },
            Some(AchievementContext {
                match_id: [1; 16],
                hand_index: Some(self.sequence as u32),
                sequence: self.sequence,
            }),
            self.sequence as u64,
            &REGISTRY,
        );
    }

    pub(super) fn count(&self, id: &str) -> u64 {
        let definition = REGISTRY.by_id(id).unwrap();
        self.book.criterion_count(definition, "progress")
    }

    pub(super) fn achieved(&self, id: &str) -> bool {
        self.book.achieved(REGISTRY.by_id(id).unwrap())
    }
}

pub(super) fn win(
    fans: &[(MahjongFan, u8)],
    flowers: u8,
    from: Option<PlayerId>,
    concealed_hand: bool,
) -> MahjongAchievementEvent {
    let points = fans
        .iter()
        .map(|&(fan, count)| fan.points() * u16::from(count))
        .sum();
    MahjongAchievementEvent::HandFinished {
        result: MahjongHandFacts {
            winners: vec![MahjongWinFacts {
                player: PlayerId(0),
                from,
                concealed_hand,
                score: MahjongScoreResult {
                    fans: fans
                        .iter()
                        .map(|&(fan, count)| MahjongFanValue {
                            fan,
                            count,
                            points: fan.points() * u16::from(count),
                        })
                        .collect(),
                    points_without_flowers: points,
                    flower_points: flowers,
                    total_points: points + u16::from(flowers),
                },
                wait_kind_count: 1,
            }],
            match_length: MahjongMatchLength::SingleHand,
            match_progress: MahjongMatchProgress {
                completed_hands: 1,
                exhaustive_draws: 0,
            },
            deltas: [0; 4],
            match_scores: [0; 4],
            match_complete: true,
        },
    }
}
