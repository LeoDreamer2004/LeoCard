use super::*;
use leocard_protocol::TexasHoldemViolation;

impl TexasHoldemSession {
    pub(super) fn update_rules(
        &mut self,
        connection: ConnectionId,
        request_id: RequestId,
        rules: TexasHoldemRuleSet,
    ) -> Vec<Delivery> {
        if self.room.player_id(connection).is_none() {
            return self
                .room
                .reject(connection, request_id, RejectReason::NotJoined);
        }
        if self.game.is_some() {
            return self
                .room
                .reject(connection, request_id, RejectReason::GameAlreadyStarted);
        }
        if self.room.host_connection != Some(connection) {
            return self
                .room
                .reject(connection, request_id, RejectReason::OnlyHostCanConfigure);
        }
        let Ok(rules) = (TexasHoldemRuleSet {
            player_count: TABLE_SEAT_COUNT,
            ..rules
        })
        .validate() else {
            return self.room.reject(
                connection,
                request_id,
                RejectReason::InvalidRuleConfiguration,
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
            return self
                .room
                .reject(connection, request_id, RejectReason::NotJoined);
        };
        let Some(game) = self.game.as_ref() else {
            return self
                .room
                .reject(connection, request_id, RejectReason::GameNotStarted);
        };
        if !matches!(game.game().phase(), Phase::Betting(_)) {
            return self.room.reject(
                connection,
                request_id,
                RejectReason::GameViolation(GameViolation::TexasHoldem(
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

    pub(super) fn start_game(
        &mut self,
        connection: ConnectionId,
        request_id: RequestId,
    ) -> Vec<Delivery> {
        if self.room.player_id(connection).is_none() {
            return self
                .room
                .reject(connection, request_id, RejectReason::NotJoined);
        }
        if self.game.is_some() {
            return self
                .room
                .reject(connection, request_id, RejectReason::GameAlreadyStarted);
        }
        if self.room.host_connection != Some(connection) {
            return self
                .room
                .reject(connection, request_id, RejectReason::OnlyHostCanStart);
        }
        let active_player_count = self
            .room
            .players
            .iter()
            .filter(|player| !player.left)
            .count();
        if active_player_count < usize::from(TexasHoldemRuleSet::MIN_PLAYERS) {
            return self.room.reject(
                connection,
                request_id,
                RejectReason::NotEnoughPlayers {
                    minimum: TexasHoldemRuleSet::MIN_PLAYERS,
                    actual: active_player_count as u8,
                },
            );
        }
        if self
            .room
            .players
            .iter()
            .any(|player| !player.left && player.seat.is_none())
        {
            return self
                .room
                .reject(connection, request_id, RejectReason::MustSelectSeat);
        }
        let not_ready = self
            .room
            .players
            .iter()
            .filter(|player| !player.left)
            .filter(|player| !player.ready)
            .map(|player| player.id)
            .collect::<Vec<_>>();
        if !not_ready.is_empty() {
            return self.room.reject(
                connection,
                request_id,
                RejectReason::PlayersNotReady { players: not_ready },
            );
        }
        // Keep lobby PlayerIds stable. Compaction is safe only once every recipient is
        // guaranteed to receive a private game snapshot carrying its reassigned `you`.
        self.room.remove_departed_players();
        match self.create_tournament() {
            Ok(()) => {
                self.room.bump_revision();
                self.broadcast_game(Some((connection, request_id)))
            }
            Err(_) => {
                #[cfg(feature = "developer")]
                self.room.remove_developer_bots();
                self.room.reject(
                    connection,
                    request_id,
                    RejectReason::InvalidRuleConfiguration,
                )
            }
        }
    }

    fn create_tournament(&mut self) -> Result<(), AdapterError> {
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
            return self
                .room
                .reject(connection, request_id, RejectReason::NotJoined);
        };
        let was_complete = self
            .game
            .as_ref()
            .is_some_and(|game| matches!(game.game().phase(), Phase::Complete(_)));
        let Some(game) = self.game.as_mut() else {
            return self
                .room
                .reject(connection, request_id, RejectReason::GameNotStarted);
        };
        let mut events = match game.act(player, action) {
            Ok(events) => events,
            Err(AdapterError::Violation(violation)) => {
                return self.room.reject(
                    connection,
                    request_id,
                    RejectReason::GameViolation(GameViolation::TexasHoldem(violation)),
                );
            }
            Err(_) => {
                return self.room.reject(
                    connection,
                    request_id,
                    RejectReason::InvalidRuleConfiguration,
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

    pub(super) fn return_to_lobby(
        &mut self,
        connection: ConnectionId,
        request_id: RequestId,
    ) -> Vec<Delivery> {
        if self.room.player_id(connection).is_none() {
            return self
                .room
                .reject(connection, request_id, RejectReason::NotJoined);
        }
        let Some(game) = self.game.as_ref() else {
            return self
                .room
                .reject(connection, request_id, RejectReason::GameNotStarted);
        };
        if !matches!(game.game().phase(), Phase::Complete(_)) || !self.tournament_complete() {
            return self
                .room
                .reject(connection, request_id, RejectReason::GameNotFinished);
        }
        self.game = None;
        self.match_profile_stats.clear();
        self.auto_play_delay = None;
        #[cfg(feature = "developer")]
        self.room.remove_developer_bots();
        let host = self.room.host_connection;
        for player in &mut self.room.players {
            player.ready = host == Some(player.connection);
            player.auto_play = player.is_bot;
            if !player.connected {
                player.seat = None;
            }
        }
        let mut deck = build_deck(self.rules.short_deck);
        fastrand::shuffle(&mut deck);
        self.shuffled_deck = Some(deck);
        self.room.bump_revision();
        self.broadcast_lobby(Some((connection, request_id)))
    }

    pub(super) fn play_again(
        &mut self,
        connection: ConnectionId,
        request_id: RequestId,
    ) -> Vec<Delivery> {
        let Some(player) = self.room.player_id(connection) else {
            return self
                .room
                .reject(connection, request_id, RejectReason::NotJoined);
        };
        let Some(game) = self.game.as_ref() else {
            return self
                .room
                .reject(connection, request_id, RejectReason::GameNotStarted);
        };
        if !matches!(game.game().phase(), Phase::Complete(_)) || self.tournament_complete() {
            return self
                .room
                .reject(connection, request_id, RejectReason::GameNotFinished);
        }
        if let Some(participant) = self.room.players.iter_mut().find(|p| p.id == player) {
            participant.ready = true;
        }
        self.room.bump_revision();
        let active = self
            .room
            .players
            .iter()
            .filter(|player| (player.connected || player.is_bot) && !player.left);
        let everyone_ready = active.clone().count() >= usize::from(TexasHoldemRuleSet::MIN_PLAYERS)
            && active.clone().all(|player| player.ready);
        if everyone_ready {
            for participant in &mut self.room.players {
                participant.ready = false;
            }
            let mut deck = build_deck(self.rules.short_deck);
            fastrand::shuffle(&mut deck);
            if self
                .game
                .as_mut()
                .is_none_or(|game| game.start_next_hand(deck).is_err())
            {
                return self.room.reject(
                    connection,
                    request_id,
                    RejectReason::InvalidRuleConfiguration,
                );
            }
            let events = self.fold_disconnected_players();
            self.record_hand_started();
            self.record_profile_events(&events);
            self.finish_hand_if_needed(false);
            self.reset_auto_play_delay_for_current_turn();
            self.room.bump_revision();
            let mut deliveries = self.broadcast_events(events);
            deliveries.extend(self.broadcast_game(Some((connection, request_id))));
            return deliveries;
        }
        self.broadcast_game(Some((connection, request_id)))
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
        if self.finished_reference_changes.is_some() {
            return;
        }
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
        let mut changes = Vec::with_capacity(standings.len());
        let match_profile_stats = self.match_profile_stats.clone();
        for ((player_id, final_chips), delta) in standings.into_iter().zip(deltas) {
            let current_profile_stats = match_profile_stats.get(usize::from(player_id.0));
            let participant = self
                .room
                .players
                .iter_mut()
                .find(|participant| participant.id == player_id)
                .expect("an adapter participant belongs to the room");
            participant.reference_points = participant
                .reference_points
                .saturating_add(i32::from(delta));
            participant.completed_games = participant.completed_games.saturating_add(1);
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
            changes.push(PlayerReferenceChange {
                player: player_id,
                profile_id: participant.profile_id,
                delta,
            });
        }
        self.finished_reference_changes = Some(changes);
    }

    pub(super) fn leave_room(
        &mut self,
        connection: ConnectionId,
        request_id: RequestId,
    ) -> Vec<Delivery> {
        let Some(index) = self
            .room
            .players
            .iter()
            .position(|player| player.connection == connection && !player.left)
        else {
            return self
                .room
                .reject(connection, request_id, RejectReason::NotJoined);
        };
        if self.room.host_connection == Some(connection) {
            return self.close_room(connection, request_id);
        }
        let id = self.room.players[index].id;
        let name = self.room.players[index].name.clone();
        self.room.players[index].ready = false;
        self.room.players[index].connected = false;
        self.room.players[index].left = true;
        if let Some(game) = self.game.as_mut() {
            let _ = game.set_connected(id, false);
        } else {
            self.room.players[index].seat = None;
        }
        let was_complete = self
            .game
            .as_ref()
            .is_some_and(|game| matches!(game.game().phase(), Phase::Complete(_)));
        let events = self.fold_disconnected_players();
        self.record_profile_events(&events);
        self.finish_hand_if_needed(was_complete);
        self.reset_auto_play_delay_for_current_turn();
        self.room.bump_revision();
        let mut deliveries =
            vec![
                self.room
                    .delivery(connection, Some(request_id), ServerEvent::LeftRoom),
            ];
        deliveries.extend(
            self.room
                .players
                .iter()
                .filter(|player| player.connected && !player.left)
                .map(|player| {
                    self.room.delivery(
                        player.connection,
                        None,
                        ServerEvent::PlayerLeft { name: name.clone() },
                    )
                }),
        );
        deliveries.extend(self.broadcast_events(events));
        deliveries.extend(if self.game.is_some() {
            self.broadcast_game(None)
        } else {
            self.broadcast_lobby(None)
        });
        deliveries
    }

    pub(super) fn close_room(
        &mut self,
        connection: ConnectionId,
        request_id: RequestId,
    ) -> Vec<Delivery> {
        self.room
            .close_room(connection, request_id)
            .unwrap_or_else(|reason| self.room.reject(connection, request_id, reason))
    }

    pub(super) fn interact(
        &mut self,
        connection: ConnectionId,
        request_id: RequestId,
        target: PlayerId,
        kind: PlayerInteractionKind,
    ) -> Vec<Delivery> {
        let Some(source) = self.room.player_id(connection) else {
            return self
                .room
                .reject(connection, request_id, RejectReason::NotJoined);
        };
        let Some(game) = self.game.as_ref() else {
            return self
                .room
                .reject(connection, request_id, RejectReason::GameNotStarted);
        };
        if !matches!(game.game().phase(), Phase::Betting(_)) {
            return self.room.reject(
                connection,
                request_id,
                RejectReason::GameViolation(GameViolation::TexasHoldem(
                    TexasHoldemViolation::HandAlreadyComplete,
                )),
            );
        }
        if source == target
            || !self
                .room
                .players
                .iter()
                .any(|player| player.id == target && !player.left)
        {
            return self.room.reject(
                connection,
                request_id,
                RejectReason::GameViolation(GameViolation::TexasHoldem(
                    TexasHoldemViolation::InvalidPlayer,
                )),
            );
        }
        let interaction = PlayerInteraction {
            source,
            target,
            kind,
            seed: fastrand::u32(..),
        };
        self.room.record_received_interaction(target, kind);
        self.room.bump_revision();
        let mut deliveries = self
            .room
            .players
            .iter()
            .filter(|player| player.connected && !player.left)
            .map(|player| {
                self.room.delivery(
                    player.connection,
                    (player.connection == connection).then_some(request_id),
                    ServerEvent::PlayerInteraction(interaction),
                )
            })
            .collect::<Vec<_>>();
        deliveries.extend(self.broadcast_game(None));
        deliveries
    }
}
