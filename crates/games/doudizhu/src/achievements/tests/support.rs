use crate::achievements::{
    DoudizhuAchievementFacts, DoudizhuAchievementTracker, DoudizhuActionContext,
};
use crate::{
    DoudizhuAction, DoudizhuCard, DoudizhuGameState, DoudizhuMode, DoudizhuRuleSet, build_deck,
};
use leocard_game_common::PlayerId;
use std::collections::HashSet;

pub(super) fn bomb(rank: u8) -> Vec<DoudizhuCard> {
    (0..4).map(|suit| DoudizhuCard(rank * 4 + suit)).collect()
}

pub(super) fn game(
    prefixes: [Vec<DoudizhuCard>; 3],
) -> (DoudizhuGameState, DoudizhuAchievementTracker) {
    game_with_kitty(DoudizhuMode::Three, &prefixes, &[])
}

pub(super) fn game_with_kitty(
    mode: DoudizhuMode,
    prefixes: &[Vec<DoudizhuCard>],
    kitty: &[DoudizhuCard],
) -> (DoudizhuGameState, DoudizhuAchievementTracker) {
    let count = usize::from(mode.players());
    assert_eq!(prefixes.len(), count);
    assert!(kitty.is_empty() || kitty.len() == mode.kitty_len());
    let reserved: HashSet<_> = prefixes.iter().flatten().chain(kitty).copied().collect();
    assert_eq!(
        reserved.len(),
        prefixes.iter().map(Vec::len).sum::<usize>() + kitty.len()
    );
    let mut rest = build_deck(mode)
        .into_iter()
        .filter(|card| !reserved.contains(card));
    let mut deck = Vec::new();
    let deal_count = usize::from(mode.deck_count()) * 54 - mode.kitty_len();
    for index in 0..deal_count {
        deck.push(
            prefixes[index % count]
                .get(index / count)
                .copied()
                .unwrap_or_else(|| rest.next().unwrap()),
        );
    }
    deck.extend_from_slice(kitty);
    deck.extend(rest);
    let mut game =
        DoudizhuGameState::new(DoudizhuRuleSet::for_mode(mode), deck, PlayerId(0)).unwrap();
    let mut tracker = DoudizhuAchievementTracker::new(mode.players());
    accepted(&mut game, &mut tracker, PlayerId(0), DoudizhuAction::Bid(3));
    (game, tracker)
}

pub(super) fn accepted(
    game: &mut DoudizhuGameState,
    tracker: &mut DoudizhuAchievementTracker,
    player: PlayerId,
    action: DoudizhuAction,
) -> Vec<(PlayerId, DoudizhuAchievementFacts)> {
    let before = DoudizhuActionContext::capture(game);
    game.act(player, action.clone()).unwrap();
    tracker.observe_action(player, &action, before, game)
}
