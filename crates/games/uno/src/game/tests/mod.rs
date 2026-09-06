use super::*;
use crate::card::{CardSide, build_deck, build_flip_deck, build_no_mercy_deck};

mod extensions;
mod flip;
mod jump_in;
mod turn;

fn no_mercy_game(player_count: u8) -> GameState {
    let rules = UnoRuleSet {
        mode: crate::Mode::NoMercy,
        ..UnoRuleSet::default()
    };
    GameState::new_with_deck(rules, player_count, build_no_mercy_deck()).unwrap()
}

fn flip_rules() -> UnoRuleSet {
    UnoRuleSet {
        mode: crate::Mode::Flip,
        ..UnoRuleSet::default()
    }
}

fn flip_card(color: UnoColor, face: UnoFace) -> UnoCard {
    UnoCard::paired(
        CardSide::colored(color, face),
        CardSide::colored(UnoColor::Pink, UnoFace::Number(1)),
        0,
    )
}

fn deck_with_prefix(prefix: &[UnoCard]) -> Vec<UnoCard> {
    let mut deck = build_deck();
    for card in prefix.iter().rev() {
        let index = deck.iter().position(|candidate| candidate == card).unwrap();
        deck.remove(index);
        deck.insert(0, *card);
    }
    deck
}

fn card(color: UnoColor, face: UnoFace, copy: u8) -> UnoCard {
    match face {
        UnoFace::Number(value) => UnoCard::number(color, value, copy),
        UnoFace::DrawTwo
        | UnoFace::DrawOne
        | UnoFace::DrawFour
        | UnoFace::DrawFive
        | UnoFace::Reverse
        | UnoFace::Skip
        | UnoFace::SkipEveryone
        | UnoFace::Flip
        | UnoFace::DiscardAll
        | UnoFace::SwapOne
        | UnoFace::RefreshHand
        | UnoFace::ReverseDrawTwo
        | UnoFace::ReverseSkip
        | UnoFace::StackOne
        | UnoFace::StackTwo => UnoCard::action(color, face, copy),
        UnoFace::Wild
        | UnoFace::DarkWild
        | UnoFace::WildDrawTwo
        | UnoFace::WildDrawFour
        | UnoFace::WildDrawColor
        | UnoFace::WildForceTrade
        | UnoFace::WildPassHands
        | UnoFace::WildPowerReverse
        | UnoFace::WildNoU
        | UnoFace::WildStackThree
        | UnoFace::WildStackNumber
        | UnoFace::WildReverseDrawFour
        | UnoFace::WildDrawSix
        | UnoFace::WildDrawTen
        | UnoFace::WildColorRoulette => UnoCard::wild(face, copy),
    }
}
