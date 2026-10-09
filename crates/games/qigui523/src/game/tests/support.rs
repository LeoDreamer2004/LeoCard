pub(super) use crate::game::TrickState;
use crate::game::{PlayerState, StartingCard};
pub(super) use crate::{
    GameError, GameState, Phase, QiGuiActionContext, QiGuiActionStatistics, QiGuiCard,
    QiGuiMatchStatistics, QiGuiPlayerId, QiGuiRank, QiGuiRuleSet, QiGuiSuit, SameCardPolicy,
    classify,
};
use std::collections::VecDeque;

pub(super) fn card(deck: u8, rank: QiGuiRank) -> QiGuiCard {
    QiGuiCard::suited(deck, QiGuiSuit::Spade, rank)
}

pub(super) fn bomb(deck: u8, rank: QiGuiRank) -> Vec<QiGuiCard> {
    QiGuiSuit::IN_STRENGTH_ORDER
        .into_iter()
        .map(|suit| QiGuiCard::suited(deck, suit, rank))
        .collect()
}

pub(super) fn heaven(deck: u8, suit: QiGuiSuit) -> Vec<QiGuiCard> {
    [
        QiGuiRank::Three,
        QiGuiRank::Two,
        QiGuiRank::Five,
        QiGuiRank::Joker,
        QiGuiRank::Seven,
    ]
    .map(|rank| QiGuiCard::suited(deck, suit, rank))
    .to_vec()
}

pub(super) fn straight(deck: u8) -> Vec<QiGuiCard> {
    [
        QiGuiRank::Four,
        QiGuiRank::Six,
        QiGuiRank::Eight,
        QiGuiRank::Nine,
        QiGuiRank::Ten,
    ]
    .map(|rank| card(deck, rank))
    .to_vec()
}

pub(super) struct Fixture {
    pub(super) game: GameState,
    pub(super) statistics: QiGuiMatchStatistics,
}

impl Fixture {
    pub(super) fn new(hands: Vec<Vec<QiGuiCard>>, pile: Vec<QiGuiCard>, hand_size: u8) -> Self {
        let game = GameState {
            rules: QiGuiRuleSet {
                deck_count: 8,
                player_count: hands.len() as u8,
                hand_size,
                developer_deck: false,
                ..Default::default()
            },
            starting_card: StartingCard {
                player: QiGuiPlayerId(0),
                card: hands[0][0],
            },
            players: hands
                .into_iter()
                .enumerate()
                .map(|(index, hand)| PlayerState {
                    id: QiGuiPlayerId(index),
                    hand,
                    score: 0,
                })
                .collect(),
            played_cards: Vec::new(),
            draw_pile: VecDeque::from(pile),
            trick: Some(TrickState::new(QiGuiPlayerId(0))),
            phase: Phase::Playing,
        };
        let statistics = QiGuiMatchStatistics::new(&game);
        Self { game, statistics }
    }

    pub(super) fn action(
        &mut self,
        player: usize,
        cards: Option<&[QiGuiCard]>,
    ) -> Result<Vec<QiGuiActionStatistics>, GameError> {
        let actor = QiGuiPlayerId(player);
        let before = QiGuiActionContext::capture(&self.game, actor).unwrap();
        let outcome = match cards {
            Some(cards) => self.game.play_cards(actor, cards)?,
            None => self.game.pass(actor)?,
        };
        let play = cards.map(|cards| classify(cards, self.game.rules()).unwrap());
        Ok(self
            .statistics
            .observe(before, &outcome, play.as_ref(), &self.game))
    }

    pub(super) fn play(
        &mut self,
        player: usize,
        cards: &[QiGuiCard],
    ) -> Vec<QiGuiActionStatistics> {
        self.action(player, Some(cards)).unwrap()
    }

    pub(super) fn pass(&mut self, player: usize) -> Vec<QiGuiActionStatistics> {
        self.action(player, None).unwrap()
    }
}
