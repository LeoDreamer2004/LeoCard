use super::DoudizhuSession;
use leocard_protocol::{DoudizhuSnapshot, PlayerId, PlayerPublicState};

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
            .map(|p| PlayerPublicState {
                id: p.id,
                profile_id: p.profile_id,
                name: p.name.clone(),
                avatar: p.avatar,
                seat: p.seat.expect("started player has a seat"),
                hand_len: lengths[usize::from(p.id.0)],
                score: 0,
                ready: p.ready,
                connected: (p.connected || p.is_bot) && !p.left,
                auto_play: p.auto_play || p.is_bot,
                reference_points: p.reference_points,
                completed_games: p.completed_games,
                game_profiles: p.game_profiles.clone(),
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
            current: PlayerId(game.current),
            landlord: game.landlord.map(PlayerId),
            highest_bid: game.highest_bid,
            bids: game.bids.clone(),
            your_hand: game.hand(you.0).unwrap_or_default().to_vec(),
            kitty: game.public_kitty().to_vec(),
            played: game.played.clone(),
            table: game.table.clone().map(|(p, play)| (PlayerId(p), play)),
            last_actions: game.last_actions.clone(),
            multiplier: game.multiplier,
            result: game.result.clone(),
            revealed_hands: game.revealed_hands(),
        }
    }
}
