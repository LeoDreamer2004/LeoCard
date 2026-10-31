use super::{
    CurrentTrick, ShengjiActionOutcome, ShengjiGameError, ShengjiGameState, ShengjiHandResult,
    ShengjiPhase, ShengjiTrickRecord, next_player, remove_cards, validate_cards_owned,
};
use crate::play::compare_for_trick;
use crate::{ShengjiCard, ShengjiTeamId, ShengjiTrickPlay, classify_lead, validate_follow};
use leocard_game_common::PlayerId;

impl ShengjiGameState {
    pub fn play_cards(
        &mut self,
        player: PlayerId,
        cards: &[ShengjiCard],
    ) -> Result<ShengjiActionOutcome, ShengjiGameError> {
        if self.phase != ShengjiPhase::Playing {
            return Err(ShengjiGameError::WrongPhase);
        }
        if self.current_player != Some(player) {
            return Err(ShengjiGameError::NotPlayersTurn {
                expected: self.current_player.unwrap(),
                actual: player,
            });
        }
        let trump = self.trump.unwrap();
        let (play, failure) = if let Some(trick) = &self.trick {
            let play = validate_follow(
                &self.players[usize::from(player.0)].hand,
                cards,
                &trick.lead,
                trump,
            )?;
            (play, None)
        } else {
            validate_cards_owned(cards, &self.players[usize::from(player.0)].hand)?;
            let opponents = self
                .players
                .iter()
                .filter(|candidate| candidate.id != player)
                .map(|candidate| candidate.hand.as_slice())
                .collect::<Vec<_>>();
            match classify_lead(cards, trump, &self.rules, &opponents)? {
                ShengjiTrickPlay::Accepted(play) => (play, None),
                ShengjiTrickPlay::ThrowFailed(failure) => {
                    let forced = failure.forced.clone();
                    (forced, Some((failure.penalty_points, cards.to_vec())))
                }
            }
        };

        let previous_winner = self
            .trick
            .as_ref()
            .map(|trick| trick.plays[trick.winner_index].0);
        remove_cards(&mut self.players[usize::from(player.0)].hand, &play.cards);
        let play_points = play.cards.iter().map(|card| card.points()).sum::<u16>();
        let was_new_trick = self.trick.is_none();
        if was_new_trick {
            self.trick = Some(CurrentTrick {
                leader: player,
                lead: play.clone(),
                plays: vec![(player, play.clone())],
                winner_index: 0,
                points: play_points,
            });
        } else {
            let trick = self.trick.as_mut().unwrap();
            let winner_play = &trick.plays[trick.winner_index].1;
            if compare_for_trick(&trick.lead, winner_play, &play, trump).is_gt() {
                trick.winner_index = trick.plays.len();
            }
            trick.points = trick.points.saturating_add(play_points);
            trick.plays.push((player, play.clone()));
        }

        self.record_first_trick_cut(player, previous_winner);

        if let Some((penalty, _)) = &failure {
            self.apply_throw_penalty(ShengjiTeamId::of(player), *penalty);
        }
        let trick_complete =
            self.trick.as_ref().unwrap().plays.len() == usize::from(self.rules.player_count());
        if trick_complete {
            return self.complete_trick();
        }
        let next = PlayerId((player.0 + 1) % self.rules.player_count());
        self.current_player = Some(next);
        if let Some((penalty_points, attempted)) = failure {
            Ok(ShengjiActionOutcome::ThrowFailed {
                player,
                attempted,
                forced: play,
                penalty_points,
                next,
            })
        } else {
            Ok(ShengjiActionOutcome::Played { player, next })
        }
    }

    fn complete_trick(&mut self) -> Result<ShengjiActionOutcome, ShengjiGameError> {
        let trick = self.trick.take().unwrap();
        let winner = trick.plays[trick.winner_index].0;
        if ShengjiTeamId::of(winner) == ShengjiTeamId::of(self.dealer.unwrap()).other() {
            self.collecting_score = self
                .collecting_score
                .saturating_add(u32::from(trick.points));
            self.trick_points = self.trick_points.saturating_add(u32::from(trick.points));
        }
        let record = ShengjiTrickRecord {
            leader: trick.leader,
            plays: trick.plays,
            winner,
            points: trick.points,
        };
        self.history.push(record.clone());
        self.current_player = Some(winner);
        if self.players.iter().all(|player| player.hand.is_empty()) {
            let result = self.finish_hand(winner, &record);
            self.phase = ShengjiPhase::Finished(result.clone());
            self.current_player = None;
            return Ok(ShengjiActionOutcome::HandComplete(result));
        }
        Ok(ShengjiActionOutcome::TrickComplete(record))
    }

    pub(super) fn apply_throw_penalty(&mut self, offender: ShengjiTeamId, points: u16) {
        let dealer_team = ShengjiTeamId::of(self.dealer.unwrap());
        if offender == dealer_team {
            self.collecting_score = self.collecting_score.saturating_add(u32::from(points));
            self.penalty_adjustment = self.penalty_adjustment.saturating_add(i32::from(points));
        } else {
            let before = self.collecting_score;
            self.collecting_score = self.collecting_score.saturating_sub(u32::from(points));
            self.penalty_adjustment = self
                .penalty_adjustment
                .saturating_sub((before - self.collecting_score) as i32);
        }
    }

    pub(super) fn finish_hand(
        &mut self,
        last_winner: PlayerId,
        last_trick: &ShengjiTrickRecord,
    ) -> ShengjiHandResult {
        let dealer = self.dealer.unwrap();
        let dealer_team = ShengjiTeamId::of(dealer);
        let collecting_team = dealer_team.other();
        let kitty_points = self.buried.iter().map(|card| card.points()).sum::<u16>();
        let (kitty_multiplier, kitty_award) = if ShengjiTeamId::of(last_winner) == collecting_team {
            let winner_play = last_trick
                .plays
                .iter()
                .find(|(player, _)| *player == last_winner)
                .map(|(_, play)| play)
                .unwrap();
            let multiplier = winner_play.kitty_multiplier();
            (multiplier, u32::from(kitty_points) * multiplier)
        } else {
            (0, 0)
        };
        self.collecting_score = self.collecting_score.saturating_add(kitty_award);

        let step = self.rules.score_step();
        let takeover = self.rules.takeover_score();
        let (promoted_team, promoted_steps, next_dealer) = if self.collecting_score == 0 {
            (
                dealer_team,
                3,
                PlayerId((dealer.0 + 2) % self.rules.player_count()),
            )
        } else if self.collecting_score < step {
            (
                dealer_team,
                2,
                PlayerId((dealer.0 + 2) % self.rules.player_count()),
            )
        } else if self.collecting_score < takeover {
            (
                dealer_team,
                1,
                PlayerId((dealer.0 + 2) % self.rules.player_count()),
            )
        } else if self.collecting_score < takeover + step {
            (
                collecting_team,
                0,
                next_player(dealer, self.rules.player_count()),
            )
        } else {
            let steps = ((self.collecting_score - takeover) / step) as u8;
            (
                collecting_team,
                steps,
                next_player(dealer, self.rules.player_count()),
            )
        };
        self.teams.promote(
            promoted_team,
            promoted_steps,
            self.rules.mandatory_five_ten_king_ace,
        );
        ShengjiHandResult {
            dealer,
            dealer_team,
            collecting_team,
            trick_points: self.trick_points,
            penalty_adjustment: self.penalty_adjustment,
            kitty_points,
            kitty_multiplier,
            collecting_score: self.collecting_score,
            promoted_team,
            promoted_steps,
            next_dealer,
            levels: self.teams.levels,
        }
    }
}
