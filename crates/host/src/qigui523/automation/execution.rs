use super::super::{QiGui523Session, from_core_player};
use super::{auto_play, bot};
use leocard_protocol::QiGui523Event;

impl QiGui523Session {
    pub(in super::super) fn play_automatic_action(&mut self) -> Vec<QiGui523Event> {
        let (player, cards) = {
            let game = self.game.as_ref().expect("an automatic action has a game");
            let player = game
                .trick()
                .expect("a playing game has a trick")
                .current_player();
            let participant = &self.players[usize::from(from_core_player(player).0)];
            let cards = if participant.is_bot {
                bot::Policy::choose(game, player)
            } else {
                auto_play::Policy::choose(game, player)
            };
            (player, cards)
        };
        self.perform_action(player, cards.as_deref())
            .expect("an automatic policy only returns legal actions")
    }
}
