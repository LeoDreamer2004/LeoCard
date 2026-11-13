use leocard_game_common::PlayerId;
use leocard_mahjong::{MahjongBotAction, MahjongClaim, MahjongGameState, MahjongPhase};

pub(super) struct Policy;

impl Policy {
    pub fn choose(game: &MahjongGameState, player: PlayerId) -> Option<MahjongBotAction> {
        match game.phase() {
            MahjongPhase::WaitingForClaims(pending) => {
                pending.options_for(player)?;
                let claim = if game.legal_claim_win_available(player).ok()? {
                    MahjongClaim::Win
                } else {
                    MahjongClaim::Pass
                };
                Some(MahjongBotAction::Respond(claim))
            }
            MahjongPhase::Playing if game.current_player() == player => {
                if game.legal_self_draw_available(player).ok()? {
                    return Some(MahjongBotAction::SelfDraw);
                }
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
    use leocard_mahjong::{
        MahjongActionOutcome, MahjongClaimOption, MahjongRuleSet, MahjongSuit, MahjongTile,
        MahjongTileKind, build_deck,
    };

    fn winning_game(player: PlayerId, false_win: bool) -> MahjongGameState {
        let hand = (1..=5)
            .flat_map(|rank| {
                (0..if rank == 5 { 2 } else { 3 }).map(move |copy| {
                    MahjongTile::new(MahjongTileKind::suited(MahjongSuit::Characters, rank), copy)
                })
            })
            .collect::<Vec<_>>();
        let mut slots = (0..3)
            .flat_map(|round| (0..4).map(move |index| round * 16 + player.index() * 4 + index))
            .collect::<Vec<_>>();
        slots.push(48 + player.index());
        // The dealer's jumped tile completes either their own hand or player 1's pair.
        slots.push(52);
        let mut deck = build_deck();
        for (slot, tile) in slots.into_iter().zip(hand) {
            let index = deck.iter().position(|held| *held == tile).unwrap();
            deck.swap(slot, index);
        }
        let mut game = MahjongGameState::new_with_deck(
            MahjongRuleSet {
                false_win,
                ..MahjongRuleSet::default()
            },
            deck,
            PlayerId(0),
        )
        .unwrap();
        while matches!(game.phase(), MahjongPhase::Dealing { .. }) {
            game.advance_deal().unwrap();
        }
        game
    }

    #[test]
    fn auto_play_takes_legal_self_draws_and_discard_wins_in_both_false_win_modes() {
        for false_win in [false, true] {
            for player in [PlayerId(0), PlayerId(1)] {
                let mut game = winning_game(player, false_win);
                let outcome = if player == PlayerId(0) {
                    assert_eq!(
                        Policy::choose(&game, player),
                        Some(MahjongBotAction::SelfDraw)
                    );
                    game.declare_self_draw(player).unwrap()
                } else {
                    let tile = game.last_drawn().unwrap();
                    game.discard(PlayerId(0), tile).unwrap();
                    assert_eq!(
                        Policy::choose(&game, player),
                        Some(MahjongBotAction::Respond(MahjongClaim::Win))
                    );
                    game.respond_to_claim(player, MahjongClaim::Win).unwrap()
                };
                let MahjongActionOutcome::HandFinished(result) = outcome else {
                    panic!("auto-play must take the available win");
                };
                assert_eq!(result.winners[0].player, player);
            }
        }
    }

    #[test]
    fn auto_play_draw_discards_and_passes_when_no_legal_win_is_available() {
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
                    assert!(!game.legal_self_draw_available(player).unwrap());
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
                        assert!(!game.legal_claim_win_available(player).unwrap());
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
