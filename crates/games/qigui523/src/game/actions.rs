use super::{
    QiGuiActionOutcome, QiGuiGameError, QiGuiGameState, QiGuiPhase, QiGuiPlayRecord, remove_cards,
};
use crate::{QiGuiCard, can_beat, classify};
use leocard_game_common::PlayerId;

impl QiGuiGameState {
    pub fn play_cards(
        &mut self,
        player: PlayerId,
        cards: &[QiGuiCard],
    ) -> Result<QiGuiActionOutcome, QiGuiGameError> {
        self.ensure_turn(player)?;
        let play = classify(cards, &self.rules)?;
        self.ensure_cards_in_hand(player, cards)?;
        if let Some(current) = self
            .trick
            .as_ref()
            .and_then(|trick| trick.winning_play.as_ref())
            && !can_beat(&play, current, &self.rules)
        {
            return Err(QiGuiGameError::PlayDoesNotBeatCurrent);
        }

        remove_cards(&mut self.players[player.index()].hand, cards);
        self.played_cards.extend_from_slice(cards);
        let trick = self.trick.as_mut().expect("playing games have a trick");
        trick.table_points += u32::from(play.score());
        trick.winning_player = Some(player);
        trick.winning_play = Some(play.clone());
        trick.passes_after_winning_play = 0;
        trick.records.push(QiGuiPlayRecord::Played { player, play });

        if self.draw_pile.is_empty() && self.players[player.index()].hand.is_empty() {
            let result = self.finish_game(player, true);
            return Ok(QiGuiActionOutcome::GameFinished(result));
        }
        let next_player = self.next_player(player);
        self.trick
            .as_mut()
            .expect("playing games have a trick")
            .current_player = next_player;
        Ok(QiGuiActionOutcome::Played {
            player,
            next_player,
        })
    }

    pub fn pass(&mut self, player: PlayerId) -> Result<QiGuiActionOutcome, QiGuiGameError> {
        self.ensure_turn(player)?;
        let trick = self.trick.as_mut().expect("playing games have a trick");
        if trick.winning_play.is_none() {
            return Err(QiGuiGameError::MustLeadWithCards);
        }
        trick.records.push(QiGuiPlayRecord::Passed { player });
        trick.passes_after_winning_play += 1;
        if trick.passes_after_winning_play == self.players.len() - 1 {
            return Ok(self.complete_trick());
        }

        let next_player = self.next_player(player);
        self.trick
            .as_mut()
            .expect("playing games have a trick")
            .current_player = next_player;
        Ok(QiGuiActionOutcome::Passed {
            player,
            next_player,
        })
    }

    fn ensure_turn(&self, player: PlayerId) -> Result<(), QiGuiGameError> {
        if matches!(self.phase, QiGuiPhase::Finished(_)) {
            return Err(QiGuiGameError::GameAlreadyFinished);
        }
        if player.index() >= self.players.len() {
            return Err(QiGuiGameError::InvalidPlayer(player));
        }
        let expected = self
            .trick
            .as_ref()
            .expect("playing games have a trick")
            .current_player;
        if player != expected {
            return Err(QiGuiGameError::NotPlayersTurn {
                expected,
                actual: player,
            });
        }
        Ok(())
    }

    fn ensure_cards_in_hand(
        &self,
        player: PlayerId,
        cards: &[QiGuiCard],
    ) -> Result<(), QiGuiGameError> {
        for card in cards {
            if !self.players[player.index()].hand.contains(card) {
                return Err(QiGuiGameError::CardNotInHand(*card));
            }
        }
        Ok(())
    }

    fn next_player(&self, player: PlayerId) -> PlayerId {
        PlayerId::from_index((player.index() + self.players.len() - 1) % self.players.len())
    }
}
