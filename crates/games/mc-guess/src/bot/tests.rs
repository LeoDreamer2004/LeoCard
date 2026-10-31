use leocard_game_common::PlayerId;
mod graph;

use super::{McGuessBot, McGuessBotObservation};
use crate::{McGuessCatalogue, McGuessEntryKind, McGuessGameState, McGuessRuleSet};
use fastrand::Rng;

#[test]
fn identical_public_hints_produce_the_same_decision_for_different_answers() {
    let catalogue = McGuessCatalogue::bundled();
    let game = |id| {
        McGuessGameState::new(
            catalogue.find(McGuessEntryKind::Block, id).unwrap().clone(),
            vec![PlayerId(0), PlayerId(1)],
            McGuessRuleSet::default(),
        )
    };
    let stone = game("minecraft:stone");
    let dirt = game("minecraft:dirt");
    let first = McGuessBotObservation::from_game(&stone, PlayerId(0)).unwrap();
    let second = McGuessBotObservation::from_game(&dirt, PlayerId(0)).unwrap();
    assert_eq!(first, second);
    assert_eq!(
        McGuessBot::choose(&first, &mut Rng::with_seed(7)),
        McGuessBot::choose(&second, &mut Rng::with_seed(7))
    );
    assert!(McGuessBotObservation::from_game(&stone, PlayerId(1)).is_none());
}
