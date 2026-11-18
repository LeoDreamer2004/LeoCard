use super::DrawGuessSession;
use crate::features::GameSnapshotView;
use leocard_draw_guess::DrawGuessPhase;
use leocard_game_common::PlayerId;
use leocard_protocol::{DrawGuessPlayerView, DrawGuessSnapshot};

impl GameSnapshotView for DrawGuessSession {
    type Snapshot = DrawGuessSnapshot;

    fn game_view(&self, recipient: PlayerId) -> Self::Snapshot {
        let game = self.game.as_ref().expect("active drawing game");
        DrawGuessSnapshot {
            match_id: self.match_id.expect("active match"),
            host_port: self.room.host_port,
            you: recipient,
            host: self.room.host_player_id().expect("room host"),
            rules: self.rules,
            players: self
                .order
                .iter()
                .map(|id| {
                    let participant = self
                        .room
                        .players
                        .iter()
                        .find(|player| player.id == *id)
                        .expect("retained player");
                    let public = participant.public_metadata();
                    DrawGuessPlayerView {
                        player: public.player,
                        seat: public.seat.expect("active seat"),
                        ready: public.ready,
                        connected: public.connected,
                    }
                })
                .collect(),
            state: game.view(recipient),
            match_complete: game.phase() == DrawGuessPhase::Finished,
            reference_changes: self.finished_reference_changes.clone().unwrap_or_default(),
        }
    }
}
