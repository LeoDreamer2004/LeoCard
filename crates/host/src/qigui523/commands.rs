use super::{QiGui523Session, map_game_error, to_core_player};
use crate::lifecycle::HostedGameLifecycle;
use crate::{AUTO_PLAY_DELAY, AutoPlayDelayState, ConnectionId, Delivery, new_match_id};
use leocard_protocol::{
    GameViolation, PlayerViolation, PublicPlay, QiGui523ProfileStats, RejectReason, RequestId,
    RoomViolation, RuleViolation, TABLE_SEAT_COUNT,
};
use leocard_qigui523::{GameState, Phase, QiGuiCard, QiGuiRuleSet, build_deck, classify};

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
        if cards.is_empty()
            || game
                .replace_player_hand(to_core_player(player), cards)
                .is_err()
        {
            return self.reject(
                connection,
                request_id,
                RejectReason::Game(GameViolation::InvalidDeveloperHand),
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
        if !matches!(game.phase(), Phase::Playing) {
            return self.reject(
                connection,
                request_id,
                RejectReason::Game(GameViolation::QiGui523(RuleViolation::GameAlreadyFinished)),
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
        let Some(game) = self.game.as_mut() else {
            return self.reject(
                connection,
                request_id,
                RejectReason::Game(GameViolation::GameNotStarted),
            );
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
        let Some(game) = self.game.as_mut() else {
            return self.reject(
                connection,
                request_id,
                RejectReason::Game(GameViolation::GameNotStarted),
            );
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
                RejectReason::Game(GameViolation::QiGui523(map_game_error(&error))),
            ),
        }
    }
}
