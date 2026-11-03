use leocard_game_common::PlayerId;
use leocard_mahjong::{MahjongBotAction, MahjongClaim, MahjongGameState, MahjongPhase};

pub(super) struct Policy;

impl Policy {
    pub fn choose(game: &MahjongGameState, player: PlayerId) -> Option<MahjongBotAction> {
        match game.phase() {
            MahjongPhase::WaitingForClaims(pending) => {
                pending.options_for(player)?;
                Some(MahjongBotAction::Respond(MahjongClaim::Pass))
            }
            MahjongPhase::Playing if game.current_player() == player => {
                let hand = game.player(player)?.hand();
                // Normal draws and flower replacements discard the drawn tile.
                // Enabling auto-play just after a chow/pung has no drawn tile;
                // use the last held tile without making a hand-strength decision.
                game.last_drawn()
                    .filter(|tile| hand.contains(tile))
                    .or_else(|| hand.last().copied())
                    .map(MahjongBotAction::Discard)
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use leocard_mahjong::{MahjongRuleSet, build_deck};

    #[test]
    fn auto_play_discards_every_draw_and_passes_every_claim_until_the_wall_is_empty() {
        let mut game =
            MahjongGameState::new_with_deck(MahjongRuleSet::default(), build_deck(), PlayerId(0))
                .unwrap();
        let mut discards = 0;
        let mut passes = 0;
        for _ in 0..1000 {
            match game.phase() {
                MahjongPhase::Dealing { .. } => {
                    game.advance_deal().unwrap();
                }
                MahjongPhase::ReplacingFlower { .. } => {
                    game.advance_flower_replacement().unwrap();
                }
                MahjongPhase::Playing => {
                    let player = game.current_player();
                    let drawn = game.last_drawn().unwrap();
                    assert_eq!(
                        Policy::choose(&game, player),
                        Some(MahjongBotAction::Discard(drawn))
                    );
                    game.discard(player, drawn).unwrap();
                    assert!(game.discards().last().unwrap().from_drawn);
                    discards += 1;
                }
                MahjongPhase::WaitingForClaims(pending) => {
                    let players = pending.waiting_for();
                    for player in players {
                        assert_eq!(
                            Policy::choose(&game, player),
                            Some(MahjongBotAction::Respond(MahjongClaim::Pass))
                        );
                        game.respond_to_claim(player, MahjongClaim::Pass).unwrap();
                        passes += 1;
                    }
                }
                MahjongPhase::Finished(result) => {
                    assert!(result.exhaustive_draw);
                    assert!(discards > 0 && passes > 0);
                    return;
                }
            }
        }
        panic!("auto draw-discard must finish the hand");
    }

    #[test]
    fn enabling_after_pung_discards_a_held_tile_without_getting_stuck() {
        use leocard_mahjong::MahjongClaimOption;
        let mut deck = build_deck();
        fastrand::Rng::with_seed(401).shuffle(&mut deck);
        let mut game =
            MahjongGameState::new_with_deck(MahjongRuleSet::default(), deck, PlayerId(0)).unwrap();
        for _ in 0..1000 {
            match game.phase() {
                MahjongPhase::Dealing { .. } => {
                    game.advance_deal().unwrap();
                }
                MahjongPhase::ReplacingFlower { .. } => {
                    game.advance_flower_replacement().unwrap();
                }
                MahjongPhase::Playing => {
                    let player = game.current_player();
                    let MahjongBotAction::Discard(tile) = Policy::choose(&game, player).unwrap()
                    else {
                        panic!("discard")
                    };
                    game.discard(player, tile).unwrap();
                }
                MahjongPhase::WaitingForClaims(pending) => {
                    let players = pending.waiting_for();
                    let claimant = players.iter().copied().find(|player| {
                        pending
                            .options_for(*player)
                            .unwrap()
                            .contains(&MahjongClaimOption::Pung)
                    });
                    for player in players
                        .iter()
                        .copied()
                        .filter(|player| Some(*player) != claimant)
                    {
                        game.respond_to_claim(player, MahjongClaim::Pass).unwrap();
                    }
                    if let Some(player) = claimant {
                        game.respond_to_claim(player, MahjongClaim::Pung).unwrap();
                        assert!(game.last_drawn().is_none());
                        let held = *game.player(player).unwrap().hand().last().unwrap();
                        assert_eq!(
                            Policy::choose(&game, player),
                            Some(MahjongBotAction::Discard(held))
                        );
                        game.discard(player, held).unwrap();
                        return;
                    }
                }
                MahjongPhase::Finished(_) => break,
            }
        }
        panic!("fixture must provide a pung opportunity");
    }
}
