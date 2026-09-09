use super::{
    MAHJONG_DEAL_INTERVAL, MahjongSession, events_for_outcome, shuffled_deck, to_core_player,
};
use crate::lifecycle::HostedGameLifecycle;
use crate::{ConnectionId, Delivery, new_match_id};
use leocard_mahjong::{
    ActionOutcome, GameError, GameState, MahjongPlayerId, MahjongRuleSet, MahjongTileKind, Phase,
};
use leocard_protocol::{
    GameViolation, MahjongCommand, MahjongEvent, PlayerId, PlayerViolation, RejectReason,
    RequestId, RoomViolation,
};
use std::time::Duration;

impl MahjongSession {
    pub(super) fn handle_game_command(
        &mut self,
        connection: ConnectionId,
        request_id: RequestId,
        command: MahjongCommand,
    ) -> Vec<Delivery> {
        match command {
            MahjongCommand::UpdateRules { rules } => {
                self.update_rules(connection, request_id, rules)
            }
            MahjongCommand::SetDeveloperHand { tiles } => {
                self.set_developer_hand(connection, request_id, tiles)
            }
            MahjongCommand::Discard { tile } => self.perform_action(
                connection,
                request_id,
                Some(MahjongEvent::TileDiscarded {
                    player: self.room.player_id(connection).unwrap_or(PlayerId(u8::MAX)),
                    tile,
                }),
                move |game, player| game.discard(player, tile),
            ),
            MahjongCommand::RespondToClaim { claim } => {
                self.perform_action(connection, request_id, None, move |game, player| {
                    game.respond_to_claim(player, claim)
                })
            }
            MahjongCommand::DeclareSelfDraw => {
                self.perform_action(connection, request_id, None, GameState::declare_self_draw)
            }
            MahjongCommand::DeclareConcealedKong { tile } => {
                self.perform_action(connection, request_id, None, move |game, player| {
                    game.declare_concealed_kong(player, tile)
                })
            }
            MahjongCommand::DeclareAddedKong { tile } => {
                self.perform_action(connection, request_id, None, move |game, player| {
                    game.declare_added_kong(player, tile)
                })
            }
        }
    }

    #[cfg(feature = "developer")]
    fn set_developer_hand(
        &mut self,
        connection: ConnectionId,
        request_id: RequestId,
        tiles: Vec<MahjongTileKind>,
    ) -> Vec<Delivery> {
        let Some(player) = self.room.player_id(connection) else {
            return self.room.reject(
                connection,
                request_id,
                RejectReason::Player(PlayerViolation::NotJoined),
            );
        };
        let Some(game) = self.game.as_mut() else {
            return self.room.reject(
                connection,
                request_id,
                RejectReason::Game(GameViolation::GameNotStarted),
            );
        };
        if let Err(error) = game.replace_player_hand_from_wall(to_core_player(player), &tiles) {
            return self.reject_game_error(connection, request_id, &error);
        }
        self.room.bump_revision();
        self.broadcast_game(Some((connection, request_id)))
    }

    #[cfg(not(feature = "developer"))]
    fn set_developer_hand(
        &self,
        connection: ConnectionId,
        request_id: RequestId,
        _tiles: Vec<MahjongTileKind>,
    ) -> Vec<Delivery> {
        self.room.reject(
            connection,
            request_id,
            RejectReason::Game(GameViolation::DeveloperFeatureUnavailable),
        )
    }

    fn update_rules(
        &mut self,
        connection: ConnectionId,
        request_id: RequestId,
        rules: MahjongRuleSet,
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
            self.shuffled_deck = Some(shuffled_deck());
            self.room.bump_revision();
        }
        self.broadcast_lobby(Some((connection, request_id)))
    }

    pub(super) fn start_next_hand(
        &mut self,
        origin: Option<(ConnectionId, RequestId)>,
    ) -> Vec<Delivery> {
        let completed_match = self.game.as_ref().is_some_and(
            |game| matches!(game.phase(), Phase::Finished(result) if result.match_complete),
        );
        let game = self.game.as_mut().expect("rematch requires an active game");
        game.start_next_hand(shuffled_deck())
            .expect("a freshly built Mahjong wall is valid");
        self.deal_delay = Duration::ZERO;
        if completed_match {
            self.match_id = Some(new_match_id());
        }
        for player in &mut self.room.players {
            player.ready = false;
        }
        self.room.bump_revision();
        self.broadcast_game(origin)
    }

    fn perform_action<F>(
        &mut self,
        connection: ConnectionId,
        request_id: RequestId,
        public_event: Option<MahjongEvent>,
        action: F,
    ) -> Vec<Delivery>
    where
        F: FnOnce(&mut GameState, MahjongPlayerId) -> Result<ActionOutcome, GameError>,
    {
        let Some(player) = self.room.player_id(connection) else {
            return self.room.reject(
                connection,
                request_id,
                RejectReason::Player(PlayerViolation::NotJoined),
            );
        };
        let Some(game) = self.game.as_mut() else {
            return self.room.reject(
                connection,
                request_id,
                RejectReason::Game(GameViolation::GameNotStarted),
            );
        };
        let dead_before = game
            .players()
            .iter()
            .map(|player| player.is_dead_hand())
            .collect::<Vec<_>>();
        match action(game, to_core_player(player)) {
            Ok(outcome) => {
                let mut events = Vec::new();
                if let Some(event) = public_event {
                    events.push(event);
                }
                events.extend(events_for_outcome(&outcome));
                for (index, core_player) in game.players().iter().enumerate() {
                    let already_reported = matches!(
                        outcome,
                        ActionOutcome::FalseWin { player, .. } if player.0 == index
                    );
                    if !already_reported && !dead_before[index] && core_player.is_dead_hand() {
                        let mut deltas = [10; MahjongRuleSet::PLAYER_COUNT];
                        deltas[index] = -30;
                        events.push(MahjongEvent::FalseWin {
                            player: PlayerId(index as u8),
                            deltas,
                        });
                    }
                }
                if matches!(game.phase(), Phase::Finished(_)) {
                    self.room.prepare_rematch();
                }
                if matches!(game.phase(), Phase::ReplacingFlower { .. }) {
                    self.deal_delay = MAHJONG_DEAL_INTERVAL;
                }
                self.room.bump_revision();
                let mut deliveries = self.broadcast_events(events);
                deliveries.extend(self.broadcast_game(Some((connection, request_id))));
                deliveries
            }
            Err(error) => self.reject_game_error(connection, request_id, &error),
        }
    }
}
