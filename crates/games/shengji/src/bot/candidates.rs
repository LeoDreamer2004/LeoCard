use super::{ShengjiGreedyBot, ShengjiGreedyBotRequest};
use crate::play::{category, classify_cards};
use crate::{
    ShengjiCard, ShengjiCategory, ShengjiClassifiedPlay, ShengjiTrump, follow_suggestions,
};
use std::collections::BTreeMap;

pub(super) fn plays(
    hand: &[ShengjiCard],
    lead: Option<&ShengjiClassifiedPlay>,
    trump: ShengjiTrump,
    limit: usize,
) -> Vec<ShengjiClassifiedPlay> {
    if let Some(lead) = lead {
        let mut choices = follow_suggestions(hand, lead, trump, limit.max(16));
        if choices.is_empty()
            && let Ok(play) = ShengjiGreedyBot::choose(ShengjiGreedyBotRequest {
                hand,
                lead: Some(lead),
                trump,
            })
        {
            choices.push(play);
        }
        return choices;
    }
    let mut faces = BTreeMap::new();
    for card in hand {
        faces
            .entry((card.suit(), card.rank()))
            .or_insert_with(Vec::new)
            .push(*card);
    }
    let mut choices = Vec::new();
    for cards in faces.values() {
        for size in 1..=cards.len() {
            if let Ok(play) = classify_cards(&cards[..size], trump) {
                choices.push(play);
            }
        }
    }
    for width in 2..=4 {
        let mut groups: BTreeMap<(usize, u8), Vec<ShengjiCard>> = BTreeMap::new();
        for cards in faces.values().filter(|cards| cards.len() >= width) {
            let cat = category(cards[0], trump);
            let index = match cat {
                ShengjiCategory::Trump => 4,
                ShengjiCategory::Suit(suit) => suit as usize,
                ShengjiCategory::Mixed => continue,
            };
            groups.insert((index, trump.strength(cards[0])), cards[..width].to_vec());
        }
        for (&(cat, start), cards) in &groups {
            let mut run = cards.clone();
            for offset in 1..=5 {
                let Some(next) = groups.get(&(cat, start + offset)) else {
                    break;
                };
                run.extend(next);
                if let Ok(play) = classify_cards(&run, trump)
                    && !play.is_throw()
                {
                    choices.push(play);
                }
            }
        }
    }
    choices
}

pub(super) fn hand_value(hand: &[ShengjiCard], trump: ShengjiTrump) -> f64 {
    let mut faces = BTreeMap::new();
    let mut suits = [0; 5];
    let mut value = 0.0;
    for card in hand {
        let strength = f64::from(trump.strength(*card));
        value += if trump.is_trump(*card) {
            2.0 + strength * 0.16
        } else {
            (strength - 8.0).max(0.0) * 0.6
        };
        *faces.entry((card.suit(), card.rank())).or_insert(0usize) += 1;
        let cat = category(*card, trump);
        suits[match cat {
            ShengjiCategory::Trump => 4,
            ShengjiCategory::Suit(suit) => suit as usize,
            ShengjiCategory::Mixed => unreachable!(),
        }] += 1;
    }
    for (face, count) in faces {
        if count >= 2 {
            let card = hand
                .iter()
                .find(|card| (card.suit(), card.rank()) == face)
                .expect("face");
            value += (count - 1) as f64 * (2.5 + f64::from(trump.strength(*card)) * 0.08);
        }
    }
    for length in &suits[..4] {
        value += match length {
            0 => 4.0,
            1 => 1.5,
            _ => 0.0,
        };
    }
    value
}
