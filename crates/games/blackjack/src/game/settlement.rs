use super::{GameState, HandResult, HandStatus, Phase};
use crate::HandValue;

impl GameState {
    pub(super) fn settle(&mut self) {
        let dealer_total = HandValue::of(&self.dealer).total;
        let dealer_blackjack = self.dealer.len() == 2 && dealer_total == 21;
        for player in &mut self.players {
            if player.hands.is_empty() {
                continue;
            }
            player.statistics.rounds += 1;
            for hand in &mut player.hands {
                let natural = hand.status == HandStatus::Blackjack;
                let total = hand.value().total;
                let result = if total > 21 {
                    HandResult::Loss
                } else if natural && dealer_blackjack {
                    HandResult::Push
                } else if dealer_blackjack {
                    HandResult::Loss
                } else if natural {
                    HandResult::Blackjack
                } else if dealer_total > 21 || total > dealer_total {
                    HandResult::Win
                } else if total == dealer_total {
                    HandResult::Push
                } else {
                    HandResult::Loss
                };
                let returned = match result {
                    HandResult::Loss => 0,
                    HandResult::Push => hand.wager,
                    HandResult::Win => hand.wager * 2,
                    HandResult::Blackjack => hand.wager * 5 / 2,
                };
                player.stack += returned;
                hand.net_chips = returned as i32 - hand.wager as i32;
                hand.result = Some(result);
                player.statistics.hands += 1;
                player.statistics.wins +=
                    u32::from(matches!(result, HandResult::Win | HandResult::Blackjack));
                player.statistics.blackjacks += u32::from(natural);
                player.statistics.busts += u32::from(total > 21);
            }
        }
        self.phase = Phase::Complete;
        self.current = None;
    }
}
