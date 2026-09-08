use super::*;
use leocard_protocol::{
    GameViolation, PlayerId, PlayerViolation, RejectReason, RequestId, RoomViolation, UnoCommand,
    UnoEvent, UnoProfileStats, UnoViolation,
};
use leocard_uno::UnoFlipSide;
use leocard_uno::{GameError, GameState, Phase, UnoRuleSet, build_deck_for_rules};

impl UnoSession {
    pub(super) fn handle_uno_command(
        &mut self,
        connection: ConnectionId,
        request_id: RequestId,
        command: UnoCommand,
    ) -> Vec<Delivery> {
        if self.pending_draw_reveal.is_some()
            && !matches!(
                command,
                UnoCommand::SetAutoPlay { .. } | UnoCommand::CallUno | UnoCommand::ReportUno { .. }
            )
        {
            return self.room.reject(
                connection,
                request_id,
                RejectReason::Game(GameViolation::Uno(UnoViolation::NotPlayersTurn)),
            );
        }
        match command {
            UnoCommand::SetAutoPlay { enabled } => {
                self.set_auto_play(connection, request_id, enabled)
            }
            UnoCommand::UpdateRules { rules } => self.update_rules(connection, request_id, rules),
            UnoCommand::ChooseInitialColor { color } => {
                self.perform_action(connection, request_id, Vec::new(), false, |game, player| {
                    game.choose_initial_color(player, color)
                })
            }
            UnoCommand::PlayCard { card, chosen_color } => self.perform_action(
                connection,
                request_id,
                vec![(card, chosen_color)],
                false,
                |game, player| {
                    let card = resolve_public_hand_card(game, player, card)?;
                    game.play_card(player, card, chosen_color)
                },
            ),
            UnoCommand::PlayCards {
                cards,
                chosen_color,
            } => self.perform_action(
                connection,
                request_id,
                cards
                    .iter()
                    .copied()
                    .map(|card| (card, chosen_color))
                    .collect(),
                false,
                move |game, player| {
                    let cards = cards
                        .into_iter()
                        .map(|card| resolve_public_hand_card(game, player, card))
                        .collect::<Result<Vec<_>, _>>()?;
                    game.play_cards(player, &cards, chosen_color)
                },
            ),
            UnoCommand::JumpIn { card } => self.perform_action(
                connection,
                request_id,
                vec![(card, None)],
                true,
                |game, player| {
                    let card = resolve_public_hand_card(game, player, card)?;
                    game.jump_in(player, card)
                },
            ),
            UnoCommand::ChooseSwapOneTarget { target } => {
                let core_target = to_core_player(target);
                self.perform_action(
                    connection,
                    request_id,
                    Vec::new(),
                    false,
                    move |game, player| {
                        let target_state = game
                            .player(core_target)
                            .ok_or(GameError::InvalidPlayer(core_target))?;
                        if target_state.hand().is_empty() {
                            return Err(GameError::InvalidSwapTargets);
                        }
                        let index = fastrand::usize(..target_state.hand().len());
                        game.choose_swap_one_target(player, core_target, index)
                    },
                )
            }
            UnoCommand::ChooseSevenSwapTarget { target } => {
                let target = to_core_player(target);
                self.perform_action(
                    connection,
                    request_id,
                    Vec::new(),
                    false,
                    move |game, player| game.choose_seven_swap_target(player, target),
                )
            }
            UnoCommand::GiveSwapOneCard { card } => {
                self.perform_action(connection, request_id, Vec::new(), false, |game, player| {
                    let card = resolve_public_hand_card(game, player, card)?;
                    game.give_swap_one_card(player, card)
                })
            }
            UnoCommand::ForceTradeHands { first, second } => {
                let first = to_core_player(first);
                let second = to_core_player(second);
                self.perform_action(
                    connection,
                    request_id,
                    Vec::new(),
                    false,
                    move |game, player| game.force_trade_hands(player, first, second),
                )
            }
            UnoCommand::DrawCard => self.perform_action(
                connection,
                request_id,
                Vec::new(),
                false,
                GameState::draw_card,
            ),
            UnoCommand::PassAfterDraw => self.perform_action(
                connection,
                request_id,
                Vec::new(),
                false,
                GameState::pass_after_draw,
            ),
            UnoCommand::AcceptDrawPenalty => self.perform_action(
                connection,
                request_id,
                Vec::new(),
                false,
                GameState::accept_draw_penalty,
            ),
            UnoCommand::ChallengeDrawFour => self.perform_action(
                connection,
                request_id,
                Vec::new(),
                false,
                GameState::challenge_draw_four,
            ),
            UnoCommand::ResolveSkip => self.perform_action(
                connection,
                request_id,
                Vec::new(),
                false,
                GameState::resolve_skip,
            ),
            UnoCommand::CallUno => self.perform_action(
                connection,
                request_id,
                Vec::new(),
                false,
                GameState::call_uno,
            ),
            UnoCommand::ReportUno { target } => {
                let core_target = to_core_player(target);
                self.perform_action(
                    connection,
                    request_id,
                    Vec::new(),
                    false,
                    move |game, reporter| game.report_uno(reporter, core_target),
                )
            }
        }
    }

    fn update_rules(
        &mut self,
        connection: ConnectionId,
        request_id: RequestId,
        rules: UnoRuleSet,
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
        let Ok(rules) = rules.validate() else {
            return self.room.reject(
                connection,
                request_id,
                RejectReason::Game(GameViolation::InvalidRuleConfiguration),
            );
        };
        if self.rules != rules {
            self.rules = rules;
            self.room.reset_ready_after_rules_change();
            let mut deck = build_deck_for_rules(rules);
            fastrand::shuffle(&mut deck);
            self.shuffled_deck = Some(deck);
            self.room.bump_revision();
        }
        self.broadcast_lobby(Some((connection, request_id)))
    }

    fn set_auto_play(
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
        if matches!(game.phase(), Phase::Finished(_)) {
            return self.reject_game_error(connection, request_id, &GameError::GameAlreadyFinished);
        }
        let participant = self
            .room
            .players
            .iter_mut()
            .find(|participant| participant.id == player)
            .expect("joined player belongs to room");
        if participant.auto_play != enabled {
            participant.auto_play = enabled;
            self.room.bump_revision();
        }
        self.reset_auto_play_delay();
        self.broadcast_game(Some((connection, request_id)))
    }

    pub(super) fn start_game(
        &mut self,
        connection: ConnectionId,
        request_id: RequestId,
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
                RejectReason::Room(RoomViolation::OnlyHostCanStart),
            );
        }
        let active = self
            .room
            .players
            .iter()
            .filter(|player| !player.left)
            .count();
        if active < usize::from(UnoRuleSet::MIN_PLAYERS) {
            return self.room.reject(
                connection,
                request_id,
                RejectReason::Room(RoomViolation::NotEnoughPlayers {
                    minimum: UnoRuleSet::MIN_PLAYERS,
                    actual: active as u8,
                }),
            );
        }
        if self
            .room
            .players
            .iter()
            .any(|player| !player.left && player.seat.is_none())
        {
            return self.room.reject(
                connection,
                request_id,
                RejectReason::Room(RoomViolation::MustSelectSeat),
            );
        }
        let not_ready = self
            .room
            .players
            .iter()
            .filter(|player| !player.left && !player.ready)
            .map(|player| player.id)
            .collect::<Vec<_>>();
        if !not_ready.is_empty() {
            return self.room.reject(
                connection,
                request_id,
                RejectReason::Room(RoomViolation::PlayersNotReady { players: not_ready }),
            );
        }
        self.room.remove_departed_players();
        self.room.players.sort_by_key(|player| {
            player
                .seat
                .expect("start validation required every player to select a seat")
                .0
        });
        for (index, player) in self.room.players.iter_mut().enumerate() {
            player.id = PlayerId(index as u8);
            player.ready = false;
            player.auto_play = player.is_bot;
        }
        let deck = self
            .shuffled_deck
            .take()
            .unwrap_or_else(|| shuffled_uno_deck(self.rules));
        match GameState::new_with_deck(self.rules, active as u8, deck) {
            Ok(game) => {
                let started_on_dark = game.flip_side() == Some(UnoFlipSide::Dark);
                self.game = Some(game);
                self.pending_draw_reveal = None;
                self.match_id = Some(new_match_id());
                self.match_profile_stats = vec![UnoProfileStats::default(); active];
                self.record_state_peaks();
                self.finished_reference_changes = None;
                self.reset_auto_play_delay();
                self.room.bump_revision();
                let mut deliveries = self.broadcast_game(Some((connection, request_id)));
                if started_on_dark {
                    deliveries.extend(self.broadcast_events(vec![UnoEvent::Flipped {
                        side: UnoFlipSide::Dark,
                    }]));
                }
                deliveries
            }
            Err(_) => self.room.reject(
                connection,
                request_id,
                RejectReason::Game(GameViolation::InvalidRuleConfiguration),
            ),
        }
    }

    pub(super) fn return_to_lobby(
        &mut self,
        connection: ConnectionId,
        request_id: RequestId,
    ) -> Vec<Delivery> {
        if self.room.player_id(connection).is_none() {
            return self.room.reject(
                connection,
                request_id,
                RejectReason::Player(PlayerViolation::NotJoined),
            );
        }
        if self.room.host_connection != Some(connection) {
            return self.room.reject(
                connection,
                request_id,
                RejectReason::Room(RoomViolation::OnlyHostCanReturnToLobby),
            );
        }
        if !self
            .game
            .as_ref()
            .is_some_and(|game| matches!(game.phase(), Phase::Finished(_)))
        {
            return self.room.reject(
                connection,
                request_id,
                RejectReason::Game(GameViolation::GameNotFinished),
            );
        }
        self.game = None;
        self.pending_draw_reveal = None;
        self.match_id = None;
        self.match_profile_stats.clear();
        self.finished_reference_changes = None;
        self.auto_play_delay = None;
        #[cfg(feature = "developer")]
        self.room.remove_developer_bots();
        let host = self.room.host_connection;
        for player in &mut self.room.players {
            player.ready = host == Some(player.connection);
            player.auto_play = false;
            if !player.connected {
                player.seat = None;
            }
        }
        self.shuffled_deck = Some(shuffled_uno_deck(self.rules));
        self.room.bump_revision();
        self.broadcast_lobby(Some((connection, request_id)))
    }

    pub(super) fn play_again(
        &mut self,
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
        if !self
            .game
            .as_ref()
            .is_some_and(|game| matches!(game.phase(), Phase::Finished(_)))
        {
            return self.room.reject(
                connection,
                request_id,
                RejectReason::Game(GameViolation::GameNotFinished),
            );
        }
        if let Some(participant) = self.room.players.iter_mut().find(|item| item.id == player) {
            participant.ready = true;
        }
        self.room.bump_revision();
        let active = self.room.players.iter().filter(|player| !player.left);
        if active.clone().count() >= usize::from(UnoRuleSet::MIN_PLAYERS)
            && active.clone().all(|player| player.ready)
        {
            return self.start_next_game(Some((connection, request_id)));
        }
        self.broadcast_game(Some((connection, request_id)))
    }

    pub(super) fn start_next_game(
        &mut self,
        origin: Option<(ConnectionId, RequestId)>,
    ) -> Vec<Delivery> {
        self.room.remove_departed_players();
        for player in &mut self.room.players {
            player.ready = false;
        }
        let player_count = self.room.players.len() as u8;
        let game =
            GameState::new_with_deck(self.rules, player_count, shuffled_uno_deck(self.rules))
                .expect("a freshly built UNO deck is valid");
        self.game = Some(game);
        self.pending_draw_reveal = None;
        self.match_id = Some(new_match_id());
        self.match_profile_stats = vec![UnoProfileStats::default(); usize::from(player_count)];
        self.record_state_peaks();
        self.finished_reference_changes = None;
        self.reset_auto_play_delay();
        self.room.bump_revision();
        self.broadcast_game(origin)
    }
}
