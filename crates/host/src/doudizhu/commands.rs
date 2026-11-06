use super::DoudizhuSession;
use crate::features::{GameCommandHandler, HandSettlement};
use crate::lifecycle::HostedGameLifecycle;
use crate::{ConnectionId, Delivery};
use leocard_doudizhu::achievements::DoudizhuActionContext;
use leocard_doudizhu::{DoudizhuAction, DoudizhuPhase, build_deck};
use leocard_game_common::PlayerId;
use leocard_protocol::{
    DoudizhuCommand, DoudizhuEvent, GameCommand, GameEvent, GameKind, GameViolation,
    PlayerViolation, RejectReason, RequestId, RoomViolation, ServerEvent,
};
use std::time::Duration;

impl DoudizhuSession {
    pub(super) fn apply_action(
        &mut self,
        player: PlayerId,
        action: DoudizhuAction,
        origin: Option<(ConnectionId, RequestId)>,
    ) -> Vec<Delivery> {
        let game = self.game.as_mut().expect("action requires a game");
        let before = DoudizhuActionContext::capture(game);
        let playing = game.phase == DoudizhuPhase::Playing;
        if let Err(error) = game.act(player, action.clone()) {
            return origin.map_or_else(Vec::new, |(connection, request)| {
                self.room
                    .reject(connection, request, RejectReason::Game(error.into()))
            });
        }
        if playing {
            self.bot_history.push((player, action.clone()));
        }
        self.statistics.observe(player, &action, game);
        let achievements = self
            .achievements
            .observe_action(player, &action, before, game);
        let mut events = vec![DoudizhuEvent::ActionApplied { player, action }];
        events.extend(achievements.into_iter().map(|(player, statistics)| {
            DoudizhuEvent::ActionAnalyzed {
                player,
                statistics: Box::new(statistics),
            }
        }));
        if game.phase == DoudizhuPhase::Redeal {
            self.redeal_hand();
            events.push(DoudizhuEvent::Redealt);
        }
        if let Some(result) = self.game.as_ref().and_then(|g| g.result.clone()) {
            events.push(DoudizhuEvent::HandFinished { result });
            events.extend(self.commit_hand(&()));
        }
        self.reset_automatic_delay();
        self.room.bump_revision();
        let hand_index = u32::from(
            self.match_state
                .as_ref()
                .expect("active match")
                .progress()
                .hand_number,
        );
        let mut deliveries: Vec<_> =
            self.room
                .broadcast_game_events(events, self.match_id.map(|id| (id, Some(hand_index))))
                .into_iter()
                .filter(|delivery| match &delivery.message.event {
                    ServerEvent::GameEvent(GameEvent::Doudizhu(
                        DoudizhuEvent::ActionAnalyzed { player, .. },
                    )) => self.room.player_id(delivery.recipient) == Some(*player),
                    _ => true,
                })
                .collect();
        deliveries.extend(self.broadcast_game(origin));
        deliveries
    }
}

impl GameCommandHandler for DoudizhuSession {
    type Command = DoudizhuCommand;

    fn parse_command(command: GameCommand) -> Result<Self::Command, GameKind> {
        let GameCommand::Doudizhu(command) = command else {
            return Err(command.kind());
        };
        Ok(command)
    }

    fn handle_command(
        &mut self,
        connection: ConnectionId,
        request: RequestId,
        command: Self::Command,
    ) -> Vec<Delivery> {
        let Some(player) = self.room.player_id(connection) else {
            return self.room.reject(
                connection,
                request,
                RejectReason::Player(PlayerViolation::NotJoined),
            );
        };
        let result: Result<(), RejectReason> = match command {
            DoudizhuCommand::UpdateRules { rules } => {
                if self.game.is_some() {
                    Err(RejectReason::Game(GameViolation::GameAlreadyStarted))
                } else if self.room.host_connection != Some(connection) {
                    Err(RejectReason::Room(RoomViolation::OnlyHostCanConfigure))
                } else if !rules.is_valid() {
                    Err(RejectReason::Game(GameViolation::InvalidRuleConfiguration))
                } else if self.room.players.iter().filter(|p| !p.left).count()
                    > usize::from(rules.mode.players())
                {
                    Err(RejectReason::Room(RoomViolation::RoomFull))
                } else {
                    if rules != self.rules {
                        self.room
                            .resize_seats(rules.mode.players())
                            .expect("validated rules and player count fit the new seats");
                        self.rules = rules;
                        self.room.reset_ready_after_rules_change();
                        let mut deck = build_deck(rules.mode);
                        fastrand::shuffle(&mut deck);
                        self.deck = Some(deck);
                        self.room.bump_revision();
                    }
                    return self.broadcast_lobby(Some((connection, request)));
                }
            }
            DoudizhuCommand::SetAutoPlay { enabled } => {
                if self
                    .game
                    .as_ref()
                    .is_none_or(|g| g.phase == DoudizhuPhase::Finished)
                {
                    Err(RejectReason::Game(GameViolation::GameNotStarted))
                } else {
                    self.room.players[usize::from(player.0)].auto_play = enabled;
                    self.automatic_elapsed = Duration::ZERO;
                    self.room.bump_revision();
                    return self.broadcast_game(Some((connection, request)));
                }
            }
            DoudizhuCommand::Act(action) => {
                if self.game.is_none() {
                    Err(RejectReason::Game(GameViolation::GameNotStarted))
                } else {
                    return self.apply_action(player, action, Some((connection, request)));
                }
            }
        };
        self.room.reject(
            connection,
            request,
            result.expect_err("success returns above"),
        )
    }
}
