use super::McGuessSession;
use crate::features::GameSnapshotView;
use leocard_game_common::PlayerId;
use leocard_protocol::{McGuessSnapshot, PlayerScore};

impl GameSnapshotView for McGuessSession {
    type Snapshot = McGuessSnapshot;

    fn game_view(&self, recipient: PlayerId) -> Self::Snapshot {
        let state = self.match_state.as_ref().expect("active match");
        McGuessSnapshot {
            match_id: self.match_id.expect("active match"),
            host_port: self.host_port,
            you: recipient,
            host: self.host_player_id().expect("room host"),
            players: state
                .scores()
                .map(|(id, _)| {
                    self.players
                        .iter()
                        .find(|player| player.id == id)
                        .expect("match participant belongs to its room")
                })
                .filter(|player| !player.left)
                .map(|player| player.public_metadata().into())
                .collect(),
            rules: self.rules,
            progress: state.progress(),
            round: self.game.as_ref().expect("active game").view(),
            graph: self.game.as_ref().expect("active game").graph_view(),
            guess_seconds_remaining: self.guess_seconds_remaining(),
            scores: state
                .scores()
                .map(|(player, score)| PlayerScore { player, score })
                .collect(),
            score_gains: state.gains().to_vec(),
            reference_changes: self.finished_reference_changes.clone().unwrap_or_default(),
            mc_process_players: self.process_presence.players(),
        }
    }
}
