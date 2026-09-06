use super::*;

impl QiGui523Session {
    pub(super) fn chat(
        &self,
        connection: ConnectionId,
        request_id: RequestId,
        content: ChatContent,
    ) -> Vec<Delivery> {
        self.room
            .chat(connection, request_id, content, self.game.is_some())
            .unwrap_or_else(|reason| self.reject(connection, request_id, reason))
    }

    #[cfg(feature = "developer")]
    pub(super) fn set_developer_hand(
        &mut self,
        connection: ConnectionId,
        request_id: RequestId,
        cards: Vec<QiGuiCard>,
    ) -> Vec<Delivery> {
        let Some(player) = self.player_id(connection) else {
            return self.reject(connection, request_id, RejectReason::NotJoined);
        };
        let Some(game) = self.game.as_mut() else {
            return self.reject(connection, request_id, RejectReason::GameNotStarted);
        };
        if cards.is_empty()
            || game
                .replace_player_hand(to_core_player(player), cards)
                .is_err()
        {
            return self.reject(connection, request_id, RejectReason::InvalidDeveloperHand);
        }
        self.bump_revision();
        self.broadcast_game(Some((connection, request_id)))
    }

    #[cfg(not(feature = "developer"))]
    pub(super) fn set_developer_hand(
        &self,
        connection: ConnectionId,
        request_id: RequestId,
        _cards: Vec<QiGuiCard>,
    ) -> Vec<Delivery> {
        self.reject(
            connection,
            request_id,
            RejectReason::DeveloperFeatureUnavailable,
        )
    }

    pub(super) fn interact(
        &mut self,
        connection: ConnectionId,
        request_id: RequestId,
        target: PlayerId,
        kind: PlayerInteractionKind,
    ) -> Vec<Delivery> {
        let Some(source) = self.player_id(connection) else {
            return self.reject(connection, request_id, RejectReason::NotJoined);
        };
        let Some(game) = self.game.as_ref() else {
            return self.reject(connection, request_id, RejectReason::GameNotStarted);
        };
        if !matches!(game.phase(), Phase::Playing) {
            return self.reject(
                connection,
                request_id,
                RejectReason::GameViolation(GameViolation::QiGui523(
                    RuleViolation::GameAlreadyFinished,
                )),
            );
        }
        if source == target
            || !self
                .players
                .iter()
                .any(|player| player.id == target && !player.left)
        {
            return self.reject(
                connection,
                request_id,
                RejectReason::GameViolation(GameViolation::QiGui523(RuleViolation::InvalidPlayer)),
            );
        }

        let interaction = PlayerInteraction {
            source,
            target,
            kind,
            seed: fastrand::u32(..),
        };
        self.record_received_interaction(target, kind);
        self.bump_revision();
        let mut deliveries = self
            .players
            .iter()
            .filter(|player| player.connected && !player.left)
            .map(|player| {
                self.delivery(
                    player.connection,
                    (player.connection == connection).then_some(request_id),
                    ServerEvent::PlayerInteraction(interaction),
                )
            })
            .collect::<Vec<_>>();
        deliveries.extend(self.broadcast_game(None));
        deliveries
    }

    pub(super) fn join(
        &mut self,
        connection: ConnectionId,
        request_id: RequestId,
        request: JoinRequest,
    ) -> Vec<Delivery> {
        if self.player_id(connection).is_some() {
            return self.reject(connection, request_id, RejectReason::AlreadyJoined);
        }
        let normalized_name = request.name.trim();
        if normalized_name.is_empty() {
            return self.reject(connection, request_id, RejectReason::NameEmpty);
        }
        if normalized_name.chars().count() > MAX_PLAYER_NAME_CHARS {
            return self.reject(
                connection,
                request_id,
                RejectReason::NameTooLong {
                    max_chars: MAX_PLAYER_NAME_CHARS as u16,
                },
            );
        }
        if !valid_identity_proof(self.room_id, &request) {
            return self.reject(connection, request_id, RejectReason::InvalidIdentityProof);
        }
        let JoinRequest {
            name,
            reconnect_token,
            profile_id,
            reference_points,
            completed_games,
            game_profiles,
            identity_signature: _,
        } = request;
        let name = name.trim();
        if let Some(index) = self
            .players
            .iter()
            .position(|player| player.reconnect_token == reconnect_token && !player.left)
        {
            if self.players[index].name != name || self.players[index].profile_id != profile_id {
                return self.reject(connection, request_id, RejectReason::AlreadyJoined);
            }
            return self.reconnect(index, connection, request_id);
        }
        if self
            .players
            .iter()
            .any(|player| player.profile_id == profile_id && !player.left)
        {
            return self.reject(connection, request_id, RejectReason::AlreadyJoined);
        }
        if self.game.is_some() {
            return self.reject(connection, request_id, RejectReason::GameAlreadyStarted);
        }
        if self.players.iter().filter(|player| !player.left).count()
            >= usize::from(self.rules.player_count)
        {
            return self.reject(connection, request_id, RejectReason::RoomFull);
        }

        let vacant = self.players.iter().position(|player| player.left);
        let player = PlayerId(
            vacant
                .unwrap_or(self.players.len())
                .try_into()
                .expect("a room contains at most six active player slots"),
        );
        let joining_as_host = self.host_connection.is_none();
        if joining_as_host {
            self.host_connection = Some(connection);
        }
        let seat = self
            .random_available_seat()
            .expect("a non-full room always has an available seat");
        let participant = Participant {
            id: player,
            profile_id,
            connection,
            name: name.to_owned(),
            avatar: None,
            avatar_png: None,
            reconnect_token,
            seat: Some(seat),
            ready: joining_as_host,
            connected: true,
            auto_play: false,
            is_bot: false,
            left: false,
            reference_points,
            completed_games,
            game_profiles,
        };
        if let Some(index) = vacant {
            self.players[index] = participant;
        } else {
            self.players.push(participant);
        }
        self.bump_revision();

        let mut deliveries = vec![self.delivery(
            connection,
            Some(request_id),
            ServerEvent::Joined { you: player },
        )];
        deliveries.extend(
            self.players
                .iter()
                .filter(|player| !player.left)
                .filter_map(|participant| {
                    Some(self.delivery(
                        connection,
                        None,
                        ServerEvent::AvatarData {
                            id: participant.avatar?,
                            png: participant.avatar_png.clone()?,
                        },
                    ))
                }),
        );
        deliveries.extend(self.broadcast_lobby(Some((connection, request_id))));
        deliveries
    }

    fn reconnect(
        &mut self,
        index: usize,
        connection: ConnectionId,
        request_id: RequestId,
    ) -> Vec<Delivery> {
        let previous_connection = self.players[index].connection;
        let player = self.players[index].id;
        let reassigned_seat = (self.game.is_none() && self.players[index].seat.is_none())
            .then(|| self.random_available_seat())
            .flatten();
        self.players[index].connection = connection;
        self.players[index].connected = true;
        if let Some(seat) = reassigned_seat {
            self.players[index].seat = Some(seat);
        }
        if self.host_connection == Some(previous_connection) {
            self.host_connection = Some(connection);
        }
        let previous_last_request = self.last_requests.remove(&previous_connection);
        if let Some(last_request) = self.last_requests.get_mut(&connection) {
            if let Some(previous_last_request) = previous_last_request {
                *last_request = (*last_request).max(previous_last_request);
            }
        } else if let Some(previous_last_request) = previous_last_request {
            self.last_requests.insert(connection, previous_last_request);
        }
        self.bump_revision();

        let mut deliveries = vec![self.delivery(
            connection,
            Some(request_id),
            ServerEvent::Joined { you: player },
        )];
        deliveries.extend(
            self.players
                .iter()
                .filter(|player| !player.left)
                .filter_map(|participant| {
                    Some(self.delivery(
                        connection,
                        None,
                        ServerEvent::AvatarData {
                            id: participant.avatar?,
                            png: participant.avatar_png.clone()?,
                        },
                    ))
                }),
        );
        deliveries.extend(if self.game.is_some() {
            self.broadcast_game(Some((connection, request_id)))
        } else {
            self.broadcast_lobby(Some((connection, request_id)))
        });
        deliveries
    }

    pub(super) fn set_avatar(
        &mut self,
        connection: ConnectionId,
        request_id: RequestId,
        png: Vec<u8>,
    ) -> Vec<Delivery> {
        match self.room.set_avatar(connection, png, self.game.is_some()) {
            Ok(mut deliveries) => {
                deliveries.extend(self.broadcast_lobby(Some((connection, request_id))));
                deliveries
            }
            Err(reason) => self.reject(connection, request_id, reason),
        }
    }

    pub(super) fn select_seat(
        &mut self,
        connection: ConnectionId,
        request_id: RequestId,
        seat: SeatId,
    ) -> Vec<Delivery> {
        match self.room.select_seat(connection, seat, self.game.is_some()) {
            Ok(()) => self.broadcast_lobby(Some((connection, request_id))),
            Err(reason) => self.reject(connection, request_id, reason),
        }
    }

    pub(super) fn set_ready(
        &mut self,
        connection: ConnectionId,
        request_id: RequestId,
        ready: bool,
    ) -> Vec<Delivery> {
        match self.room.set_ready(connection, ready, self.game.is_some()) {
            Ok(()) => self.broadcast_lobby(Some((connection, request_id))),
            Err(reason) => self.reject(connection, request_id, reason),
        }
    }

    pub(super) fn set_auto_play(
        &mut self,
        connection: ConnectionId,
        request_id: RequestId,
        enabled: bool,
    ) -> Vec<Delivery> {
        let Some(player) = self.player_id(connection) else {
            return self.reject(connection, request_id, RejectReason::NotJoined);
        };
        let Some(game) = self.game.as_ref() else {
            return self.reject(connection, request_id, RejectReason::GameNotStarted);
        };
        if !matches!(game.phase(), Phase::Playing) {
            return self.reject(
                connection,
                request_id,
                RejectReason::GameViolation(GameViolation::QiGui523(
                    RuleViolation::GameAlreadyFinished,
                )),
            );
        }

        let participant = &mut self.players[usize::from(player.0)];
        let changed = participant.auto_play != enabled;
        participant.auto_play = enabled;
        if self.current_player() == Some(player) {
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
            self.bump_revision();
        }
        self.broadcast_game(Some((connection, request_id)))
    }

    pub(super) fn update_rules(
        &mut self,
        connection: ConnectionId,
        request_id: RequestId,
        rules: QiGuiRuleSet,
    ) -> Vec<Delivery> {
        if self.player_id(connection).is_none() {
            return self.reject(connection, request_id, RejectReason::NotJoined);
        }
        if self.game.is_some() {
            return self.reject(connection, request_id, RejectReason::GameAlreadyStarted);
        }
        if self.host_connection != Some(connection) {
            return self.reject(connection, request_id, RejectReason::OnlyHostCanConfigure);
        }
        let rules = QiGuiRuleSet {
            player_count: TABLE_SEAT_COUNT,
            ..rules
        };
        let Ok(rules) = rules.validate() else {
            return self.reject(
                connection,
                request_id,
                RejectReason::InvalidRuleConfiguration,
            );
        };

        if self.rules != rules {
            self.rules = rules;
            self.room.reset_ready_after_rules_change();
            let mut deck = build_deck(rules.deck_count);
            fastrand::shuffle(&mut deck);
            self.shuffled_deck = Some(deck);
            self.bump_revision();
        }
        self.broadcast_lobby(Some((connection, request_id)))
    }

    pub(super) fn start_game(
        &mut self,
        connection: ConnectionId,
        request_id: RequestId,
    ) -> Vec<Delivery> {
        if self.player_id(connection).is_none() {
            return self.reject(connection, request_id, RejectReason::NotJoined);
        }
        if self.game.is_some() {
            return self.reject(connection, request_id, RejectReason::GameAlreadyStarted);
        }
        if self.host_connection != Some(connection) {
            return self.reject(connection, request_id, RejectReason::OnlyHostCanStart);
        }
        let active_player_count = self.players.iter().filter(|player| !player.left).count();
        if active_player_count < 2 {
            return self.reject(
                connection,
                request_id,
                RejectReason::NotEnoughPlayers {
                    minimum: 2,
                    actual: active_player_count as u8,
                },
            );
        }
        let not_seated: Vec<_> = self
            .players
            .iter()
            .filter(|player| !player.left)
            .filter(|player| player.seat.is_none())
            .map(|player| player.id)
            .collect();
        if !not_seated.is_empty() {
            return self.reject(connection, request_id, RejectReason::MustSelectSeat);
        }
        let not_ready: Vec<_> = self
            .players
            .iter()
            .filter(|player| !player.left)
            .filter(|player| !player.ready)
            .map(|player| player.id)
            .collect();
        if !not_ready.is_empty() {
            return self.reject(
                connection,
                request_id,
                RejectReason::PlayersNotReady { players: not_ready },
            );
        }

        // PlayerId 在大厅内必须保持稳定；只在确定能够开局、且即将给每位玩家
        // 下发包含新 `you` 的私有牌局快照时，才压缩并按座位重排参与者。
        self.remove_departed_players();

        let deck = self
            .shuffled_deck
            .take()
            .expect("a validated deck exists until the game starts");
        self.players.sort_by_key(|player| {
            player
                .seat
                .expect("all players selected a seat before starting")
                .0
        });
        for (index, player) in self.players.iter_mut().enumerate() {
            player.id = PlayerId(index as u8);
            player.ready = false;
            player.auto_play = player.is_bot;
        }
        let game_rules = QiGuiRuleSet {
            player_count: self.players.len() as u8,
            ..self.rules
        };
        #[cfg(not(feature = "developer"))]
        let game = GameState::new_with_deck(game_rules, deck)
            .expect("host validated rules and deck during construction");
        #[cfg(feature = "developer")]
        let game = {
            let mut deck = deck;
            if game_rules.developer_deck {
                deck.truncate(
                    usize::from(game_rules.player_count) * usize::from(game_rules.hand_size),
                );
                GameState::new_with_development_deck(game_rules, deck)
                    .expect("the development deck is a valid shuffled subset")
            } else {
                GameState::new_with_deck(game_rules, deck)
                    .expect("host validated rules and deck during construction")
            }
        };
        self.game = Some(game);
        self.match_id = Some(new_match_id());
        self.finished_reference_changes = None;
        self.match_profile_stats = vec![QiGui523ProfileStats::default(); self.players.len()];
        self.auto_play_delay = None;
        self.reset_auto_play_delay_for_current_turn();
        self.initialize_turn_timer();
        self.bump_revision();
        self.broadcast_game(Some((connection, request_id)))
    }

    pub(super) fn return_to_lobby(
        &mut self,
        connection: ConnectionId,
        request_id: RequestId,
    ) -> Vec<Delivery> {
        if self.player_id(connection).is_none() {
            return self.reject(connection, request_id, RejectReason::NotJoined);
        }
        if self.host_connection != Some(connection) {
            return self.reject(
                connection,
                request_id,
                RejectReason::OnlyHostCanReturnToLobby,
            );
        }
        let Some(game) = self.game.as_ref() else {
            return self.reject(connection, request_id, RejectReason::GameNotStarted);
        };
        if !matches!(game.phase(), Phase::Finished(_)) {
            return self.reject(connection, request_id, RejectReason::GameNotFinished);
        }

        self.game = None;
        self.match_id = None;
        self.finished_reference_changes = None;
        self.match_profile_stats.clear();
        self.turn_timer = None;
        self.auto_play_delay = None;
        #[cfg(feature = "developer")]
        self.room.remove_developer_bots();
        let host_connection = self.host_connection;
        for player in &mut self.players {
            player.ready = host_connection == Some(player.connection);
            player.auto_play = false;
            if !player.connected {
                player.seat = None;
            }
        }
        let mut deck = build_deck(self.rules.deck_count);
        fastrand::shuffle(&mut deck);
        self.shuffled_deck = Some(deck);
        self.bump_revision();
        self.broadcast_lobby(Some((connection, request_id)))
    }

    pub(super) fn play_again(
        &mut self,
        connection: ConnectionId,
        request_id: RequestId,
    ) -> Vec<Delivery> {
        let Some(player) = self.player_id(connection) else {
            return self.reject(connection, request_id, RejectReason::NotJoined);
        };
        let Some(game) = self.game.as_ref() else {
            return self.reject(connection, request_id, RejectReason::GameNotStarted);
        };
        if !matches!(game.phase(), Phase::Finished(_)) {
            return self.reject(connection, request_id, RejectReason::GameNotFinished);
        }

        let participant = &mut self.players[usize::from(player.0)];
        if !participant.ready {
            participant.ready = true;
            self.bump_revision();
        }

        let active_players = self.players.iter().filter(|player| !player.left);
        let active_count = active_players.clone().count();
        if active_count >= 2 && active_players.clone().all(|player| player.ready) {
            return self.start_next_game(Some((connection, request_id)));
        }
        self.broadcast_game(Some((connection, request_id)))
    }

    pub(super) fn leave_room(
        &mut self,
        connection: ConnectionId,
        request_id: RequestId,
    ) -> Vec<Delivery> {
        let Some(index) = self
            .players
            .iter()
            .position(|player| player.connection == connection && !player.left)
        else {
            return self.reject(connection, request_id, RejectReason::NotJoined);
        };
        if self.host_connection == Some(connection) {
            return self.close_room(connection, request_id);
        }

        let name = self.players[index].name.clone();
        self.players[index].ready = false;
        self.players[index].connected = false;
        self.players[index].left = true;
        let automatic_effect = if self.current_player_is_disconnected() {
            let effect = self.play_automatic_action();
            self.reset_timer_for_current_turn();
            effect
        } else {
            None
        };
        self.bump_revision();

        let mut deliveries = automatic_effect
            .map(|effect| self.broadcast_play_effect(effect))
            .unwrap_or_default();
        deliveries.push(self.delivery(connection, Some(request_id), ServerEvent::LeftRoom));
        deliveries.extend(
            self.players
                .iter()
                .filter(|player| player.connected && !player.left)
                .map(|player| {
                    self.delivery(
                        player.connection,
                        None,
                        ServerEvent::PlayerLeft { name: name.clone() },
                    )
                }),
        );
        let should_start_next = self
            .game
            .as_ref()
            .is_some_and(|game| matches!(game.phase(), Phase::Finished(_)))
            && self.players.iter().filter(|player| !player.left).count() >= 2
            && self
                .players
                .iter()
                .filter(|player| !player.left)
                .all(|player| player.ready);
        if should_start_next {
            deliveries.extend(self.start_next_game(None));
        } else if self.game.is_some() {
            deliveries.extend(self.broadcast_game(None));
        } else {
            deliveries.extend(self.broadcast_lobby(None));
        }
        deliveries
    }

    fn start_next_game(&mut self, origin: Option<(ConnectionId, RequestId)>) -> Vec<Delivery> {
        self.game = None;
        self.turn_timer = None;
        self.auto_play_delay = None;
        self.remove_departed_players();
        for player in &mut self.players {
            player.ready = false;
        }

        let mut deck = build_deck(self.rules.deck_count);
        fastrand::shuffle(&mut deck);
        let game_rules = QiGuiRuleSet {
            player_count: self.players.len() as u8,
            ..self.rules
        };
        #[cfg(not(feature = "developer"))]
        let game = GameState::new_with_deck(game_rules, deck)
            .expect("a freshly built deck always matches the configured rules");
        #[cfg(feature = "developer")]
        let game = {
            let mut deck = deck;
            if game_rules.developer_deck {
                deck.truncate(
                    usize::from(game_rules.player_count) * usize::from(game_rules.hand_size),
                );
                GameState::new_with_development_deck(game_rules, deck)
                    .expect("the development deck is a valid shuffled subset")
            } else {
                GameState::new_with_deck(game_rules, deck)
                    .expect("a freshly built deck always matches the configured rules")
            }
        };
        self.game = Some(game);
        self.match_id = Some(new_match_id());
        self.finished_reference_changes = None;
        self.match_profile_stats = vec![QiGui523ProfileStats::default(); self.players.len()];
        self.reset_auto_play_delay_for_current_turn();
        self.initialize_turn_timer();
        self.bump_revision();
        self.broadcast_game(origin)
    }

    pub(super) fn close_room(
        &mut self,
        connection: ConnectionId,
        request_id: RequestId,
    ) -> Vec<Delivery> {
        self.room
            .close_room(connection, request_id)
            .unwrap_or_else(|reason| self.reject(connection, request_id, reason))
    }

    pub(super) fn play_cards(
        &mut self,
        connection: ConnectionId,
        request_id: RequestId,
        cards: &[QiGuiCard],
    ) -> Vec<Delivery> {
        let Some(player) = self.player_id(connection) else {
            return self.reject(connection, request_id, RejectReason::NotJoined);
        };
        let Some(game) = self.game.as_mut() else {
            return self.reject(connection, request_id, RejectReason::GameNotStarted);
        };

        match game.play_cards(to_core_player(player), cards) {
            Ok(_) => {
                let play = classify(cards, &self.rules)
                    .expect("the game accepted a play that the shared classifier recognizes");
                let effect = (
                    player,
                    PublicPlay {
                        kind: play.kind().clone(),
                        cards: cards.to_vec(),
                    },
                );
                self.reset_timer_for_current_turn();
                self.bump_revision();
                self.broadcast_game_after_action(Some((connection, request_id)), Some(effect))
            }
            Err(error) => self.reject(
                connection,
                request_id,
                RejectReason::GameViolation(GameViolation::QiGui523(map_game_error(&error))),
            ),
        }
    }

    pub(super) fn pass(
        &mut self,
        connection: ConnectionId,
        request_id: RequestId,
    ) -> Vec<Delivery> {
        let Some(player) = self.player_id(connection) else {
            return self.reject(connection, request_id, RejectReason::NotJoined);
        };
        let Some(game) = self.game.as_mut() else {
            return self.reject(connection, request_id, RejectReason::GameNotStarted);
        };

        match game.pass(to_core_player(player)) {
            Ok(_) => {
                self.reset_timer_for_current_turn();
                self.bump_revision();
                self.broadcast_game_after_update(Some((connection, request_id)))
            }
            Err(error) => self.reject(
                connection,
                request_id,
                RejectReason::GameViolation(GameViolation::QiGui523(map_game_error(&error))),
            ),
        }
    }
}
