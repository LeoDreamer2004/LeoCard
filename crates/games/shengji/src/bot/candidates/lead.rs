use super::super::observation::category_index;
use super::groups;
use crate::play::{category, classify_cards};
use crate::{ShengjiCard, ShengjiClassifiedPlay, ShengjiTrump};
use std::collections::BTreeMap;

pub(in super::super) fn units(
    hand: &[ShengjiCard],
    trump: ShengjiTrump,
) -> Vec<ShengjiClassifiedPlay> {
    let faces = groups(hand);
    let mut choices = Vec::new();
    for cards in &faces {
        for size in 1..=cards.len() {
            choices.push(classify_cards(&cards[..size], trump).expect("same face"));
        }
    }
    for width in 2..=4 {
        let mut levels = BTreeMap::<_, Vec<&[ShengjiCard]>>::new();
        for cards in faces.iter().filter(|cards| cards.len() >= width) {
            levels
                .entry((
                    category_index(category(cards[0], trump)),
                    trump.strength(cards[0]),
                ))
                .or_default()
                .push(&cards[..width]);
        }
        for (&(cat, start), firsts) in &levels {
            for first in firsts {
                let mut run = first.to_vec();
                for strength in start + 1..=17 {
                    let Some(next) = levels.get(&(cat, strength)) else {
                        break;
                    };
                    run.extend_from_slice(next[0]);
                    if let Ok(play) = classify_cards(&run, trump)
                        && !play.is_throw()
                    {
                        choices.push(play);
                    }
                }
            }
        }
    }
    choices
}
