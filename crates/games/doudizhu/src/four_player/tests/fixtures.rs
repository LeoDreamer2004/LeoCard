use crate::{DoudizhuCard, DoudizhuGameState, DoudizhuMode, DoudizhuRuleSet, build_deck};
use leocard_game_common::PlayerId;

pub(super) fn position(hands: [&[u8]; 4], actor: u8, landlord: u8) -> DoudizhuGameState {
    let mut groups: [Vec<DoudizhuCard>; 15] = Default::default();
    for card in build_deck(DoudizhuMode::Four).into_iter().rev() {
        groups[usize::from(card.rank())].push(card);
    }
    let hands: Vec<Vec<_>> = hands
        .into_iter()
        .map(|ranks| {
            ranks
                .iter()
                .map(|rank| groups[usize::from(*rank)].pop().unwrap())
                .collect()
        })
        .collect();
    let played = groups.into_iter().flatten().collect();
    let mut game = DoudizhuGameState::sampled_position(
        DoudizhuRuleSet::for_mode(DoudizhuMode::Four),
        hands,
        PlayerId(actor),
        PlayerId(landlord),
        None,
    );
    game.played = played;
    game
}
