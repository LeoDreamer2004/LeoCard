use crate::achievements::{
    DoudizhuAchievementFacts, DoudizhuAchievementTracker, DoudizhuActionContext,
};
use crate::{Action, Card, GameState, Mode, RuleSet, build_deck};
use std::collections::HashSet;

pub(super) fn bomb(rank: u8) -> Vec<Card> {
    (0..4).map(|suit| Card(rank * 4 + suit)).collect()
}

pub(super) fn game(prefixes: [Vec<Card>; 3]) -> (GameState, DoudizhuAchievementTracker) {
    game_with_kitty(Mode::Three, &prefixes, &[])
}

pub(super) fn game_with_kitty(
    mode: Mode,
    prefixes: &[Vec<Card>],
    kitty: &[Card],
) -> (GameState, DoudizhuAchievementTracker) {
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
    let mut game = GameState::new(RuleSet::for_mode(mode), deck, 0).unwrap();
    let mut tracker = DoudizhuAchievementTracker::new(mode.players());
    accepted(&mut game, &mut tracker, 0, Action::Bid(3));
    (game, tracker)
}

pub(super) fn accepted(
    game: &mut GameState,
    tracker: &mut DoudizhuAchievementTracker,
    player: u8,
    action: Action,
) -> Vec<(u8, DoudizhuAchievementFacts)> {
    let before = DoudizhuActionContext::capture(game);
    game.act(player, action.clone()).unwrap();
    tracker.observe_action(player, &action, before, game)
}
