//! 国标麻将的成就条件；只读取实时游戏事实。
use crate::AchievementTrigger;
use leocard_mahjong::{Fan, MahjongMatchLength, MahjongScoreResult};
use leocard_protocol::{GameEvent, MahjongEvent, MahjongHandResultView, MahjongWinView, PlayerId};
use std::collections::BTreeSet;

pub(super) struct MahjongFacts<'a> {
    pub player: PlayerId,
    pub result: &'a MahjongHandResultView,
}
impl<'a> MahjongFacts<'a> {
    pub fn from_trigger(trigger: &'a AchievementTrigger) -> Option<Self> {
        let AchievementTrigger::Game {
            player,
            event: GameEvent::Mahjong(MahjongEvent::HandFinished { result }),
        } = trigger
        else {
            return None;
        };
        Some(Self {
            player: *player,
            result,
        })
    }

    pub fn win(&self) -> Option<&'a MahjongWinView> {
        self.result
            .winners
            .iter()
            .find(|winner| winner.player == self.player)
    }

    pub fn fan_count(&self, predicate: impl Fn(Fan) -> bool) -> u64 {
        self.win().map_or(0, |win| {
            win.score
                .fans
                .iter()
                .filter(|value| predicate(value.fan))
                .map(|value| u64::from(value.count))
                .sum()
        })
    }

    pub fn unique_fans(&self, predicate: impl Fn(Fan) -> bool) -> usize {
        self.win()
            .map_or(0, |win| unique_fans(&win.score, predicate))
    }

    pub fn final_score(&self, length: MahjongMatchLength, threshold: i32) -> bool {
        self.result.match_complete
            && self.result.match_length == length
            && self
                .result
                .match_scores
                .get(usize::from(self.player.0))
                .is_some_and(|score| *score >= threshold)
    }

    pub fn all_draws(&self) -> bool {
        self.result.match_complete
            && self.result.match_length != MahjongMatchLength::SingleHand
            && self.result.match_progress.completed_hands == self.result.match_length.hand_count()
            && self.result.match_progress.completed_hands
                == self.result.match_progress.exhaustive_draws
    }

    pub fn broad_flush(&self) -> bool {
        self.win().is_some_and(|win| win.wait_kind_count >= 5)
            && self.fan_count(|fan| fan == Fan::FullFlush) > 0
    }

    pub fn last_hand_comeback(&self) -> bool {
        if !self.result.match_complete
            || self.result.match_length == MahjongMatchLength::SingleHand
            || self.player.0 >= 4
        {
            return false;
        }
        let before = std::array::from_fn(|index| {
            self.result.match_scores[index] - self.result.deltas[index]
        });
        placement(&before, self.player) == 4
            && placement(&self.result.match_scores, self.player) == 1
    }

    pub fn fed_three(&self) -> bool {
        let winners = &self.result.winners;
        winners.len() == 3
            && winners.iter().all(|win| win.from == Some(self.player))
            && winners
                .iter()
                .map(|win| win.player.0)
                .collect::<BTreeSet<_>>()
                .len()
                == 3
    }
}

fn placement(scores: &[i32; 4], player: PlayerId) -> usize {
    let mut order = [0, 1, 2, 3];
    order.sort_by_key(|index| (std::cmp::Reverse(scores[*index]), *index));
    order
        .iter()
        .position(|index| *index == usize::from(player.0))
        .unwrap()
        + 1
}

fn unique_fans(score: &MahjongScoreResult, predicate: impl Fn(Fan) -> bool) -> usize {
    score
        .fans
        .iter()
        .filter(|value| value.count > 0 && predicate(value.fan))
        .map(|value| value.fan)
        .collect::<BTreeSet<_>>()
        .len()
}

pub(super) fn mahjong_fan(trigger: &AchievementTrigger, fan: Fan) -> bool {
    MahjongFacts::from_trigger(trigger)
        .is_some_and(|facts| facts.fan_count(|value| value == fan) > 0)
}

pub(super) fn mahjong_wins(trigger: &AchievementTrigger) -> u64 {
    u64::from(MahjongFacts::from_trigger(trigger).is_some_and(|facts| facts.win().is_some()))
}

pub(super) fn mahjong_hands(trigger: &AchievementTrigger) -> u64 {
    u64::from(MahjongFacts::from_trigger(trigger).is_some())
}

pub(super) fn mahjong_low_fans(trigger: &AchievementTrigger) -> u64 {
    MahjongFacts::from_trigger(trigger).map_or(0, |facts| {
        facts.fan_count(|fan| fan != Fan::FlowerTiles && fan.points() <= 2)
    })
}
pub(super) const BIG_EIGHT: &[Fan] = &[
    Fan::MixedShiftedChows,
    Fan::MixedTripleChow,
    Fan::MixedStraight,
    Fan::PureStraight,
    Fan::HalfFlush,
    Fan::PureShiftedChows,
    Fan::AllTypes,
    Fan::AllPungs,
];
pub(super) const SMALL_EIGHT: &[Fan] = &[
    Fan::SevenPairs,
    Fan::OutsideHand,
    Fan::KnittedStraight,
    Fan::LesserHonorsAndKnittedTiles,
    Fan::UpperFour,
    Fan::LowerFour,
    Fan::ChickenHand,
    Fan::MeldedHand,
];

pub(super) fn mahjong_group(trigger: &AchievementTrigger, group: &[Fan]) -> u64 {
    u64::from(
        MahjongFacts::from_trigger(trigger)
            .is_some_and(|facts| facts.fan_count(|fan| group.contains(&fan)) > 0),
    )
}

pub(super) fn mahjong_false_win(trigger: &AchievementTrigger) -> u64 {
    u64::from(
        matches!(trigger, AchievementTrigger::Game { player, event: GameEvent::Mahjong(MahjongEvent::FalseWin { player: actor, .. }) } if player == actor),
    )
}

pub(super) fn mahjong_high_win(trigger: &AchievementTrigger) -> u64 {
    u64::from(
        MahjongFacts::from_trigger(trigger)
            .is_some_and(|facts| facts.unique_fans(|fan| fan.points() >= 48) > 0),
    )
}

pub(super) fn mahjong_eight_points(trigger: &AchievementTrigger, one_point_only: bool) -> u64 {
    u64::from(MahjongFacts::from_trigger(trigger).is_some_and(|facts| {
        facts
            .win()
            .is_some_and(|win| win.score.points_without_flowers == 8)
            && if one_point_only {
                facts.unique_fans(|fan| fan != Fan::FlowerTiles && fan.points() == 1) == 8
            } else {
                facts.unique_fans(|fan| fan != Fan::FlowerTiles) >= 6
            }
    }))
}

pub(super) fn mahjong_insufficient_fan(trigger: &AchievementTrigger) -> u64 {
    u64::from(
        matches!(trigger, AchievementTrigger::Game { player, event: GameEvent::Mahjong(MahjongEvent::WinUnavailable { player: actor, points_without_flowers }) } if player == actor && *points_without_flowers < 8),
    )
}
