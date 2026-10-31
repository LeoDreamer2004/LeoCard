use crate::play::category;
use crate::{ShengjiCard, ShengjiRank, ShengjiTrump};

pub(super) const FACES: usize = 54;

pub(super) fn face(card: ShengjiCard) -> usize {
    card.suit().map_or_else(
        || {
            if card.rank() == ShengjiRank::SmallJoker {
                52
            } else {
                53
            }
        },
        |suit| suit as usize * 13 + card.rank() as usize,
    )
}

pub(super) fn counts(cards: &[ShengjiCard]) -> [u8; FACES] {
    let mut counts = [0; FACES];
    for &card in cards {
        counts[face(card)] += 1;
    }
    counts
}

pub(super) fn category_index(card: ShengjiCard, trump: ShengjiTrump) -> usize {
    super::observation::category_index(category(card, trump))
}

/// Values structure separately from rank: a connected pair is useful even when
/// its individual cards are small. All sizes are relative to the current hand.
pub(super) fn hand_value(hand: &[ShengjiCard], trump: ShengjiTrump) -> f64 {
    let mut lengths = [0; 5];
    for &card in hand {
        lengths[category_index(card, trump)] += 1;
    }
    material_value(hand, trump)
        + lengths[..4]
            .iter()
            .map(|&length| match length {
                0 if lengths[4] > 0 => 4.5,
                1 => 1.0,
                _ => 0.0,
            })
            .sum::<f64>()
}

/// The strength and combinations actually committed to a trick, excluding
/// void-suit bonuses that only make sense for the remaining hand.
pub(super) fn material_value(hand: &[ShengjiCard], trump: ShengjiTrump) -> f64 {
    let counts = counts(hand);
    let mut runs = [[[false; 18]; 5]; 3];
    let mut value = 0.0;
    for &card in hand {
        let strength = f64::from(trump.strength(card));
        let cat = category_index(card, trump);
        value += if cat == 4 {
            1.8 + strength * 0.24
        } else {
            (strength - 7.0).max(0.0) * 0.8
        };
        let copies = counts[face(card)];
        // Only count each face once; physical deck copies are interchangeable.
        if hand
            .iter()
            .any(|other| other.same_face(card) && other.deck() < card.deck())
        {
            continue;
        }
        for width in 2..=copies.min(4) {
            value += 3.2 + strength * 0.10;
            runs[usize::from(width - 2)][cat][usize::from(trump.strength(card))] = true;
        }
    }
    for (width, categories) in runs.iter().enumerate() {
        for strengths in categories {
            value += strengths
                .windows(2)
                .filter(|pair| pair[0] && pair[1])
                .count() as f64
                * (3.0 + width as f64 * 1.8);
        }
    }
    value
}

pub(super) fn trump_value(hand: &[ShengjiCard], trump: ShengjiTrump) -> f64 {
    let trumps: Vec<_> = hand
        .iter()
        .copied()
        .filter(|card| trump.is_trump(*card))
        .collect();
    hand_value(&trumps, trump) * 1.5 + hand_value(hand, trump) * 0.3
}
