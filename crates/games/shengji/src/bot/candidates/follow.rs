use super::super::hand::hand_value;
use super::{combinations, groups, unique, units};
use crate::play::category;
use crate::{
    ShengjiCard, ShengjiClassifiedPlay, ShengjiGreedyBot, ShengjiGreedyBotRequest, ShengjiTrump,
    follow_suggestions, validate_follow,
};

pub(super) fn responses(
    hand: &[ShengjiCard],
    lead: &ShengjiClassifiedPlay,
    trump: ShengjiTrump,
    limit: usize,
) -> Vec<ShengjiClassifiedPlay> {
    let required = lead.cards.len();
    let same: Vec<_> = hand
        .iter()
        .copied()
        .filter(|card| category(*card, trump) == lead.category)
        .collect();
    let (fixed, pool) = if same.len() < required {
        (
            same,
            hand.iter()
                .copied()
                .filter(|card| category(*card, trump) != lead.category)
                .collect(),
        )
    } else {
        (Vec::new(), same)
    };
    let choose = required - fixed.len();
    let mut choices = Vec::new();
    let complete = combinations(&groups(&pool), choose, limit.max(32) * 4, &mut |cards| {
        let mut cards = cards.to_vec();
        cards.extend_from_slice(&fixed);
        if let Ok(play) = validate_follow(hand, &cards, lead, trump) {
            choices.push(play);
        }
    });
    if !complete {
        // Make winning shapes and both extremes of point feeding available even
        // when the full discard space is too large to enumerate.
        for play in units(hand, trump)
            .into_iter()
            .filter(|play| play.cards.len() == required)
        {
            if let Ok(play) = validate_follow(hand, &play.cards, lead, trump) {
                choices.push(play);
            }
        }
        for order in 0..4 {
            let mut sorted = pool.clone();
            sorted.sort_by_key(|card| match order {
                0 => (card.points(), u16::from(trump.strength(*card))),
                1 => (u16::MAX - card.points(), u16::from(trump.strength(*card))),
                2 => (u16::from(trump.strength(*card)), card.points()),
                _ => (u16::MAX - u16::from(trump.strength(*card)), card.points()),
            });
            let mut cards = fixed.clone();
            cards.extend_from_slice(&sorted[..choose]);
            if let Ok(play) = validate_follow(hand, &cards, lead, trump) {
                choices.push(play);
            }
        }
        choices.extend(follow_suggestions(hand, lead, trump, 4));
        improve_fills(hand, lead, trump, &mut choices);
    }
    if choices.is_empty() {
        choices.push(
            ShengjiGreedyBot::choose(ShengjiGreedyBotRequest {
                hand,
                lead: Some(lead),
                trump,
            })
            .expect("legal follow"),
        );
    }
    unique(&mut choices);
    choices
}

fn improve_fills(
    hand: &[ShengjiCard],
    lead: &ShengjiClassifiedPlay,
    trump: ShengjiTrump,
    choices: &mut Vec<ShengjiClassifiedPlay>,
) {
    let seeds: Vec<_> = choices.iter().take(2).cloned().collect();
    for seed in seeds {
        for feed in [false, true] {
            let mut best = seed.clone();
            let value = |play: &ShengjiClassifiedPlay| {
                let kept: Vec<_> = hand
                    .iter()
                    .copied()
                    .filter(|card| !play.cards.contains(card))
                    .collect();
                hand_value(&kept, trump)
                    + play
                        .cards
                        .iter()
                        .map(|card| f64::from(card.points()))
                        .sum::<f64>()
                        * if feed { 3.0 } else { -3.0 }
            };
            let mut score = value(&best);
            for slot in 0..best.cards.len() {
                let mut cards = best.cards.clone();
                for &card in hand {
                    if best.cards.contains(&card) {
                        continue;
                    }
                    cards[slot] = card;
                    if let Ok(play) = validate_follow(hand, &cards, lead, trump) {
                        let next = value(&play);
                        if next > score {
                            score = next;
                            best = play;
                        }
                    }
                    cards.clone_from(&best.cards);
                }
            }
            choices.push(best);
        }
    }
}
