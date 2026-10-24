use crate::{GameSetup, HostError, HostSession};
use leocard_blackjack::{BlackjackRuleSet, GameError as BlackjackError};
use leocard_mahjong::{GameError as MahjongError, MahjongRuleSet, RuleError as MahjongRuleError};
use leocard_minecraft_guess::GameError as MinecraftGuessError;
use leocard_protocol::{GameKind, RoomId};
use leocard_qigui523::{GameError as QiGui523Error, QiGuiRuleSet, RuleError as QiGui523RuleError};
use leocard_shengji::{GameError as ShengjiError, RuleError as ShengjiRuleError, ShengjiRuleSet};
use leocard_texas_holdem::{
    GameError as TexasHoldemError, RuleError as TexasHoldemRuleError, TexasHoldemRuleSet,
};
use leocard_uno::{GameError as UnoError, UnoRuleSet};
use std::error::Error;

#[test]
fn every_game_error_retains_its_source_and_display() {
    let errors: [HostError; 7] = [
        BlackjackError::InvalidRules.into(),
        MahjongError::WrongPhase.into(),
        MinecraftGuessError::GraphUnavailable.into(),
        QiGui523Error::InvalidDeckContents.into(),
        ShengjiError::InvalidDeckContents.into(),
        TexasHoldemError::InvalidDeckContents.into(),
        UnoError::InvalidDeckContents.into(),
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
            HostError::Blackjack(BlackjackError::InvalidRules),
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
            HostError::Mahjong(MahjongError::InvalidRules(
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
            HostError::QiGui523(QiGui523Error::InvalidRules(QiGui523RuleError::DeckCount(0))),
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
            HostError::Shengji(ShengjiError::Rules(ShengjiRuleError::InvalidDeckCount(0))),
        ),
        (
            GameSetup::TexasHoldem {
                host_port: 52300,
                rules: TexasHoldemRuleSet {
                    starting_chips: 1,
                    ..Default::default()
                },
                shuffled_deck: Vec::new(),
            },
            HostError::TexasHoldem(TexasHoldemError::InvalidRules(
                TexasHoldemRuleError::StartingChips(1),
            )),
        ),
    ];
    for (setup, expected) in cases {
        assert_eq!(HostSession::new(RoomId(1), setup).unwrap_err(), expected);
    }
}

#[test]
fn session_initialization_keeps_bad_decks_in_their_game() {
    let rules = [
        BlackjackRuleSet::default().into(),
        MahjongRuleSet::default().into(),
        QiGuiRuleSet::default().into(),
        ShengjiRuleSet::default().into(),
        TexasHoldemRuleSet::default().into(),
        UnoRuleSet::default().into(),
    ];
    for rules in rules {
        let mut setup = GameSetup::shuffled(52300, rules);
        match &mut setup {
            GameSetup::Blackjack { shuffled_shoe, .. } => shuffled_shoe.clear(),
            GameSetup::Mahjong { shuffled_deck, .. } => shuffled_deck.clear(),
            GameSetup::QiGui523 { shuffled_deck, .. } => shuffled_deck.clear(),
            GameSetup::Shengji { shuffled_deck, .. } => shuffled_deck.clear(),
            GameSetup::TexasHoldem { shuffled_deck, .. } => shuffled_deck.clear(),
            GameSetup::Uno { shuffled_deck, .. } => shuffled_deck.clear(),
            GameSetup::MinecraftGuess { .. } => unreachable!(),
        }
        let kind = setup.kind();
        let error = HostSession::new(RoomId(1), setup).unwrap_err();
        assert!(matches!(
            (kind, error),
            (
                GameKind::Blackjack,
                HostError::Blackjack(BlackjackError::InvalidShoe)
            ) | (
                GameKind::Mahjong,
                HostError::Mahjong(MahjongError::InvalidDeckSize { actual: 0, .. })
            ) | (
                GameKind::QiGui523,
                HostError::QiGui523(QiGui523Error::InvalidDeckSize { actual: 0, .. })
            ) | (
                GameKind::Shengji,
                HostError::Shengji(ShengjiError::InvalidDeckSize { actual: 0, .. })
            ) | (
                GameKind::TexasHoldem,
                HostError::TexasHoldem(TexasHoldemError::InvalidDeckSize { actual: 0, .. })
            ) | (
                GameKind::Uno,
                HostError::Uno(UnoError::InvalidDeckSize { actual: 0, .. })
            )
        ));
    }
}
