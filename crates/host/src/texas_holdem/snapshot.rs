use super::*;
use leocard_protocol::{
    GameEvent, GameKind, GameRules, GameSnapshot, LobbySnapshot, PlayerId, RequestId, ServerEvent,
    TexasHoldemEvent, TexasHoldemPhaseView, TexasHoldemSnapshot,
};

impl TexasHoldemSession {
    pub(super) fn lobby_snapshot(&self) -> LobbySnapshot {
        self.room
            .lobby_snapshot(GameKind::TexasHoldem, GameRules::TexasHoldem(self.rules))
    }

    pub(super) fn broadcast_lobby(
        &self,
        origin: Option<(ConnectionId, RequestId)>,
    ) -> Vec<Delivery> {
        self.room.broadcast_lobby(
            GameKind::TexasHoldem,
            GameRules::TexasHoldem(self.rules),
            origin,
        )
    }

    pub(super) fn broadcast_game(
        &self,
        origin: Option<(ConnectionId, RequestId)>,
    ) -> Vec<Delivery> {
        assert!(self.game.is_some(), "game broadcast requires a game");
        self.room
            .players
            .iter()
            .filter(|player| player.connected && !player.left)
            .map(|player| {
                let reply = origin
                    .filter(|(connection, _)| *connection == player.connection)
                    .map(|(_, request)| request);
                let snapshot = self.game_snapshot(player.id);
                self.room.delivery(
                    player.connection,
                    reply,
                    ServerEvent::GameSnapshot(GameSnapshot::TexasHoldem(snapshot)),
                )
            })
            .collect()
    }

    pub(super) fn game_snapshot(&self, recipient: PlayerId) -> TexasHoldemSnapshot {
        let mut snapshot = self
            .game
            .as_ref()
            .expect("a game snapshot requires an active game")
            .snapshot(recipient)
            .expect("an active room participant belongs to the adapter");
        for state in &mut snapshot.players {
            if let Some(participant) = self
                .room
                .players
                .iter()
                .find(|player| player.id == state.id)
            {
                state.connected =
                    (participant.connected || participant.is_bot) && !participant.left;
                state.auto_play = participant.auto_play;
                state.ready = participant.ready;
                state.reference_points = participant.reference_points;
                state.completed_games = participant.completed_games;
                state.game_profiles.clone_from(&participant.game_profiles);
            }
        }
        if let TexasHoldemPhaseView::HandComplete {
            tournament_complete,
            reference_changes,
            ..
        } = &mut snapshot.phase
        {
            *tournament_complete = self.tournament_complete();
            *reference_changes = self.finished_reference_changes.clone().unwrap_or_default();
        }
        snapshot
    }

    pub(super) fn broadcast_events(&self, events: Vec<TexasHoldemEvent>) -> Vec<Delivery> {
        events
            .into_iter()
            .flat_map(|event| {
                self.room
                    .players
                    .iter()
                    .filter(|player| player.connected && !player.left)
                    .map(move |player| {
                        self.room.delivery(
                            player.connection,
                            None,
                            ServerEvent::GameEvent(GameEvent::TexasHoldem(event.clone())),
                        )
                    })
            })
            .collect()
    }
}
