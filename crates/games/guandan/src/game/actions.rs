use super::{
    GuandanAction, GuandanActionOutcome, GuandanGameError, GuandanGameResult, GuandanGameState,
    GuandanPhase,
};
use crate::{GuandanCard, can_beat, classify};
use leocard_game_common::PlayerId;
use std::array;

impl GuandanGameState {
    pub fn act(
        &mut self,
        player: PlayerId,
        action: GuandanAction,
    ) -> Result<GuandanActionOutcome, GuandanGameError> {
        if player.index() >= self.players.len() {
            return Err(GuandanGameError::InvalidPlayer(player));
        }
        match (&self.phase, action) {
            (GuandanPhase::Finished(_), _) => Err(GuandanGameError::GameAlreadyFinished),
            (GuandanPhase::Tribute, GuandanAction::Tribute(card)) => {
                self.submit_tribute(player, card)
            }
            (GuandanPhase::Tribute, GuandanAction::ReturnTribute(card)) => {
                self.return_tribute(player, card)
            }
            (GuandanPhase::Playing, GuandanAction::Play(cards)) => self.play_cards(player, &cards),
            (GuandanPhase::Playing, GuandanAction::Pass) => self.pass(player),
            _ => Err(GuandanGameError::WrongPhase),
        }
    }

    pub fn play_cards(
        &mut self,
        player: PlayerId,
        cards: &[GuandanCard],
    ) -> Result<GuandanActionOutcome, GuandanGameError> {
        self.ensure_playing_turn(player)?;
        let play = classify(cards, self.rules.starting_level)?;
        for card in cards {
            self.ensure_card_in_hand(player, *card)?;
        }
        if let Some(current) = &self.winning_play
            && !can_beat(&play, current, self.rules.starting_level)
        {
            return Err(GuandanGameError::PlayDoesNotBeatCurrent);
        }
        let hand = &mut self.players[player.index()].hand;
        for card in cards {
            let position = hand
                .iter()
                .position(|candidate| candidate == card)
                .expect("validated card");
            hand.remove(position);
        }
        self.played_cards.extend_from_slice(cards);
        self.winning_player = Some(player);
        self.winning_play = Some(play.clone());
        self.passes = 0;
        self.last_actions[player.index()] = Some(GuandanAction::Play(cards.to_vec()));
        if self.players[player.index()].hand.is_empty() {
            self.finishing_order.push(player);
            if self.finishing_order.len() == 3
                || (self.finishing_order.len() == 2
                    && self.finishing_order[0].0 % 2 == player.0 % 2)
            {
                let result = self.finish_game();
                return Ok(GuandanActionOutcome::GameFinished(result));
            }
        }
        let next_player = self.next_active_after(player);
        self.current_player = next_player;
        Ok(GuandanActionOutcome::Played {
            player,
            play,
            next_player,
        })
    }

    pub fn pass(&mut self, player: PlayerId) -> Result<GuandanActionOutcome, GuandanGameError> {
        self.ensure_playing_turn(player)?;
        if self.winning_play.is_none() {
            return Err(GuandanGameError::MustLeadWithCards);
        }
        self.last_actions[player.index()] = Some(GuandanAction::Pass);
        self.passes += 1;
        let winner = self.winning_player.expect("winning play has a player");
        let active = self
            .players
            .iter()
            .filter(|player| !player.hand.is_empty())
            .count();
        let required_passes = active - usize::from(!self.players[winner.index()].hand.is_empty());
        if self.passes >= required_passes {
            let leader = if self.players[winner.index()].hand.is_empty() {
                let partner = PlayerId((winner.0 + 2) % 4);
                if self.players[partner.index()].hand.is_empty() {
                    self.next_active_after(winner)
                } else {
                    partner
                }
            } else {
                winner
            };
            self.current_player = leader;
            self.winning_player = None;
            self.winning_play = None;
            self.passes = 0;
            self.last_actions = array::from_fn(|_| None);
            return Ok(GuandanActionOutcome::TrickCompleted {
                winner,
                next_player: leader,
            });
        }
        let next_player = self.next_active_after(player);
        self.current_player = next_player;
        Ok(GuandanActionOutcome::Passed {
            player,
            next_player,
        })
    }

    fn ensure_playing_turn(&self, player: PlayerId) -> Result<(), GuandanGameError> {
        if player.index() >= self.players.len() {
            return Err(GuandanGameError::InvalidPlayer(player));
        }
        if !matches!(self.phase, GuandanPhase::Playing) {
            return Err(if matches!(self.phase, GuandanPhase::Finished(_)) {
                GuandanGameError::GameAlreadyFinished
            } else {
                GuandanGameError::WrongPhase
            });
        }
        if self.current_player != player {
            return Err(GuandanGameError::NotPlayersTurn {
                expected: self.current_player,
                actual: player,
            });
        }
        Ok(())
    }

    fn next_active_after(&self, player: PlayerId) -> PlayerId {
        (1..=4)
            .map(|offset| PlayerId((player.0 + offset) % 4))
            .find(|next| !self.players[next.index()].hand.is_empty())
            .expect("unfinished game has an active player")
    }

    fn finish_game(&mut self) -> GuandanGameResult {
        let mut order = self.finishing_order.clone();
        for player in 0..4 {
            let id = PlayerId(player);
            if !order.contains(&id) {
                order.push(id);
            }
        }
        let finishing_order: [PlayerId; 4] = order.try_into().expect("four players ranked");
        let winner = finishing_order[0];
        let partner_position = finishing_order
            .iter()
            .position(|player| player.0 % 2 == winner.0 % 2 && *player != winner)
            .expect("winning player has a partner");
        let result = GuandanGameResult {
            known_positions: if self.finishing_order.len() == 2 {
                2
            } else {
                4
            },
            finishing_order,
            winning_team: winner.0 % 2,
            level_gain: (4 - partner_position) as u8,
        };
        self.phase = GuandanPhase::Finished(result.clone());
        result
    }
}
