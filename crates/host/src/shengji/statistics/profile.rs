use super::super::{ShengjiSession, record_shengji_component};
use leocard_shengji::{
    ActionOutcome, GameState, ShengjiBidKind, ShengjiClassifiedPlay, ShengjiPlayerId,
};

impl ShengjiSession {
    pub(in crate::shengji) fn record_current_declaration(&mut self) {
        let Some((player, kind)) = self
            .game
            .as_ref()
            .and_then(|game| game.bidding().current())
            .map(|declaration| (declaration.player, declaration.kind))
        else {
            return;
        };
        let Some(stats) = self.statistics.hand.profiles.get_mut(usize::from(player.0)) else {
            return;
        };
        match kind {
            ShengjiBidKind::Initial => stats.declaration_games = 1,
            ShengjiBidKind::Counter | ShengjiBidKind::SelfCounter => stats.counter_games = 1,
            ShengjiBidKind::Protect => {}
        }
    }

    pub(in crate::shengji) fn record_profile_outcome(&mut self, outcome: &ActionOutcome) {
        match outcome {
            ActionOutcome::BottomCopyDecision {
                player,
                copied: true,
                ..
            } => {
                if let Some(stats) = self.statistics.hand.profiles.get_mut(usize::from(player.0)) {
                    stats.counter_games = 1;
                }
            }
            ActionOutcome::FiveTrumpCrossingDecision {
                player,
                crossing: true,
                ..
            } => {
                if let Some(stats) = self.statistics.hand.profiles.get_mut(usize::from(player.0)) {
                    stats.crossing_games = 1;
                }
            }
            ActionOutcome::Played { player, .. } | ActionOutcome::ThrowFailed { player, .. } => {
                let play =
                    self.game
                        .as_ref()
                        .and_then(GameState::current_trick)
                        .and_then(|trick| {
                            trick.plays.last().map(|(_, play)| {
                                (play.clone(), trick.leader == *player, trick.winner)
                            })
                        });
                if let Some((play, is_lead, winner)) = play {
                    self.record_profile_play(*player, &play, is_lead, winner == *player);
                }
            }
            ActionOutcome::TrickComplete(trick) => {
                if let Some((player, play)) = trick.plays.last() {
                    self.record_profile_play(
                        *player,
                        play,
                        trick.leader == *player,
                        trick.winner == *player,
                    );
                }
            }
            ActionOutcome::HandComplete(_) => {
                let play = self
                    .game
                    .as_ref()
                    .and_then(|game| game.history().last())
                    .and_then(|trick| {
                        trick.plays.last().map(|(player, play)| {
                            (*player, play.clone(), trick.leader, trick.winner)
                        })
                    });
                if let Some((player, play, leader, winner)) = play {
                    self.record_profile_play(player, &play, leader == player, winner == player);
                }
            }
            _ => {}
        }
    }

    pub(in crate::shengji) fn record_profile_play(
        &mut self,
        player: ShengjiPlayerId,
        play: &ShengjiClassifiedPlay,
        is_lead: bool,
        is_winning: bool,
    ) {
        let Some(stats) = self.statistics.hand.profiles.get_mut(usize::from(player.0)) else {
            return;
        };
        stats.plays = stats.plays.saturating_add(1);
        if is_winning {
            stats.winning_plays = stats.winning_plays.saturating_add(1);
        }
        if is_lead && play.is_throw() {
            stats.play_category_counts[4] = stats.play_category_counts[4].saturating_add(1);
            stats.longest_throw = stats
                .longest_throw
                .max(play.cards.len().min(usize::from(u16::MAX)) as u16);
            for component in &play.components {
                record_shengji_component(stats, component);
            }
        } else {
            record_shengji_component(stats, play.strongest_component());
        }
    }
}
