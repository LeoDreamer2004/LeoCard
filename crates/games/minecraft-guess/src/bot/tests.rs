use super::{MinecraftGuessBot, MinecraftGuessBotObservation};
use crate::{Catalogue, EntryKind, GameState, MinecraftGuessRuleSet};
use fastrand::Rng;

#[test]
fn identical_public_hints_produce_the_same_decision_for_different_answers() {
    let catalogue = Catalogue::bundled();
    let game = |id| {
        GameState::new(
            catalogue.find(EntryKind::Block, id).unwrap().clone(),
            vec![0, 1],
            MinecraftGuessRuleSet::default(),
        )
    };
    let stone = game("minecraft:stone");
    let dirt = game("minecraft:dirt");
    let first = MinecraftGuessBotObservation::from_game(&stone, 0).unwrap();
    let second = MinecraftGuessBotObservation::from_game(&dirt, 0).unwrap();
    assert_eq!(first, second);
    assert_eq!(
        MinecraftGuessBot::choose(&first, &mut Rng::with_seed(7)),
        MinecraftGuessBot::choose(&second, &mut Rng::with_seed(7))
    );
    assert!(MinecraftGuessBotObservation::from_game(&stone, 1).is_none());
}
