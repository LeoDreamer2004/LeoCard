use super::{BlackjackGameState, BlackjackHandResult, BlackjackHandStatus, BlackjackPhase};
use crate::BlackjackHandValue;

impl BlackjackGameState {
    pub(super) fn settle(&mut self) {
        let dealer_total = BlackjackHandValue::of(&self.dealer).total;
        let dealer_blackjack = self.dealer.len() == 2 && dealer_total == 21;
        for player in &mut self.players {
            if player.hands.is_empty() {
                continue;
            }
            player.statistics.rounds += 1;
            for hand in &mut player.hands {
                let natural = hand.status == BlackjackHandStatus::Blackjack;
                let total = hand.value().total;
                let result = if total > 21 {
                    BlackjackHandResult::Loss
                } else if natural && dealer_blackjack {
                    BlackjackHandResult::Push
                } else if dealer_blackjack {
                    BlackjackHandResult::Loss
                } else if natural {
                    BlackjackHandResult::Blackjack
                } else if dealer_total > 21 || total > dealer_total {
                    BlackjackHandResult::Win
                } else if total == dealer_total {
                    BlackjackHandResult::Push
                } else {
                    BlackjackHandResult::Loss
                };
                let returned = match result {
                    BlackjackHandResult::Loss => 0,
                    BlackjackHandResult::Push => hand.wager,
                    BlackjackHandResult::Win => hand.wager * 2,
                    BlackjackHandResult::Blackjack => hand.wager * 5 / 2,
                };
                player.stack += returned;
                hand.net_chips = returned as i32 - hand.wager as i32;
                hand.result = Some(result);
                player.statistics.hands += 1;
                player.statistics.wins += u32::from(matches!(
                    result,
                    BlackjackHandResult::Win | BlackjackHandResult::Blackjack
                ));
                player.statistics.blackjacks += u32::from(natural);
                player.statistics.busts += u32::from(total > 21);
            }
        }
        self.phase = BlackjackPhase::Complete;
        self.current = None;
    }
}
