use super::MahjongLinkSession;
use crate::features::GameSnapshotView;
use leocard_game_common::PlayerId;
use leocard_protocol::{MahjongLinkPlayerView, MahjongLinkSnapshot};

impl GameSnapshotView for MahjongLinkSession {
    type Snapshot = MahjongLinkSnapshot;

    fn game_view(&self, recipient: PlayerId) -> Self::Snapshot {
        let game = self.game.as_ref().expect("active mahjong link game");
        MahjongLinkSnapshot {
            match_id: self.match_id.expect("active match"),
            host_port: self.room.host_port,
            you: recipient,
            host: self.room.host_player_id().expect("room host"),
            rules: self.rules,
            board: game.board().clone(),
            players: self
                .order
                .iter()
                .enumerate()
                .map(|(index, id)| {
                    let participant = self
                        .room
                        .players
                        .iter()
                        .find(|player| player.id == *id)
                        .expect("retained participant");
                    let public = participant.public_metadata();
                    MahjongLinkPlayerView {
                        player: public.player,
                        seat: public.seat.expect("active player seat"),
                        ready: public.ready,
                        connected: public.connected,
                        auto_play: participant.auto_play,
                        score: game.scores()[index],
                    }
                })
                .collect(),
            current_player: game
                .current_player()
                .map(|index| self.order[usize::from(index)]),
            turn: game.turn(),
            push_used: game.push_used(),
            consecutive_passes: game.consecutive_passes(),
            match_complete: game.complete(),
            last_action: self.last_action.clone(),
            reference_changes: Vec::new(),
        }
    }
}
