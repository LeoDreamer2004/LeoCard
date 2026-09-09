use super::{
    AdapterError, TablePlayer, TexasHoldemAdapter, TexasHoldemSession,
    merge_texas_holdem_profile_stats,
};
use crate::lifecycle::HostedGameLifecycle;
use crate::player::settle_completed_match_profiles_once;
use crate::{AUTO_PLAY_DELAY, AutoPlayDelayState, ConnectionId, Delivery, new_match_id};
use leocard_protocol::TexasHoldemViolation;
use leocard_protocol::{
    GameViolation, PlayerViolation, RejectReason, RequestId, RoomViolation, TABLE_SEAT_COUNT,
    TexasHoldemProfileStats,
};
use leocard_qigui523::reference_point_deltas;
use leocard_texas_holdem::{Phase, TexasHoldemAction, TexasHoldemRuleSet, build_deck};

impl TexasHoldemSession {
    pub(super) fn update_rules(
        &mut self,
        connection: ConnectionId,
        request_id: RequestId,
        rules: TexasHoldemRuleSet,
    ) -> Vec<Delivery> {
        if self.room.player_id(connection).is_none() {
            return self.room.reject(
                connection,
                request_id,
                RejectReason::Player(PlayerViolation::NotJoined),
            );
        }
        if self.game.is_some() {
            return self.room.reject(
                connection,
                request_id,
                RejectReason::Game(GameViolation::GameAlreadyStarted),
            );
        }
        if self.room.host_connection != Some(connection) {
            return self.room.reject(
                connection,
                request_id,
                RejectReason::Room(RoomViolation::OnlyHostCanConfigure),
            );
        }
        let Ok(rules) = (TexasHoldemRuleSet {
            player_count: TABLE_SEAT_COUNT,
            ..rules
        })
        .validate() else {
            return self.room.reject(
                connection,
                request_id,
                RejectReason::Game(GameViolation::InvalidRuleConfiguration),
            );
        };
        if self.rules != rules {
            self.rules = rules;
            self.room.reset_ready_after_rules_change();
            let mut deck = build_deck(rules.short_deck);
            fastrand::shuffle(&mut deck);
            self.shuffled_deck = Some(deck);
            self.room.bump_revision();
        }
        self.broadcast_lobby(Some((connection, request_id)))
    }

    pub(super) fn set_auto_play(
        &mut self,
        connection: ConnectionId,
        request_id: RequestId,
        enabled: bool,
    ) -> Vec<Delivery> {
        let Some(player) = self.room.player_id(connection) else {
            return self.room.reject(
                connection,
                request_id,
                RejectReason::Player(PlayerViolation::NotJoined),
            );
        };
        let Some(game) = self.game.as_ref() else {
            return self.room.reject(
                connection,
                request_id,
                RejectReason::Game(GameViolation::GameNotStarted),
            );
        };
        if !matches!(game.game().phase(), Phase::Betting(_)) {
            return self.room.reject(
                connection,
                request_id,
                RejectReason::Game(GameViolation::TexasHoldem(
                    TexasHoldemViolation::HandAlreadyComplete,
                )),
            );
        }

        let participant = self
            .room
            .players
            .iter_mut()
            .find(|participant| participant.id == player)
            .expect("joined player belongs to the room");
        let changed = participant.auto_play != enabled;
        participant.auto_play = enabled;
        if self
            .game
            .as_ref()
            .and_then(TexasHoldemAdapter::current_player)
            == Some(player)
        {
            self.auto_play_delay = enabled.then_some(AutoPlayDelayState {
                player,
                remaining: AUTO_PLAY_DELAY,
            });
        } else if self
            .auto_play_delay
            .as_ref()
            .is_some_and(|delay| delay.player == player)
        {
            self.auto_play_delay = None;
        }
        if changed {
            self.room.bump_revision();
        }
        self.broadcast_game(Some((connection, request_id)))
    }

    pub(super) fn create_tournament(&mut self) -> Result<(), AdapterError> {
        let deck = self.shuffled_deck.take().unwrap_or_else(|| {
            let mut deck = build_deck(self.rules.short_deck);
            fastrand::shuffle(&mut deck);
            deck
        });
        let host = self
            .room
            .host_player_id()
            .expect("a started room has a host");
        let table_players = self
            .room
            .players
            .iter_mut()
            .filter(|player| !player.left)
            .map(|player| {
                player.ready = false;
                player.auto_play = player.is_bot;
                TablePlayer {
                    id: player.id,
                    profile_id: player.profile_id,
                    name: player.name.clone(),
                    avatar: player.avatar,
                    seat: player.seat.expect("start validation required every seat"),
                    connected: player.connected || player.is_bot,
                    reference_points: player.reference_points,
                    completed_games: player.completed_games,
                }
            })
            .collect::<Vec<_>>();
        let first_dealer = table_players[fastrand::usize(..table_players.len())].id;
        self.game = Some(TexasHoldemAdapter::new(
            new_match_id(),
            self.room.host_port,
            host,
            table_players,
            self.rules,
            first_dealer,
            deck,
        )?);
        self.match_profile_stats =
            vec![TexasHoldemProfileStats::default(); self.room.players.len()];
        self.record_hand_started();
        self.finished_reference_changes = None;
        self.auto_play_delay = None;
        self.reset_auto_play_delay_for_current_turn();
        Ok(())
    }

    pub(super) fn act(
        &mut self,
        connection: ConnectionId,
        request_id: RequestId,
        action: TexasHoldemAction,
    ) -> Vec<Delivery> {
        let Some(player) = self.room.player_id(connection) else {
            return self.room.reject(
                connection,
                request_id,
                RejectReason::Player(PlayerViolation::NotJoined),
            );
        };
        let was_complete = self
            .game
            .as_ref()
            .is_some_and(|game| matches!(game.game().phase(), Phase::Complete(_)));
        let Some(game) = self.game.as_mut() else {
            return self.room.reject(
                connection,
                request_id,
                RejectReason::Game(GameViolation::GameNotStarted),
            );
        };
        let mut events = match game.act(player, action) {
            Ok(events) => events,
            Err(AdapterError::Violation(violation)) => {
                return self.room.reject(
                    connection,
                    request_id,
                    RejectReason::Game(GameViolation::TexasHoldem(violation)),
                );
            }
            Err(_) => {
                return self.room.reject(
                    connection,
                    request_id,
                    RejectReason::Game(GameViolation::InvalidRuleConfiguration),
                );
            }
        };
        events.extend(self.fold_disconnected_players());
        self.record_profile_events(&events);
        self.finish_hand_if_needed(was_complete);
        self.reset_auto_play_delay_for_current_turn();
        self.room.bump_revision();
        let mut deliveries = self.broadcast_events(events);
        deliveries.extend(self.broadcast_game(Some((connection, request_id))));
        deliveries
    }

    pub(super) fn tournament_complete(&self) -> bool {
        self.game.as_ref().is_some_and(|game| {
            matches!(game.game().phase(), Phase::Complete(_))
                && game
                    .game()
                    .players()
                    .iter()
                    .any(|player| player.stack() == 0)
        })
    }

    pub(super) fn finish_hand_if_needed(&mut self, was_complete: bool) {
        if was_complete
            || !self
                .game
                .as_ref()
                .is_some_and(|game| matches!(game.game().phase(), Phase::Complete(_)))
        {
            return;
        }
        self.record_completed_hand();
        self.room.prepare_rematch();
        self.auto_play_delay = None;
        if self.tournament_complete() {
            self.apply_finished_reference_points();
        }
    }

    pub(super) fn apply_finished_reference_points(&mut self) {
        let Some(game) = self.game.as_ref() else {
            return;
        };
        let standings = game
            .players()
            .iter()
            .zip(game.game().players())
            .map(|(participant, state)| (participant.id, state.stack()))
            .collect::<Vec<_>>();
        let scores = standings
            .iter()
            .map(|(_, stack)| *stack)
            .collect::<Vec<_>>();
        let deltas = reference_point_deltas(&scores)
            .expect("a Texas Hold'em table always contains between three and six players");
        let settlements = standings
            .iter()
            .zip(deltas)
            .map(|((player, _), delta)| (*player, delta))
            .collect::<Vec<_>>();
        let match_profile_stats = self.match_profile_stats.clone();
        settle_completed_match_profiles_once(
            &mut self.finished_reference_changes,
            &mut self.room,
            settlements,
            |participant, delta| {
                let player_id = participant.id;
                let final_chips = standings
                    .iter()
                    .find_map(|(player, chips)| (*player == player_id).then_some(*chips))
                    .expect("a settlement player has final chips");
                let current_profile_stats = match_profile_stats.get(usize::from(player_id.0));
                let placement = 1 + scores.iter().filter(|other| **other > final_chips).count();
                let aggregate = participant
                    .game_profiles
                    .texas_holdem
                    .get_or_insert_with(TexasHoldemProfileStats::default);
                aggregate.completed_games = aggregate.completed_games.saturating_add(1);
                aggregate.total_reference_delta = aggregate
                    .total_reference_delta
                    .saturating_add(i64::from(delta));
                aggregate.total_final_chips = aggregate
                    .total_final_chips
                    .saturating_add(u64::from(final_chips));
                if let Some(count) = aggregate.placement_counts.get_mut(placement - 1) {
                    *count = count.saturating_add(1);
                }
                if let Some(current) = current_profile_stats {
                    merge_texas_holdem_profile_stats(aggregate, current);
                }
            },
        );
    }
}
