use super::{TexasSession, hand_category_index, record_wager};
use leocard_protocol::TexasEvent;
use leocard_texas_holdem::{TexasAction, TexasPhase, evaluate_player_hand};

impl TexasSession {
    pub(super) fn record_profile_events(&mut self, events: &[TexasEvent]) {
        for event in events {
            if let TexasEvent::HandAnalyzed { player, statistics } = event {
                if !statistics.hole_cards.is_empty()
                    && let Some(stats) = self.match_profile_stats.get_mut(usize::from(player.0))
                {
                    stats.entry_tracked_hands = stats.entry_tracked_hands.saturating_add(1);
                    stats.entered_hands = stats
                        .entered_hands
                        .saturating_add(u32::from(statistics.entered_pot));
                }
                continue;
            }
            let TexasEvent::ActionApplied {
                player,
                action,
                amount,
                ..
            } = event
            else {
                continue;
            };
            let Some(stats) = self.match_profile_stats.get_mut(usize::from(player.0)) else {
                continue;
            };
            match action {
                TexasAction::PostBlind => continue,
                TexasAction::Fold => {
                    stats.voluntary_actions = stats.voluntary_actions.saturating_add(1);
                    stats.hands_folded = stats.hands_folded.saturating_add(1);
                }
                TexasAction::Check => {
                    stats.voluntary_actions = stats.voluntary_actions.saturating_add(1);
                    stats.check_actions = stats.check_actions.saturating_add(1);
                }
                TexasAction::Call => {
                    stats.voluntary_actions = stats.voluntary_actions.saturating_add(1);
                    record_wager(stats, *amount);
                }
                TexasAction::RaiseTo(_) => {
                    stats.voluntary_actions = stats.voluntary_actions.saturating_add(1);
                    stats.raise_actions = stats.raise_actions.saturating_add(1);
                    record_wager(stats, *amount);
                }
                TexasAction::AllIn => {
                    stats.voluntary_actions = stats.voluntary_actions.saturating_add(1);
                    stats.all_in_actions = stats.all_in_actions.saturating_add(1);
                    record_wager(stats, *amount);
                }
            }
        }
    }

    pub(super) fn record_hand_started(&mut self) {
        let Some(game) = self.game.as_ref() else {
            return;
        };
        for (participant, state) in game.players().iter().zip(game.game().players()) {
            if !state.hole_cards().is_empty()
                && let Some(stats) = self
                    .match_profile_stats
                    .get_mut(usize::from(participant.id.0))
            {
                stats.hands_played = stats.hands_played.saturating_add(1);
            }
        }
    }

    pub(super) fn record_completed_hand(&mut self) {
        let Some(game) = self.game.as_ref() else {
            return;
        };
        let TexasPhase::Complete(result) = game.game().phase() else {
            return;
        };
        if !result.showdown {
            return;
        }
        let categories = game
            .players()
            .iter()
            .zip(game.game().players())
            .filter(|(_, state)| !state.folded())
            .filter_map(|(participant, state)| {
                evaluate_player_hand(state.hole_cards(), game.game().community(), game.rules())
                    .ok()
                    .map(|hand| (participant.id, hand.category()))
            })
            .collect::<Vec<_>>();
        for (player, category) in categories {
            if let Some(stats) = self.match_profile_stats.get_mut(usize::from(player.0)) {
                let count = &mut stats.hand_category_counts[hand_category_index(category)];
                *count = count.saturating_add(1);
            }
        }
    }
}
