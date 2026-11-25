use crate::{GuandanPlay, GuandanPlayKind};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::sync::OnceLock;

#[derive(Deserialize)]
struct Action {
    id: usize,
    #[serde(rename = "type")]
    kind: String,
    points: Vec<u8>,
}

static ACTIONS: OnceLock<Vec<Action>> = OnceLock::new();

pub(super) fn action_id(play: &GuandanPlay, level: u8) -> Option<usize> {
    let actions = ACTIONS.get_or_init(|| {
        serde_json::from_str(include_str!("actions.json"))
            .expect("versioned model action vocabulary")
    });
    let rank = |rank| if rank == level { 15 } else { rank };
    let (kind, mut points) = match play.kind {
        GuandanPlayKind::Single => ("single".to_owned(), vec![rank(play.value)]),
        GuandanPlayKind::Pair => ("pair".to_owned(), vec![rank(play.value); 2]),
        GuandanPlayKind::Triple => ("triple".to_owned(), vec![rank(play.value); 3]),
        GuandanPlayKind::Bomb(size) => (
            format!("{size}_bomb"),
            vec![rank(play.value); usize::from(size)],
        ),
        GuandanPlayKind::JokerBomb => ("joker_bomb".to_owned(), vec![16, 16, 17, 17]),
        GuandanPlayKind::FullHouse => {
            let mut counts = BTreeMap::<u8, u8>::new();
            for card in &play.cards {
                if !card.is_wild(level) && card.rank != play.value {
                    *counts.entry(card.rank).or_default() += 1;
                }
            }
            let pair = counts
                .iter()
                .max_by_key(|(_, count)| *count)
                .map(|(rank, _)| *rank)?;
            let mut points = vec![rank(play.value); 3];
            points.extend([rank(pair); 2]);
            ("three_with_pair".to_owned(), points)
        }
        GuandanPlayKind::Straight
        | GuandanPlayKind::StraightFlush
        | GuandanPlayKind::ConsecutivePairs
        | GuandanPlayKind::ConsecutiveTriples => {
            let (kind, copies) = match play.kind {
                GuandanPlayKind::Straight => ("straight", 1),
                GuandanPlayKind::StraightFlush => ("flush_rocket", 1),
                GuandanPlayKind::ConsecutivePairs => ("pair_chain", 2),
                _ => ("gangban", 3),
            };
            let length = play.cards.len() / copies;
            let start = usize::from(play.value) + 1 - length;
            let points = (start..start + length)
                .flat_map(|r| vec![r as u8; copies])
                .collect();
            (kind.to_owned(), points)
        }
    };
    points.sort_unstable();
    actions
        .iter()
        .find(|action| {
            let mut expected = action.points.clone();
            expected.sort_unstable();
            action.kind == kind && expected == points
        })
        .map(|action| action.id)
}
