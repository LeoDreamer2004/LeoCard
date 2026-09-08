use super::*;
use leocard_protocol::{
    GameSnapshot, PlayerId, PlayerViolation, RejectReason, RequestId, ServerEvent, TexasHoldemEvent,
};
use leocard_texas_holdem::{PassiveBot, Phase, TexasHoldemAction, evaluate_player_hand};

impl TexasHoldemSession {
    pub(super) fn snapshot(
        &self,
        connection: ConnectionId,
        request_id: RequestId,
    ) -> Vec<Delivery> {
        let Some(player) = self.room.player_id(connection) else {
            return self.room.reject(
                connection,
                request_id,
                RejectReason::Player(PlayerViolation::NotJoined),
            );
        };
        let event = if self.game.is_some() {
            ServerEvent::GameSnapshot(GameSnapshot::TexasHoldem(self.game_snapshot(player)))
        } else {
            ServerEvent::LobbySnapshot(self.lobby_snapshot())
        };
        vec![self.room.delivery(connection, Some(request_id), event)]
    }

    pub(super) fn current_auto_play_player(&self) -> Option<PlayerId> {
        let current = self
            .game
            .as_ref()
            .and_then(TexasHoldemAdapter::current_player)?;
        self.room
            .players
            .iter()
            .find(|player| player.id == current)
            .is_some_and(|player| {
                !player.left && (player.connected || player.is_bot) && player.auto_play
            })
            .then_some(current)
    }

    pub(super) fn reset_auto_play_delay_for_current_turn(&mut self) {
        self.auto_play_delay = self
            .current_auto_play_player()
            .map(|player| AutoPlayDelayState {
                player,
                remaining: AUTO_PLAY_DELAY,
            });
    }

    pub(super) fn play_automatic_action(&mut self) -> Option<Vec<TexasHoldemEvent>> {
        let (player, action) = {
            let game = self.game.as_ref()?;
            let player = game.current_player()?;
            let core_player = game.game().current_player()?;
            let action = PassiveBot::choose_for_game(game.game(), core_player)?;
            (player, action)
        };
        let was_complete = self
            .game
            .as_ref()
            .is_some_and(|game| matches!(game.game().phase(), Phase::Complete(_)));
        let mut events = self.game.as_mut()?.act(player, action).ok()?;
        events.extend(self.fold_disconnected_players());
        self.record_profile_events(&events);
        self.finish_hand_if_needed(was_complete);
        self.reset_auto_play_delay_for_current_turn();
        Some(events)
    }

    pub(super) fn record_profile_events(&mut self, events: &[TexasHoldemEvent]) {
        for event in events {
            let TexasHoldemEvent::ActionApplied {
                player,
                action,
                amount,
            } = event
            else {
                continue;
            };
            let Some(stats) = self.match_profile_stats.get_mut(usize::from(player.0)) else {
                continue;
            };
            match action {
                TexasHoldemAction::PostBlind => continue,
                TexasHoldemAction::Fold => {
                    stats.voluntary_actions = stats.voluntary_actions.saturating_add(1);
                    stats.hands_folded = stats.hands_folded.saturating_add(1);
                }
                TexasHoldemAction::Check => {
                    stats.voluntary_actions = stats.voluntary_actions.saturating_add(1);
                    stats.check_actions = stats.check_actions.saturating_add(1);
                }
                TexasHoldemAction::Call => {
                    stats.voluntary_actions = stats.voluntary_actions.saturating_add(1);
                    record_wager(stats, *amount);
                }
                TexasHoldemAction::RaiseTo(_) => {
                    stats.voluntary_actions = stats.voluntary_actions.saturating_add(1);
                    stats.raise_actions = stats.raise_actions.saturating_add(1);
                    record_wager(stats, *amount);
                }
                TexasHoldemAction::AllIn => {
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
        let Phase::Complete(result) = game.game().phase() else {
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

    pub(super) fn fold_disconnected_players(&mut self) -> Vec<TexasHoldemEvent> {
        let mut events = Vec::new();
        while let Some(current) = self
            .game
            .as_ref()
            .and_then(TexasHoldemAdapter::current_player)
        {
            let disconnected = self
                .room
                .players
                .iter()
                .find(|player| player.id == current)
                .is_some_and(|player| (!player.connected && !player.is_bot) || player.left);
            if !disconnected {
                break;
            }
            let Some(game) = self.game.as_mut() else {
                break;
            };
            let automatic_action = if game.game().blind_to_post().is_some() {
                TexasHoldemAction::PostBlind
            } else {
                TexasHoldemAction::Fold
            };
            match game.act(current, automatic_action) {
                Ok(auto_events) => events.extend(auto_events),
                Err(_) => break,
            }
        }
        events
    }
}
