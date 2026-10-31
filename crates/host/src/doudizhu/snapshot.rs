use super::DoudizhuSession;
use leocard_game_common::PlayerId;
use leocard_protocol::{DoudizhuPlayerView, DoudizhuSnapshot};

impl DoudizhuSession {
    pub(super) fn snapshot_for(&self, you: PlayerId) -> DoudizhuSnapshot {
        let game = self
            .game
            .as_ref()
            .expect("snapshot requires a started game");
        let lengths = game.hand_lengths();
        let state = self.match_state.as_ref().expect("active match");
        let players = self
            .room
            .players
            .iter()
            .map(|participant| {
                let public = participant.public_metadata();
                DoudizhuPlayerView {
                    player: public.player,
                    seat: public.seat.expect("started player has a seat"),
                    hand_len: lengths[participant.id.index()],
                    ready: public.ready,
                    connected: public.connected,
                    auto_play: public.auto_play || participant.is_bot,
                }
            })
            .collect();
        DoudizhuSnapshot {
            match_id: self.match_id.expect("started match"),
            host_port: self.room.host_port,
            you,
            host: self.room.host_player_id().expect("room host"),
            players,
            rules: self.rules,
            progress: state.progress(),
            scores: state.scores().to_vec(),
            match_complete: state.complete(),
            reference_changes: self.finished_reference_changes.clone().unwrap_or_default(),
            phase: game.phase,
            current: game.current,
            landlord: game.landlord,
            highest_bid: game.highest_bid,
            bids: game.bids.clone(),
            your_hand: game.hand(you).unwrap_or_default().to_vec(),
            kitty: game.public_kitty().to_vec(),
            played: game.played.clone(),
            table: game.table.clone(),
            last_actions: game.last_actions.clone(),
            multiplier: game.multiplier,
            result: game.result.clone(),
            revealed_hands: game.revealed_hands(),
        }
    }
}
