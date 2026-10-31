pub(super) use super::super::prelude::*;
use crate::build_deck_for_rules;
pub(super) use crate::{
    UnoActionContext, UnoActionStatistics, UnoCard, UnoColor, UnoFace, UnoMatchStatistics, UnoMode,
    UnoPlayerStatistics, UnoRuleSet,
};
use leocard_game_common::PlayerId;

pub(super) fn table(
    rules: UnoRuleSet,
    hands: Vec<Vec<UnoCard>>,
    top: UnoCard,
) -> (UnoGameState, UnoMatchStatistics) {
    let mut game =
        UnoGameState::new_with_deck(rules, hands.len() as u8, build_deck_for_rules(rules)).unwrap();
    for (player, hand) in game.players.iter_mut().zip(hands) {
        player.hand = hand;
    }
    game.discard_pile = vec![top];
    game.current_color = top.color();
    game.current_player = PlayerId(0);
    game.pending_skip = 0;
    game.skip_turns.fill(0);
    let stats = UnoMatchStatistics::new(&game);
    (game, stats)
}

pub(super) fn record(
    game: &mut UnoGameState,
    stats: &mut UnoMatchStatistics,
    player: usize,
    played: &[(UnoCard, Option<UnoColor>)],
    jump_in: bool,
    action: impl FnOnce(&mut UnoGameState, PlayerId) -> Result<UnoActionOutcome, UnoGameError>,
) -> Vec<UnoActionStatistics> {
    let actor = PlayerId::from_index(player);
    let before = UnoActionContext::capture(game, actor).unwrap();
    let outcome = action(game, actor).unwrap();
    stats.observe(before, &outcome, played, jump_in, game)
}

pub(super) fn progress(stats: &UnoMatchStatistics, player: usize) -> &UnoPlayerStatistics {
    stats
        .player_statistics(PlayerId::from_index(player))
        .unwrap()
}

pub(super) fn number(color: UnoColor, value: u8) -> UnoCard {
    UnoCard::number(color, value, 0)
}
