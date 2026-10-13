use leocard_qigui523::{
    GameState, PlayRecord, QiGui523Bot, QiGui523BotRequest, QiGuiCard, QiGuiPlayerId,
};

pub(super) struct Policy;

impl Policy {
    pub fn choose(game: &GameState, player: QiGuiPlayerId) -> Option<Vec<QiGuiCard>> {
        let trick = game.trick().expect("a playing game has a trick");
        if let Some(current_play) = trick.winning_play() {
            let played_cards = trick
                .records()
                .iter()
                .flat_map(|record| match record {
                    PlayRecord::Played { play, .. } => play.cards(),
                    PlayRecord::Passed { .. } => &[],
                })
                .copied()
                .collect::<Vec<_>>();
            QiGui523Bot::new()
                .choose(QiGui523BotRequest {
                    hand: game.player(player).expect("current player exists").hand(),
                    current_play,
                    played_cards: &played_cards,
                    rules: game.rules(),
                })
                .map(|play| play.cards().to_vec())
        } else {
            game.player(player)
                .expect("current player exists")
                .hand()
                .iter()
                .copied()
                .min_by_key(|card| (card.rank().strength(), card.suit().strength(), card.deck()))
                .map(|card| vec![card])
        }
    }
}
