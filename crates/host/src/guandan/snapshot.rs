use super::GuandanSession;
use leocard_game_common::PlayerId;
use leocard_guandan::{GuandanAction, GuandanPhase};
use leocard_protocol::{GuandanPlayerView, GuandanPublicAction, GuandanSnapshot};

impl GuandanSession {
    pub(super) fn snapshot_for(&self, you: PlayerId) -> GuandanSnapshot {
        let game = self
            .game
            .as_ref()
            .expect("snapshot requires a started game");
        let state = self.match_state.as_ref().expect("active match");
        let players = self
            .room
            .players
            .iter()
            .map(|participant| {
                let public = participant.public_metadata();
                GuandanPlayerView {
                    player: public.player,
                    seat: public.seat.expect("started player has a seat"),
                    hand_len: game
                        .player(participant.id)
                        .map_or(0, |player| player.hand().len() as u16),
                    ready: public.ready,
                    connected: public.connected,
                    auto_play: public.auto_play || participant.is_bot,
                }
            })
            .collect();
        GuandanSnapshot {
            match_id: self.match_id.expect("started match"),
            host_port: self.room.host_port,
            you,
            host: self.room.host_player_id().expect("room host"),
            players,
            rules: self.rules,
            level: game.rules().starting_level,
            team_levels: state.levels(),
            hand_number: state.hand_number(),
            match_complete: state.complete(),
            reference_changes: self.finished_reference_changes.clone().unwrap_or_default(),
            phase: game.phase().clone(),
            current: game.current_player(),
            tribute_action: (game.current_player() == Some(you))
                .then(|| game.pending_tribute_action())
                .flatten(),
            received_tribute: game.received_tribute_for(you),
            tribute_pairs: game.tribute_pairs().to_vec(),
            tribute_resisted: game.tribute_resisted(),
            your_hand: game
                .player(you)
                .map_or_else(Vec::new, |player| player.hand().to_vec()),
            played_cards: game.played_cards().to_vec(),
            table: game
                .winning_player()
                .zip(game.winning_play())
                .map(|(player, play)| (player, play.clone())),
            last_actions: public_actions(game.last_actions()),
            previous_trick: game.previous_trick().map(public_actions),
            finishing_order: game.finishing_order().to_vec(),
            level_advance: state.level_advance(),
            result: match game.phase() {
                GuandanPhase::Finished(result) => Some(result.clone()),
                _ => None,
            },
        }
    }
}

fn public_actions(actions: &[Option<GuandanAction>; 4]) -> [Option<GuandanPublicAction>; 4] {
    std::array::from_fn(|index| match &actions[index] {
        Some(GuandanAction::Play(cards)) => Some(GuandanPublicAction::Play(cards.clone())),
        Some(GuandanAction::Pass) => Some(GuandanPublicAction::Pass),
        _ => None,
    })
}
