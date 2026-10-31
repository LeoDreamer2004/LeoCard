use super::super::evaluation::{Evaluation, PositionView};
use super::super::hand::counts;
use super::super::knowledge::PublicKnowledge;
use crate::play::classify_cards;
use crate::{
    ShengjiBiddingObservation, ShengjiBot, ShengjiBotAction, ShengjiBotObservation, ShengjiCard,
    ShengjiClassifiedPlay, ShengjiGameState, ShengjiMode, ShengjiRank, ShengjiRuleSet, ShengjiSuit,
    ShengjiTeamProgress, ShengjiTrickRecord, ShengjiTrump, build_deck_for,
};
use leocard_game_common::{PlayerId, SearchBudget, SearchRandom};
use std::time::Duration;

pub(super) fn card(suit: ShengjiSuit, rank: ShengjiRank) -> ShengjiCard {
    ShengjiCard::suited(0, suit, rank)
}

pub(super) fn trump() -> ShengjiTrump {
    ShengjiTrump::new(ShengjiRank::Ten, Some(ShengjiSuit::Heart)).unwrap()
}

pub(super) fn play(cards: &[ShengjiCard]) -> ShengjiClassifiedPlay {
    classify_cards(cards, trump()).unwrap()
}

pub(super) struct Table {
    pub hands: Vec<Vec<ShengjiCard>>,
    pub public: PublicKnowledge,
    pub rules: ShengjiRuleSet,
    pub trick: Option<ShengjiTrickRecord>,
    pub trump: ShengjiTrump,
}

impl Table {
    pub fn new(hands: Vec<Vec<ShengjiCard>>) -> Self {
        let rules = ShengjiRuleSet::for_mode(if hands.len() == 6 {
            ShengjiMode::SixPlayer
        } else {
            ShengjiMode::Classic
        });
        let public = PublicKnowledge {
            remaining: counts(&hands.iter().flatten().copied().collect::<Vec<_>>()),
            voids: vec![[false; 5]; hands.len()],
            multiplicity: vec![[rules.deck_count; 5]; hands.len()],
        };
        Self {
            hands,
            public,
            rules,
            trick: None,
            trump: trump(),
        }
    }

    pub fn best(&self, player: u8) -> ShengjiClassifiedPlay {
        let lengths: Vec<_> = self.hands.iter().map(Vec::len).collect();
        Evaluation::new(PositionView {
            player: PlayerId(player),
            hand: &self.hands[usize::from(player)],
            trump: self.trump,
            rules: self.rules,
            trick: self.trick.as_ref(),
            public: &self.public,
            lengths: &lengths,
            collecting_score: 0,
            bottom: None,
        })
        .ranked(16)
        .remove(0)
        .0
    }
}

pub(super) fn game(mode: ShengjiMode, seed: u64) -> ShengjiGameState {
    let rules = ShengjiRuleSet::for_mode(mode);
    let mut deck = build_deck_for(rules.deck_count);
    let mut random = SearchRandom::new(seed);
    for index in (1..deck.len()).rev() {
        deck.swap(index, random.index(index + 1));
    }
    let mut game = ShengjiGameState::new(
        rules,
        ShengjiTeamProgress::for_rules(&rules),
        Some(PlayerId(0)),
        PlayerId(0),
        deck,
    )
    .unwrap();
    game.deal_all().unwrap();
    for player in 0..rules.player_count() {
        let player = PlayerId(player);
        if let Some(cards) =
            ShengjiBot::declaration(&ShengjiBiddingObservation::from_game(&game, player).unwrap())
        {
            game.declare(player, &cards).unwrap();
        }
    }
    game.close_bidding_and_take_kitty().unwrap();
    let player = game.dealer().unwrap();
    let ShengjiBotAction::Bury(cards) = ShengjiBot::choose(
        &ShengjiBotObservation::from_game(&game, player).unwrap(),
        &SearchBudget::new(Duration::ZERO),
    ) else {
        panic!("bury")
    };
    game.bury(player, &cards).unwrap();
    game
}
