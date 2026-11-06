use super::{QiGui523Session, map_game_error};
use crate::features::GameCommandHandler;
use crate::lifecycle::HostedGameLifecycle;
use crate::{
    AUTOMATIC_ACTION_DELAY, AutomaticActionDelayState, ConnectionId, Delivery, new_match_id,
};
use leocard_game_common::PlayerId;
use leocard_protocol::{
    GameCommand, GameKind, GameViolation, PlayerViolation, QiGui523Command, QiGui523Event,
    QiGui523PublicPlay, QiGui523Violation, RejectReason, RequestId, RoomViolation,
    TABLE_SEAT_COUNT,
};
use leocard_qigui523::{
    QiGuiActionContext, QiGuiCard, QiGuiGameError, QiGuiGameState, QiGuiMatchStatistics,
    QiGuiPhase, QiGuiRuleSet, build_deck, classify,
};

impl QiGui523Session {
    #[cfg(feature = "developer")]
    pub(super) fn set_developer_hand(
        &mut self,
        connection: ConnectionId,
        request_id: RequestId,
        cards: Vec<QiGuiCard>,
    ) -> Vec<Delivery> {
        let Some(player) = self.player_id(connection) else {
            return self.reject(
                connection,
                request_id,
                RejectReason::Player(PlayerViolation::NotJoined),
            );
        };
        let Some(game) = self.game.as_mut() else {
            return self.reject(
                connection,
                request_id,
                RejectReason::Game(GameViolation::GameNotStarted),
            );
        };
        if cards.is_empty() || game.replace_player_hand(player, cards).is_err() {
            return self.reject(
                connection,
                request_id,
                RejectReason::Game(QiGui523Violation::InvalidDeveloperHand.into()),
            );
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
            RejectReason::Game(GameViolation::DeveloperFeatureUnavailable),
        )
    }

    pub(super) fn set_auto_play(
        &mut self,
        connection: ConnectionId,
        request_id: RequestId,
        enabled: bool,
    ) -> Vec<Delivery> {
        let Some(player) = self.player_id(connection) else {
            return self.reject(
                connection,
                request_id,
                RejectReason::Player(PlayerViolation::NotJoined),
            );
        };
        let Some(game) = self.game.as_ref() else {
            return self.reject(
                connection,
                request_id,
                RejectReason::Game(GameViolation::GameNotStarted),
            );
        };
        if !matches!(game.phase(), QiGuiPhase::Playing) {
            return self.reject(
                connection,
                request_id,
                RejectReason::Game(GameViolation::QiGui523(
                    QiGui523Violation::GameAlreadyFinished,
                )),
            );
        }

        let participant = &mut self.players[usize::from(player.0)];
        let changed = participant.auto_play != enabled;
        participant.auto_play = enabled;
        if self.current_player() == Some(player) {
            self.automatic_action_delay = enabled.then_some(AutomaticActionDelayState {
                player,
                remaining: AUTOMATIC_ACTION_DELAY,
            });
        } else if self
            .automatic_action_delay
            .as_ref()
            .is_some_and(|delay| delay.player == player)
        {
            self.automatic_action_delay = None;
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
            return self.reject(
                connection,
                request_id,
                RejectReason::Player(PlayerViolation::NotJoined),
            );
        }
        if self.game.is_some() {
            return self.reject(
                connection,
                request_id,
                RejectReason::Game(GameViolation::GameAlreadyStarted),
            );
        }
        if self.host_connection != Some(connection) {
            return self.reject(
                connection,
                request_id,
                RejectReason::Room(RoomViolation::OnlyHostCanConfigure),
            );
        }
        let rules = QiGuiRuleSet {
            player_count: TABLE_SEAT_COUNT,
            ..rules
        };
        let Ok(rules) = rules.validate() else {
            return self.reject(
                connection,
                request_id,
                RejectReason::Game(GameViolation::InvalidRuleConfiguration),
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

    pub(super) fn start_next_game(
        &mut self,
        origin: Option<(ConnectionId, RequestId)>,
    ) -> Vec<Delivery> {
        self.game = None;
        self.turn_timer = None;
        self.automatic_action_delay = None;
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
        let game = QiGuiGameState::new_with_deck(game_rules, deck)
            .expect("a freshly built deck always matches the configured rules");
        #[cfg(feature = "developer")]
        let game = {
            let mut deck = deck;
            if game_rules.developer_deck {
                deck.truncate(
                    usize::from(game_rules.player_count) * usize::from(game_rules.hand_size),
                );
                QiGuiGameState::new_with_development_deck(game_rules, deck)
                    .expect("the development deck is a valid shuffled subset")
            } else {
                QiGuiGameState::new_with_deck(game_rules, deck)
                    .expect("a freshly built deck always matches the configured rules")
            }
        };
        self.statistics = Some(QiGuiMatchStatistics::new(&game));
        self.game = Some(game);
        self.match_id = Some(new_match_id());
        self.finished_reference_changes = None;
        self.reset_automatic_action_delay_for_current_turn();
        self.initialize_turn_timer();
        self.bump_revision();
        self.broadcast_game(origin)
    }

    pub(super) fn play_cards(
        &mut self,
        connection: ConnectionId,
        request_id: RequestId,
        cards: &[QiGuiCard],
    ) -> Vec<Delivery> {
        let Some(player) = self.player_id(connection) else {
            return self.reject(
                connection,
                request_id,
                RejectReason::Player(PlayerViolation::NotJoined),
            );
        };
        let Some(_) = self.game.as_ref() else {
            return self.reject(
                connection,
                request_id,
                RejectReason::Game(GameViolation::GameNotStarted),
            );
        };

        match self.perform_action(player, Some(cards)) {
            Ok(events) => {
                self.reset_timer_for_current_turn();
                self.bump_revision();
                self.broadcast_game_after_action(Some((connection, request_id)), events)
            }
            Err(error) => self.reject(
                connection,
                request_id,
                RejectReason::Game(GameViolation::QiGui523(map_game_error(&error))),
            ),
        }
    }

    pub(super) fn pass(
        &mut self,
        connection: ConnectionId,
        request_id: RequestId,
    ) -> Vec<Delivery> {
        let Some(player) = self.player_id(connection) else {
            return self.reject(
                connection,
                request_id,
                RejectReason::Player(PlayerViolation::NotJoined),
            );
        };
        let Some(_) = self.game.as_ref() else {
            return self.reject(
                connection,
                request_id,
                RejectReason::Game(GameViolation::GameNotStarted),
            );
        };

        match self.perform_action(player, None) {
            Ok(events) => {
                self.reset_timer_for_current_turn();
                self.bump_revision();
                self.broadcast_game_after_action(Some((connection, request_id)), events)
            }
            Err(error) => self.reject(
                connection,
                request_id,
                RejectReason::Game(GameViolation::QiGui523(map_game_error(&error))),
            ),
        }
    }
}

impl GameCommandHandler for QiGui523Session {
    type Command = QiGui523Command;

    fn parse_command(command: GameCommand) -> Result<Self::Command, GameKind> {
        let GameCommand::QiGui523(command) = command else {
            return Err(command.kind());
        };
        Ok(command)
    }

    fn handle_command(
        &mut self,
        connection: ConnectionId,
        request_id: RequestId,
        command: Self::Command,
    ) -> Vec<Delivery> {
        match command {
            QiGui523Command::SetAutoPlay { enabled } => {
                self.set_auto_play(connection, request_id, enabled)
            }
            QiGui523Command::UpdateRules { rules } => {
                self.update_rules(connection, request_id, rules)
            }
            QiGui523Command::PlayCards { cards } => self.play_cards(connection, request_id, &cards),
            QiGui523Command::SetDeveloperHand { cards } => {
                self.set_developer_hand(connection, request_id, cards)
            }
            QiGui523Command::Pass => self.pass(connection, request_id),
        }
    }
}

impl QiGui523Session {
    pub(super) fn perform_action(
        &mut self,
        player: PlayerId,
        cards: Option<&[QiGuiCard]>,
    ) -> Result<Vec<QiGui523Event>, QiGuiGameError> {
        let game = self.game.as_mut().expect("actions require an active game");
        let before = QiGuiActionContext::capture(game, player);
        let outcome = match cards {
            Some(cards) => game.play_cards(player, cards)?,
            None => game.pass(player)?,
        };
        let play =
            cards.map(|cards| classify(cards, game.rules()).expect("accepted play is valid"));
        let mut events = Vec::new();
        if let Some(play) = &play {
            events.push(QiGui523Event::PlayEffect {
                player,
                play: QiGui523PublicPlay {
                    kind: play.kind().clone(),
                    cards: play.cards().to_vec(),
                },
            });
        } else {
            events.push(QiGui523Event::Passed { player });
        }
        let statistics = self
            .statistics
            .as_mut()
            .expect("an active game has statistics");
        events.extend(
            statistics
                .observe(
                    before.expect("an accepted action has a playing context"),
                    &outcome,
                    play.as_ref(),
                    game,
                )
                .into_iter()
                .map(|statistics| QiGui523Event::ActionAnalyzed {
                    player: statistics.player,
                    statistics: Box::new(statistics),
                }),
        );
        Ok(events)
    }
}
