use super::statistics::GuandanStatistics;
use crate::action_timing::attach_action_timing;
use crate::lifecycle::HostedGameLifecycle;
use crate::{ConnectionId, Delivery, HostError, RoomSession, new_match_id};
use leocard_game_common::PlayerId;
use leocard_guandan::{
    GuandanAction, GuandanCard, GuandanGameState, GuandanMatchState, GuandanPhase,
    GuandanProfileStats, GuandanRuleSet, auto_play_action, build_deck,
};
use leocard_protocol::{
    ClientMessage, GuandanEvent, GuandanPublicAction, MatchId, PlayerReferenceChange, RejectReason,
    RequestId, Revision, RoomId, ServerEvent,
};
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct GuandanSession {
    pub(super) room: RoomSession,
    pub(super) rules: GuandanRuleSet,
    pub(super) deck: Option<Vec<GuandanCard>>,
    pub(super) game: Option<GuandanGameState>,
    pub(super) match_id: Option<MatchId>,
    pub(super) match_state: Option<GuandanMatchState>,
    pub(super) finished_reference_changes: Option<Vec<PlayerReferenceChange>>,
    pub(super) automatic_elapsed: Duration,
    pub(super) statistics: GuandanStatistics,
}

impl GuandanSession {
    pub fn new(
        room_id: RoomId,
        host_port: u16,
        rules: GuandanRuleSet,
        deck: Vec<GuandanCard>,
    ) -> Result<Self, HostError> {
        GuandanGameState::new(rules, deck.clone(), PlayerId(0))?;
        Ok(Self {
            room: RoomSession::new_with_seat_count(room_id, host_port, 4, 4),
            rules,
            deck: Some(deck),
            game: None,
            match_id: None,
            match_state: None,
            finished_reference_changes: None,
            automatic_elapsed: Duration::ZERO,
            statistics: GuandanStatistics::default(),
        })
    }

    pub fn room_id(&self) -> RoomId {
        self.room.room_id
    }

    pub fn revision(&self) -> Revision {
        self.room.revision
    }

    pub fn is_closed(&self) -> bool {
        self.room.closed
    }

    pub fn is_current_connection(&self, connection: ConnectionId) -> bool {
        self.room.player_id(connection).is_some()
    }

    pub fn heartbeat(&self) -> Vec<Delivery> {
        self.room.heartbeat()
    }

    pub fn handle(&mut self, connection: ConnectionId, message: ClientMessage) -> Vec<Delivery> {
        self.dispatch_with_action_timing(connection, message)
    }

    pub fn advance_time(&mut self, elapsed: Duration) -> Vec<Delivery> {
        self.room.action_timing.advance(elapsed);
        let deliveries = self.advance_game_time(elapsed);
        self.refresh_action_timing();
        attach_action_timing(self, deliveries)
    }

    fn advance_game_time(&mut self, elapsed: Duration) -> Vec<Delivery> {
        let Some(game) = &self.game else {
            return Vec::new();
        };
        if elapsed.is_zero() {
            return Vec::new();
        }
        if matches!(game.phase(), GuandanPhase::Finished(_)) {
            if !self.match_state.as_ref().expect("active match").complete() && self.everyone_ready()
            {
                self.automatic_elapsed += elapsed;
                if self.automatic_elapsed >= Duration::from_secs(3) {
                    self.continue_match();
                    self.room.bump_revision();
                    return self.broadcast_game(None);
                }
            }
            return Vec::new();
        }
        let Some(player) = game.current_player() else {
            return Vec::new();
        };
        let participant = &self.room.players[player.index()];
        if !(participant.is_bot
            || participant.auto_play
            || !participant.connected
            || participant.left)
        {
            self.automatic_elapsed = Duration::ZERO;
            return Vec::new();
        }
        self.automatic_elapsed += elapsed;
        if self.automatic_elapsed < Duration::from_millis(800) {
            return Vec::new();
        }
        let Some(action) = auto_play_action(game, player) else {
            return Vec::new();
        };
        self.apply_action(player, action, None)
    }

    pub fn disconnect(&mut self, connection: ConnectionId) -> Vec<Delivery> {
        if self.room.disconnect_spectator(connection) {
            return Vec::new();
        }
        let Some(player) = self
            .room
            .players
            .iter_mut()
            .find(|player| player.connection == connection && player.connected)
        else {
            return Vec::new();
        };
        player.connected = false;
        if self.game.is_none() {
            player.left = true;
            player.seat = None;
            player.ready = false;
        }
        self.room.bump_revision();
        if self.room.host_connection == Some(connection) {
            self.room.closed = true;
            self.room.broadcast_event(None, ServerEvent::RoomClosed)
        } else if self.game.is_some() {
            self.broadcast_game(None)
        } else {
            self.broadcast_lobby(None)
        }
    }

    pub(super) fn begin_match(&mut self) {
        self.statistics = GuandanStatistics::default();
        self.match_state = Some(
            GuandanMatchState::new(self.rules.starting_level, PlayerId(fastrand::u8(..4)))
                .expect("validated rules"),
        );
        self.match_id = Some(new_match_id());
        self.finished_reference_changes = None;
        for player in &mut self.room.players {
            player.auto_play = false;
        }
        let deck = self.deck.take().unwrap_or_else(|| {
            let mut deck = build_deck();
            fastrand::shuffle(&mut deck);
            deck
        });
        self.deal_hand(deck);
    }

    pub(super) fn continue_match(&mut self) {
        if self.match_state.as_ref().expect("active match").complete() {
            self.begin_match();
            return;
        }
        assert!(self.match_state.as_mut().expect("active match").next_hand());
        let mut deck = build_deck();
        fastrand::shuffle(&mut deck);
        self.deal_hand(deck);
    }

    fn deal_hand(&mut self, deck: Vec<GuandanCard>) {
        let state = self.match_state.as_ref().expect("active match");
        let rules = GuandanRuleSet {
            starting_level: state.current_level(),
        };
        self.game = Some(
            GuandanGameState::new_with_tribute(
                rules,
                deck,
                state.next_starter(),
                state.previous_result().cloned(),
            )
            .expect("validated deck and rules"),
        );
        self.automatic_elapsed = Duration::ZERO;
        for player in &mut self.room.players {
            player.ready = player.is_bot;
        }
    }

    pub(super) fn everyone_ready(&self) -> bool {
        self.room
            .players
            .iter()
            .all(|player| player.ready || player.is_bot || player.left || !player.connected)
    }

    pub(super) fn apply_action(
        &mut self,
        player: PlayerId,
        action: GuandanAction,
        origin: Option<(ConnectionId, RequestId)>,
    ) -> Vec<Delivery> {
        let game = self.game.as_mut().expect("action requires a game");
        if let Err(error) = game.act(player, action.clone()) {
            return origin.map_or_else(Vec::new, |(connection, request)| {
                self.room
                    .reject(connection, request, RejectReason::Game(error.into()))
            });
        }
        self.automatic_elapsed = Duration::ZERO;
        if matches!(action, GuandanAction::Play(_)) {
            self.statistics
                .observe_play(player, game.winning_play().expect("accepted play"));
        }
        let event = match action {
            GuandanAction::Play(cards) => {
                let public = GuandanPublicAction::Play(cards);
                GuandanEvent::ActionApplied {
                    player,
                    action: public,
                }
            }
            GuandanAction::Pass => {
                let public = GuandanPublicAction::Pass;
                GuandanEvent::ActionApplied {
                    player,
                    action: public,
                }
            }
            GuandanAction::Tribute(_) | GuandanAction::ReturnTribute(_) => {
                GuandanEvent::TributeSubmitted { player }
            }
        };
        let mut events = vec![event];
        if let GuandanPhase::Finished(result) = game.phase() {
            let result = result.clone();
            let state = self.match_state.as_mut().expect("active match");
            assert!(state.finish_hand(&result));
            self.statistics.finish_hand(&result, state.level_advance());
            self.room.prepare_rematch();
            events.push(GuandanEvent::HandFinished { result });
            if self.match_state.as_ref().expect("active match").complete() {
                self.settle_match();
            }
        }
        self.room.bump_revision();
        let hand_index = self
            .match_state
            .as_ref()
            .expect("active match")
            .hand_number();
        let mut deliveries = self
            .room
            .broadcast_game_events(events, self.match_id.map(|id| (id, Some(hand_index))));
        deliveries.extend(self.broadcast_game(origin));
        deliveries
    }

    pub(super) fn settle_match(&mut self) {
        let winner = self
            .match_state
            .as_ref()
            .expect("active match")
            .previous_result()
            .expect("completed match")
            .winning_team;
        self.room.settle_completed_match_profiles_once(
            &mut self.finished_reference_changes,
            (0..4).map(|index| (PlayerId(index), 0)),
            |player, delta| {
                let aggregate = player
                    .game_profiles
                    .guandan
                    .get_or_insert_with(GuandanProfileStats::default);
                self.statistics
                    .merge_into(player.id, aggregate, player.id.0 % 2 == winner, delta);
            },
        );
    }
}
