use super::{QiGuiActionContext, QiGuiActionStatistics, QiGuiMatchStatistics};
use crate::{
    BombKind, ClassifiedPlay, PlayComparison, PlayRecord, QiGuiPlayKind, QiGuiRank, QiGuiSuit,
    SameCardPolicy, compare_plays,
};

impl QiGuiMatchStatistics {
    pub(super) fn observe_play(
        &mut self,
        before: &QiGuiActionContext,
        play: Option<&ClassifiedPlay>,
        report: &mut QiGuiActionStatistics,
    ) {
        let progress = &mut self.players[before.actor.0];
        let run = &mut self.runs[before.actor.0];
        let Some(play) = play else {
            run.straight = 0;
            return;
        };
        progress.plays += 1;
        report.played_kind = Some(play.kind().clone());
        report.played_count = play.cards().len() as u16;
        run.straight = if matches!(play.kind(), QiGuiPlayKind::Straight { .. }) {
            run.straight + 1
        } else {
            0
        };
        progress.max_straight_run = progress.max_straight_run.max(run.straight);
        match play.kind() {
            QiGuiPlayKind::Straight { card_count } => {
                progress.straight_plays += 1;
                progress.longest_straight = progress.longest_straight.max(*card_count as u16);
                report.straight_from_four = *card_count >= 5
                    && play
                        .cards()
                        .iter()
                        .any(|card| card.rank() == QiGuiRank::Four);
            }
            QiGuiPlayKind::ConsecutivePairs { pair_count } => {
                progress.consecutive_pair_plays += 1;
                progress.longest_consecutive_pairs =
                    progress.longest_consecutive_pairs.max(*pair_count as u16);
            }
            QiGuiPlayKind::Airplane { triple_count } => {
                progress.airplane_plays += 1;
                progress.longest_airplane = progress.longest_airplane.max(*triple_count as u16);
            }
            QiGuiPlayKind::Bomb(_) => progress.bomb_plays += 1,
            QiGuiPlayKind::HeavenBomb => progress.heaven_bomb_plays += 1,
            _ => {}
        }
        report.late_bomb = before.draw_pile_empty && is_bomb(play.kind());
        report.full_hand_play = report.rules.hand_size >= 8
            && before.hands[before.actor.0].len() == usize::from(report.rules.hand_size)
            && play.cards().len() == before.hands[before.actor.0].len();
        report.diamond_four = matches!(play.kind(), QiGuiPlayKind::Single)
            && play.cards()[0].rank() == QiGuiRank::Four
            && play.cards()[0].suit() == QiGuiSuit::Diamond;
        if let Some(current) = before.trick.winning_play() {
            report.spade_seven_follow = report.rules.same_card_policy == SameCardPolicy::CanFollow
                && matches!(play.kind(), QiGuiPlayKind::Single)
                && matches!(current.kind(), QiGuiPlayKind::Single)
                && [play.cards()[0], current.cards()[0]]
                    .iter()
                    .all(|card| card.rank() == QiGuiRank::Seven && card.suit() == QiGuiSuit::Spade);
            report.heaven_over_heaven = matches!(play.kind(), QiGuiPlayKind::HeavenBomb)
                && matches!(current.kind(), QiGuiPlayKind::HeavenBomb)
                && compare_plays(play, current, &report.rules) == PlayComparison::Greater;
        }
        report.bomb_revenge =
            bomb_rank(play.kind()) == Some(QiGuiRank::Five) && bomb_revenge(before);
    }
}

fn bomb_revenge(before: &QiGuiActionContext) -> bool {
    let mut plays = before
        .trick
        .records()
        .iter()
        .rev()
        .filter_map(|record| match record {
            PlayRecord::Played { player, play } => Some((*player, play)),
            PlayRecord::Passed { .. } => None,
        });
    matches!((plays.next(), plays.next()), (Some((king_player, king)), Some((_, ten)))
        if king_player != before.actor
            && bomb_rank(king.kind()) == Some(QiGuiRank::King)
            && bomb_rank(ten.kind()) == Some(QiGuiRank::Ten))
}

pub(super) fn is_bomb(kind: &QiGuiPlayKind) -> bool {
    matches!(kind, QiGuiPlayKind::Bomb(_) | QiGuiPlayKind::HeavenBomb)
}

pub(super) fn bomb_rank(kind: &QiGuiPlayKind) -> Option<QiGuiRank> {
    match kind {
        QiGuiPlayKind::Bomb(BombKind::OfAKind { rank, .. }) => Some(*rank),
        _ => None,
    }
}
