use super::TexasHoldemSession;
use crate::Delivery;
use leocard_protocol::{
    GameEvent, PlayerId, ServerEvent, TexasHoldemEvent, TexasHoldemPhaseView, TexasHoldemSnapshot,
};

impl TexasHoldemSession {
    pub(super) fn game_snapshot(&self, recipient: PlayerId) -> TexasHoldemSnapshot {
        let game = self
            .game
            .as_ref()
            .expect("a game snapshot requires an active game");
        let mut snapshot = if self
            .room
            .players
            .iter()
            .any(|player| player.id == recipient)
        {
            game.snapshot(recipient)
                .expect("an active room participant belongs to the game")
        } else {
            game.spectator_snapshot(recipient)
        };
        for state in &mut snapshot.players {
            if let Some(participant) = self
                .room
                .players
                .iter()
                .find(|player| player.id == state.id)
            {
                let public = participant.public_metadata();
                state.connected = public.connected;
                state.auto_play = public.auto_play;
                state.ready = public.ready;
                state.reference_points = public.reference_points;
                state.completed_games = public.completed_games;
                state.game_profiles = public.game_profiles;
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
        self.room
            .broadcast_game_events(
                events,
                self.game
                    .as_ref()
                    .map(|game| (game.match_id(), Some(game.game().hand_number()))),
            )
            .into_iter()
            .filter(|delivery| match &delivery.message.event {
                ServerEvent::GameEvent(GameEvent::TexasHoldem(
                    TexasHoldemEvent::HandAnalyzed { player, .. },
                )) => self.room.player_id(delivery.recipient) == Some(*player),
                _ => true,
            })
            .collect()
    }
}
