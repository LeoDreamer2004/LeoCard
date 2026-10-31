pub(super) use crate::game::QiGuiTrickState;
use crate::game::{QiGuiPlayerState, QiGuiStartingCard};
pub(super) use crate::{
    QiGuiActionContext, QiGuiActionStatistics, QiGuiCard, QiGuiGameError, QiGuiGameState,
    QiGuiMatchStatistics, QiGuiPhase, QiGuiRank, QiGuiRuleSet, QiGuiSameCardPolicy, QiGuiSuit,
    classify,
};
use leocard_game_common::PlayerId;

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
    pub(super) game: QiGuiGameState,
    pub(super) statistics: QiGuiMatchStatistics,
}

impl Fixture {
    pub(super) fn new(hands: Vec<Vec<QiGuiCard>>, pile: Vec<QiGuiCard>, hand_size: u8) -> Self {
        let game = QiGuiGameState {
            rules: QiGuiRuleSet {
                deck_count: 8,
                player_count: hands.len() as u8,
                hand_size,
                developer_deck: false,
                ..Default::default()
            },
            starting_card: QiGuiStartingCard {
                player: PlayerId(0),
                card: hands[0][0],
            },
            players: hands
                .into_iter()
                .enumerate()
                .map(|(index, hand)| QiGuiPlayerState {
                    id: PlayerId::from_index(index),
                    hand,
                    score: 0,
                })
                .collect(),
            played_cards: Vec::new(),
            draw_pile: VecDeque::from(pile),
            trick: Some(QiGuiTrickState::new(PlayerId(0))),
            phase: QiGuiPhase::Playing,
        };
        let statistics = QiGuiMatchStatistics::new(&game);
        Self { game, statistics }
    }

    pub(super) fn action(
        &mut self,
        player: usize,
        cards: Option<&[QiGuiCard]>,
    ) -> Result<Vec<QiGuiActionStatistics>, QiGuiGameError> {
        let actor = PlayerId::from_index(player);
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
