pub(super) use super::super::prelude::*;
use crate::build_deck_for_rules;
pub(super) use crate::{
    Mode, UnoActionContext, UnoActionStatistics, UnoCard, UnoColor, UnoFace, UnoMatchStatistics,
    UnoPlayerStatistics, UnoRuleSet,
};

pub(super) fn table(
    rules: UnoRuleSet,
    hands: Vec<Vec<UnoCard>>,
    top: UnoCard,
) -> (GameState, UnoMatchStatistics) {
    let mut game =
        GameState::new_with_deck(rules, hands.len() as u8, build_deck_for_rules(rules)).unwrap();
    for (player, hand) in game.players.iter_mut().zip(hands) {
        player.hand = hand;
    }
    game.discard_pile = vec![top];
    game.current_color = top.color();
    game.current_player = UnoPlayerId(0);
    game.pending_skip = 0;
    game.skip_turns.fill(0);
    let stats = UnoMatchStatistics::new(&game);
    (game, stats)
}

pub(super) fn record(
    game: &mut GameState,
    stats: &mut UnoMatchStatistics,
    player: usize,
    played: &[(UnoCard, Option<UnoColor>)],
    jump_in: bool,
    action: impl FnOnce(&mut GameState, UnoPlayerId) -> Result<ActionOutcome, GameError>,
) -> Vec<UnoActionStatistics> {
    let actor = UnoPlayerId(player);
    let before = UnoActionContext::capture(game, actor).unwrap();
    let outcome = action(game, actor).unwrap();
    stats.observe(before, &outcome, played, jump_in, game)
}

pub(super) fn progress(stats: &UnoMatchStatistics, player: usize) -> &UnoPlayerStatistics {
    stats.player_statistics(UnoPlayerId(player)).unwrap()
}

pub(super) fn number(color: UnoColor, value: u8) -> UnoCard {
    UnoCard::number(color, value, 0)
}
