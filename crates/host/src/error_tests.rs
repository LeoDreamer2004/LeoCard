use crate::{GameSetup, HostError, HostSession};
use leocard_blackjack::{BlackjackGameError, BlackjackRuleSet};
use leocard_doudizhu::{DoudizhuGameError, DoudizhuRuleSet};
use leocard_mahjong::{MahjongGameError, MahjongRuleError, MahjongRuleSet};
use leocard_mc_guess::McGuessGameError;
use leocard_protocol::{GameKind, RoomId};
use leocard_qigui523::{QiGuiGameError, QiGuiRuleError, QiGuiRuleSet};
use leocard_shengji::{ShengjiGameError, ShengjiRuleError, ShengjiRuleSet};
use leocard_texas_holdem::{TexasGameError, TexasRuleError, TexasRuleSet};
use leocard_uno::{UnoGameError, UnoRuleSet};
use std::error::Error;

#[test]
fn every_game_error_retains_its_source_and_display() {
    let errors: [HostError; 8] = [
        DoudizhuGameError::InvalidDeck.into(),
        BlackjackGameError::InvalidRules.into(),
        MahjongGameError::WrongPhase.into(),
        McGuessGameError::GraphUnavailable.into(),
        QiGuiGameError::InvalidDeckContents.into(),
        ShengjiGameError::InvalidDeckContents.into(),
        TexasGameError::InvalidDeckContents.into(),
        UnoGameError::InvalidDeckContents.into(),
    ];
    for error in errors {
        assert_eq!(error.to_string(), error.source().unwrap().to_string());
    }
}

#[test]
fn session_initialization_keeps_rule_errors_in_their_game() {
    let cases = [
        (
            GameSetup::Blackjack {
                host_port: 52300,
                rules: BlackjackRuleSet {
                    rounds: 0,
                    ..Default::default()
                },
                shuffled_shoe: Vec::new(),
            },
            HostError::Blackjack(BlackjackGameError::InvalidRules),
        ),
        (
            GameSetup::Mahjong {
                host_port: 52300,
                rules: MahjongRuleSet {
                    false_win: true,
                    minimum_eight_points: false,
                    ..Default::default()
                },
                shuffled_deck: Vec::new(),
            },
            HostError::Mahjong(MahjongGameError::InvalidRules(
                MahjongRuleError::FalseWinRequiresMinimum,
            )),
        ),
        (
            GameSetup::QiGui523 {
                host_port: 52300,
                rules: QiGuiRuleSet {
                    deck_count: 0,
                    ..Default::default()
                },
                shuffled_deck: Vec::new(),
            },
            HostError::QiGui523(QiGuiGameError::InvalidRules(QiGuiRuleError::DeckCount(0))),
        ),
        (
            GameSetup::Shengji {
                host_port: 52300,
                rules: ShengjiRuleSet {
                    deck_count: 0,
                    ..Default::default()
                },
                shuffled_deck: Vec::new(),
            },
            HostError::Shengji(ShengjiGameError::Rules(ShengjiRuleError::InvalidDeckCount(
                0,
            ))),
        ),
        (
            GameSetup::Texas {
                host_port: 52300,
                rules: TexasRuleSet {
                    starting_chips: 1,
                    ..Default::default()
                },
                shuffled_deck: Vec::new(),
            },
            HostError::Texas(TexasGameError::InvalidRules(TexasRuleError::StartingChips(
                1,
            ))),
        ),
    ];
    for (setup, expected) in cases {
        assert_eq!(HostSession::new(RoomId(1), setup).unwrap_err(), expected);
    }
}

#[test]
fn session_initialization_keeps_bad_decks_in_their_game() {
    let rules = [
        DoudizhuRuleSet::default().into(),
        BlackjackRuleSet::default().into(),
        MahjongRuleSet::default().into(),
        QiGuiRuleSet::default().into(),
        ShengjiRuleSet::default().into(),
        TexasRuleSet::default().into(),
        UnoRuleSet::default().into(),
    ];
    for rules in rules {
        let mut setup = GameSetup::shuffled(52300, rules);
        match &mut setup {
            GameSetup::Doudizhu { shuffled_deck, .. } => shuffled_deck.clear(),
            GameSetup::Blackjack { shuffled_shoe, .. } => shuffled_shoe.clear(),
            GameSetup::Mahjong { shuffled_deck, .. } => shuffled_deck.clear(),
            GameSetup::QiGui523 { shuffled_deck, .. } => shuffled_deck.clear(),
            GameSetup::Shengji { shuffled_deck, .. } => shuffled_deck.clear(),
            GameSetup::Texas { shuffled_deck, .. } => shuffled_deck.clear(),
            GameSetup::Uno { shuffled_deck, .. } => shuffled_deck.clear(),
            GameSetup::McGuess { .. } => unreachable!(),
        }
        let kind = setup.kind();
        let error = HostSession::new(RoomId(1), setup).unwrap_err();
        assert!(matches!(
            (kind, error),
            (
                GameKind::Doudizhu,
                HostError::Doudizhu(DoudizhuGameError::InvalidDeck)
            ) | (
                GameKind::Blackjack,
                HostError::Blackjack(BlackjackGameError::InvalidShoe)
            ) | (
                GameKind::Mahjong,
                HostError::Mahjong(MahjongGameError::InvalidDeckSize { actual: 0, .. })
            ) | (
                GameKind::QiGui523,
                HostError::QiGui523(QiGuiGameError::InvalidDeckSize { actual: 0, .. })
            ) | (
                GameKind::Shengji,
                HostError::Shengji(ShengjiGameError::InvalidDeckSize { actual: 0, .. })
            ) | (
                GameKind::Texas,
                HostError::Texas(TexasGameError::InvalidDeckSize { actual: 0, .. })
            ) | (
                GameKind::Uno,
                HostError::Uno(UnoGameError::InvalidDeckSize { actual: 0, .. })
            )
        ));
    }
}
