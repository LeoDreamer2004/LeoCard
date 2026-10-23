use super::super::QiGui523Session;
use leocard_protocol::QiGui523Event;
use leocard_qigui523::{QiGui523Bot, QiGui523BotObservation};

impl QiGui523Session {
    pub(in super::super) fn play_automatic_action(&mut self) -> Vec<QiGui523Event> {
        let (player, cards) = {
            let game = self.game.as_ref().expect("an automatic action has a game");
            let player = game
                .trick()
                .expect("a playing game has a trick")
                .current_player();
            let observation = QiGui523BotObservation::from_game(game, player)
                .expect("the automatic player owns the current turn");
            let cards = QiGui523Bot::choose_action(&observation);
            (player, cards)
        };
        self.perform_action(player, cards.as_deref())
            .expect("an automatic policy only returns legal actions")
    }
}
