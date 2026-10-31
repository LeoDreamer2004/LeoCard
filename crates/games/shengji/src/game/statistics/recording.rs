use super::{ShengjiBurialStatistics, ShengjiHandStatistics, ShengjiOpeningHandStatistics};
use crate::{ShengjiCard, ShengjiCategory, ShengjiGameState, ShengjiPhase, ShengjiRank};
use leocard_game_common::PlayerId;

impl ShengjiGameState {
    /// Capture the complete hand after every bury/copy/crossing decision has finished.
    pub(in crate::game) fn record_opening_hands(&mut self) {
        let trump = self.trump.expect("playing requires a locked trump");
        for player in &self.players {
            self.statistics.opening[usize::from(player.id.0)] =
                Some(ShengjiOpeningHandStatistics {
                    card_count: player.hand.len() as u8,
                    trump_count: player
                        .hand
                        .iter()
                        .filter(|card| trump.is_trump(**card))
                        .count() as u8,
                    joker_count: player
                        .hand
                        .iter()
                        .filter(|card| {
                            matches!(card.rank(), ShengjiRank::SmallJoker | ShengjiRank::BigJoker)
                        })
                        .count() as u8,
                });
        }
    }

    pub(in crate::game) fn record_burial(&mut self, player: PlayerId, cards: &[ShengjiCard]) {
        self.statistics.burial[usize::from(player.0)] = Some(ShengjiBurialStatistics {
            card_count: cards.len() as u8,
            scoring_card_count: cards.iter().filter(|card| card.points() > 0).count() as u8,
            points: cards.iter().map(|card| card.points()).sum(),
        });
    }

    pub(in crate::game) fn record_first_trick_cut(
        &mut self,
        player: PlayerId,
        previous_winner: Option<PlayerId>,
    ) {
        if !self.history.is_empty() || self.dealer == Some(player) || previous_winner != self.dealer
        {
            return;
        }
        let trick = self
            .trick
            .as_ref()
            .expect("an accepted play belongs to a trick");
        if Some(trick.leader) == self.dealer
            && matches!(trick.lead.category, ShengjiCategory::Suit(_))
            && trick.plays[trick.winner_index].0 == player
            && trick.plays[trick.winner_index].1.category == ShengjiCategory::Trump
        {
            self.statistics.first_trick_cut_dealer[usize::from(player.0)] = true;
        }
    }

    /// This result is public as soon as the dealer is cut; no private hand facts are exposed.
    pub fn first_trick_cut_dealer(&self, player: PlayerId) -> bool {
        self.statistics
            .first_trick_cut_dealer
            .get(usize::from(player.0))
            .copied()
            .unwrap_or(false)
    }

    pub fn hand_statistics(&self, player: PlayerId) -> Option<ShengjiHandStatistics> {
        let ShengjiPhase::Finished(result) = &self.phase else {
            return None;
        };
        let index = usize::from(player.0);
        let opening_hand = self.statistics.opening.get(index).copied().flatten()?;
        let last_trick = self.history.last()?;
        let last_winning_play = last_trick
            .plays
            .iter()
            .find(|(actor, _)| *actor == last_trick.winner)?
            .1
            .clone();
        Some(ShengjiHandStatistics {
            player,
            deck_count: self.rules.deck_count,
            opening_hand,
            burial: self.statistics.burial[index],
            bottom_burier: self.bottom_burier,
            first_trick_cut_dealer: self.statistics.first_trick_cut_dealer[index],
            result: result.clone(),
            last_trick_winner: last_trick.winner,
            last_winning_play,
        })
    }
}
