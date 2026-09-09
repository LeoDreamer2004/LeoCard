use super::{UnoCard, UnoColor, UnoFace, build_flip_deck};
use crate::UnoRuleSet;

pub fn build_deck() -> Vec<UnoCard> {
    let mut cards = Vec::with_capacity(108);
    for color in UnoColor::LIGHT {
        cards.push(UnoCard::number(color, 0, 0));
        for value in 1..=9 {
            cards.push(UnoCard::number(color, value, 0));
            cards.push(UnoCard::number(color, value, 1));
        }
        for face in [UnoFace::DrawTwo, UnoFace::Reverse, UnoFace::Skip] {
            cards.push(UnoCard::action(color, face, 0));
            cards.push(UnoCard::action(color, face, 1));
        }
    }
    for copy in 0..4 {
        cards.push(UnoCard::wild(UnoFace::Wild, copy));
        cards.push(UnoCard::wild(UnoFace::WildDrawFour, copy));
    }
    cards
}

pub fn build_swap_pack() -> Vec<UnoCard> {
    let mut cards = Vec::with_capacity(16);
    for color in UnoColor::LIGHT {
        cards.push(UnoCard::action(color, UnoFace::SwapOne, 0));
        cards.push(UnoCard::action(color, UnoFace::RefreshHand, 0));
    }
    for copy in 0..4 {
        cards.push(UnoCard::wild(UnoFace::WildForceTrade, copy));
        cards.push(UnoCard::wild(UnoFace::WildPassHands, copy));
    }
    cards
}

pub fn build_reverse_pack() -> Vec<UnoCard> {
    let mut cards = Vec::with_capacity(16);
    for color in UnoColor::LIGHT {
        cards.push(UnoCard::action(color, UnoFace::ReverseDrawTwo, 0));
        cards.push(UnoCard::action(color, UnoFace::ReverseSkip, 0));
    }
    for copy in 0..4 {
        cards.push(UnoCard::wild(UnoFace::WildPowerReverse, copy));
        cards.push(UnoCard::wild(UnoFace::WildNoU, copy));
    }
    cards
}

pub fn build_stack_pack() -> Vec<UnoCard> {
    let mut cards = Vec::with_capacity(16);
    for color in UnoColor::LIGHT {
        cards.push(UnoCard::action(color, UnoFace::StackOne, 0));
        cards.push(UnoCard::action(color, UnoFace::StackTwo, 0));
    }
    for copy in 0..4 {
        cards.push(UnoCard::wild(UnoFace::WildStackThree, copy));
        cards.push(UnoCard::wild(UnoFace::WildStackNumber, copy));
    }
    cards
}

pub fn build_no_mercy_deck() -> Vec<UnoCard> {
    let mut cards = Vec::with_capacity(168);
    for color in UnoColor::LIGHT {
        for value in 0..=9 {
            cards.push(UnoCard::number(color, value, 0));
            cards.push(UnoCard::number(color, value, 1));
        }
        for face in [
            UnoFace::DrawTwo,
            UnoFace::Reverse,
            UnoFace::Skip,
            UnoFace::DiscardAll,
        ] {
            for copy in 0..3 {
                cards.push(UnoCard::action(color, face, copy));
            }
        }
        for face in [UnoFace::DrawFour, UnoFace::SkipEveryone] {
            for copy in 0..2 {
                cards.push(UnoCard::action(color, face, copy));
            }
        }
    }
    for copy in 0..8 {
        cards.push(UnoCard::wild(UnoFace::WildReverseDrawFour, copy));
        cards.push(UnoCard::wild(UnoFace::WildColorRoulette, copy));
    }
    for copy in 0..4 {
        cards.push(UnoCard::wild(UnoFace::WildDrawSix, copy));
        cards.push(UnoCard::wild(UnoFace::WildDrawTen, copy));
    }
    cards
}

pub fn build_deck_for_rules(rules: UnoRuleSet) -> Vec<UnoCard> {
    if rules.is_no_mercy() {
        return build_no_mercy_deck();
    }
    if rules.is_flip() {
        return build_flip_deck();
    }
    let mut cards = build_deck();
    if rules.swap_pack {
        cards.extend(build_swap_pack());
    }
    if rules.reverse_pack {
        cards.extend(build_reverse_pack());
    }
    if rules.stack_pack {
        cards.extend(build_stack_pack());
    }
    cards
}
