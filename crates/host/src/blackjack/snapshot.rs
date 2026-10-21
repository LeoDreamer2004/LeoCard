use super::BlackjackSession;
use leocard_protocol::{BlackjackPlayerView, BlackjackSnapshot, PlayerId};

impl BlackjackSession {
    pub(super) fn snapshot_for(&self, recipient: PlayerId) -> BlackjackSnapshot {
        let game = self.game.as_ref().expect("active blackjack game");
        let index = self.order.iter().position(|player| *player == recipient);
        BlackjackSnapshot {
            match_id: self.match_id.expect("active match"),
            host_port: self.room.host_port,
            you: recipient,
            host: self.room.host_player_id().expect("room host"),
            rules: self.rules,
            round: game.round(),
            phase: game.phase(),
            players: self
                .order
                .iter()
                .zip(game.players())
                .map(|(id, state)| {
                    let participant = self
                        .room
                        .players
                        .iter()
                        .find(|player| player.id == *id)
                        .expect("retained match participant");
                    BlackjackPlayerView {
                        player: participant.public_metadata().into(),
                        stack: state.stack,
                        round_start_stack: state.round_start_stack,
                        hands: state.hands.clone(),
                        auto_play: participant.auto_play,
                    }
                })
                .collect(),
            dealer: game.visible_dealer().to_vec(),
            dealer_hidden: game.dealer_has_hidden_card(),
            current_player: game.current_player().map(|index| self.order[index]),
            current_hand: game.current_hand().map(|hand| hand as u8),
            available: index
                .map(|index| game.available_actions(index))
                .unwrap_or_default(),
            seconds_remaining: self.seconds_remaining(),
            match_complete: game.match_complete(),
            reference_changes: self.finished_reference_changes.clone().unwrap_or_default(),
        }
    }
}
