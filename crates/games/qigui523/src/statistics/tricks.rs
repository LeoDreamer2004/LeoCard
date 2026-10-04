use super::{QiGuiActionContext, QiGuiActionStatistics, QiGuiMatchStatistics, plays::bomb_rank};
use crate::{ActionOutcome, ClassifiedPlay, PlayRecord, QiGuiRank};

impl QiGuiMatchStatistics {
    pub(super) fn observe_trick(
        &mut self,
        before: &QiGuiActionContext,
        outcome: &ActionOutcome,
        play: Option<&ClassifiedPlay>,
        reports: &mut [QiGuiActionStatistics],
    ) {
        let winner = match outcome {
            ActionOutcome::TrickCompleted { winner, .. } => *winner,
            ActionOutcome::GameFinished(_) => play.map_or_else(
                || {
                    before
                        .trick
                        .winning_player()
                        .expect("a completed trick has a winner")
                },
                |_| before.actor,
            ),
            _ => return,
        };
        let mut records = before.trick.records().to_vec();
        if let Some(play) = play {
            records.push(PlayRecord::Played {
                player: before.actor,
                play: play.clone(),
            });
        }
        let played = records
            .iter()
            .filter_map(|record| match record {
                PlayRecord::Played { player, play } => Some((*player, play)),
                PlayRecord::Passed { .. } => None,
            })
            .collect::<Vec<_>>();
        let points = before.trick.table_points() + play.map_or(0, |play| u32::from(play.score()));
        let winner_stats = &mut self.players[winner.0];
        winner_stats.max_trick_points = winner_stats.max_trick_points.max(points);
        winner_stats.first_trick_with_points |= self.completed_tricks == 0 && points > 0;
        winner_stats.won_scoring_bomb |= played.last().is_some_and(|(_, play)| {
            matches!(
                bomb_rank(play.kind()),
                Some(QiGuiRank::Five | QiGuiRank::Ten | QiGuiRank::King)
            )
        });
        let winner_report = &mut reports[winner.0];
        for (_, play) in &played {
            winner_report.capture(play.cards());
        }
        winner_report.captured_points += points;
        for (index, (statistics, run)) in self.players.iter_mut().zip(&mut self.runs).enumerate() {
            run.silent = if played.iter().any(|(player, _)| player.0 == index) {
                0
            } else {
                run.silent + 1
            };
            run.unanswered = if played.len() == 1
                && played[0].0.0 == index
                && before.trick.leader().0 == index
            {
                run.unanswered + 1
            } else {
                0
            };
            statistics.max_silent_tricks = statistics.max_silent_tricks.max(run.silent);
            statistics.max_unanswered_tricks = statistics.max_unanswered_tricks.max(run.unanswered);
        }
        self.completed_tricks += 1;
    }
}
