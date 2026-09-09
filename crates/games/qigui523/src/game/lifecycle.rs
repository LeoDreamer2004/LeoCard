use super::{ActionOutcome, GameResult, GameState, Phase, QiGuiPlayerId, TrickState, sort_hands};

impl GameState {
    pub(super) fn complete_trick(&mut self) -> ActionOutcome {
        let old_trick = self.trick.take().expect("playing games have a trick");
        let winner = old_trick
            .winning_player
            .expect("a trick cannot complete without a play");
        let points = old_trick.table_points;
        self.players[winner.0].score += points;

        let cards_drawn = self.refill_hands_from(winner);
        if self.draw_pile.is_empty()
            && let Some(finisher) = self.first_empty_player_from(winner)
        {
            let result = self.finish_game(finisher, false);
            return ActionOutcome::GameFinished(result);
        }
        self.trick = Some(TrickState::new(winner));
        ActionOutcome::TrickCompleted {
            winner,
            points,
            next_player: winner,
            cards_drawn,
        }
    }

    fn refill_hands_from(&mut self, winner: QiGuiPlayerId) -> usize {
        let mut cards_drawn = 0;
        loop {
            let mut drew_in_cycle = false;
            for offset in 0..self.players.len() {
                let player_index = (winner.0 + offset) % self.players.len();
                if self.players[player_index].hand.len() < usize::from(self.rules.hand_size)
                    && let Some(card) = self.draw_pile.pop_front()
                {
                    self.players[player_index].hand.push(card);
                    cards_drawn += 1;
                    drew_in_cycle = true;
                }
            }
            if !drew_in_cycle
                || self.draw_pile.is_empty()
                || self
                    .players
                    .iter()
                    .all(|player| player.hand.len() >= usize::from(self.rules.hand_size))
            {
                break;
            }
        }
        sort_hands(&mut self.players);
        cards_drawn
    }

    fn first_empty_player_from(&self, from: QiGuiPlayerId) -> Option<QiGuiPlayerId> {
        (0..self.players.len())
            .map(|offset| QiGuiPlayerId((from.0 + offset) % self.players.len()))
            .find(|player| self.players[player.0].hand.is_empty())
    }

    pub(super) fn finish_game(
        &mut self,
        finisher: QiGuiPlayerId,
        collect_table_points: bool,
    ) -> GameResult {
        if collect_table_points {
            let table_points = self.trick.as_ref().map_or(0, |trick| trick.table_points);
            self.players[finisher.0].score += table_points;
        }
        let captured_hand_points = self
            .players
            .iter()
            .flat_map(|player| player.hand.iter())
            .map(|card| u32::from(card.score()))
            .sum();
        self.players[finisher.0].score += captured_hand_points;
        self.trick = None;

        let result = GameResult {
            finisher,
            scores: self.players.iter().map(|player| player.score).collect(),
            captured_hand_points,
        };
        self.phase = Phase::Finished(result.clone());
        result
    }
}
