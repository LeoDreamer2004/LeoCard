use super::{ActionOutcome, GameError, GameState, Phase, PlayRecord, remove_cards};
use crate::{QiGuiCard, QiGuiPlayerId, can_beat, classify};

impl GameState {
    pub fn play_cards(
        &mut self,
        player: QiGuiPlayerId,
        cards: &[QiGuiCard],
    ) -> Result<ActionOutcome, GameError> {
        self.ensure_turn(player)?;
        let play = classify(cards, &self.rules)?;
        self.ensure_cards_in_hand(player, cards)?;
        if let Some(current) = self
            .trick
            .as_ref()
            .and_then(|trick| trick.winning_play.as_ref())
            && !can_beat(&play, current, &self.rules)
        {
            return Err(GameError::PlayDoesNotBeatCurrent);
        }

        remove_cards(&mut self.players[player.0].hand, cards);
        let trick = self.trick.as_mut().expect("playing games have a trick");
        trick.table_points += u32::from(play.score());
        trick.winning_player = Some(player);
        trick.winning_play = Some(play.clone());
        trick.passes_after_winning_play = 0;
        trick.records.push(PlayRecord::Played { player, play });

        if self.draw_pile.is_empty() && self.players[player.0].hand.is_empty() {
            let result = self.finish_game(player, true);
            return Ok(ActionOutcome::GameFinished(result));
        }
        let next_player = self.next_player(player);
        self.trick
            .as_mut()
            .expect("playing games have a trick")
            .current_player = next_player;
        Ok(ActionOutcome::Played {
            player,
            next_player,
        })
    }

    pub fn pass(&mut self, player: QiGuiPlayerId) -> Result<ActionOutcome, GameError> {
        self.ensure_turn(player)?;
        let trick = self.trick.as_mut().expect("playing games have a trick");
        if trick.winning_play.is_none() {
            return Err(GameError::MustLeadWithCards);
        }
        trick.records.push(PlayRecord::Passed { player });
        trick.passes_after_winning_play += 1;
        if trick.passes_after_winning_play == self.players.len() - 1 {
            return Ok(self.complete_trick());
        }

        let next_player = self.next_player(player);
        self.trick
            .as_mut()
            .expect("playing games have a trick")
            .current_player = next_player;
        Ok(ActionOutcome::Passed {
            player,
            next_player,
        })
    }

    fn ensure_turn(&self, player: QiGuiPlayerId) -> Result<(), GameError> {
        if matches!(self.phase, Phase::Finished(_)) {
            return Err(GameError::GameAlreadyFinished);
        }
        if player.0 >= self.players.len() {
            return Err(GameError::InvalidPlayer(player));
        }
        let expected = self
            .trick
            .as_ref()
            .expect("playing games have a trick")
            .current_player;
        if player != expected {
            return Err(GameError::NotPlayersTurn {
                expected,
                actual: player,
            });
        }
        Ok(())
    }

    fn ensure_cards_in_hand(
        &self,
        player: QiGuiPlayerId,
        cards: &[QiGuiCard],
    ) -> Result<(), GameError> {
        for card in cards {
            if !self.players[player.0].hand.contains(card) {
                return Err(GameError::CardNotInHand(*card));
            }
        }
        Ok(())
    }

    fn next_player(&self, player: QiGuiPlayerId) -> QiGuiPlayerId {
        QiGuiPlayerId((player.0 + self.players.len() - 1) % self.players.len())
    }
}
