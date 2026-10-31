use super::metrics::Metrics;
use leocard_game_common::PlayerId;
use leocard_mahjong::{
    MahjongBot, MahjongBotAction, MahjongBotObservation, MahjongClaim, MahjongClaimOption,
    MahjongGameState, MahjongMeldKind, MahjongPhase, MahjongRuleSet, build_deck,
};

use std::time::Duration;

pub fn run(seed: usize, budget: Duration, mixed: bool, metrics: &mut Metrics) {
    let mut deck = build_deck();
    fastrand::Rng::with_seed(seed as u64).shuffle(&mut deck);
    let mut game = MahjongGameState::new_with_deck(
        MahjongRuleSet {
            minimum_eight_points: seed % 4 != 3,
            multiple_winners: seed.is_multiple_of(3),
            ..MahjongRuleSet::default()
        },
        deck,
        PlayerId::from_index(if mixed { 0 } else { seed % 4 }),
    )
    .unwrap();
    for _ in 0..1000 {
        let player = match game.phase() {
            MahjongPhase::Dealing { .. } => {
                game.advance_deal().unwrap();
                continue;
            }
            MahjongPhase::ReplacingFlower { .. } => {
                game.advance_flower_replacement().unwrap();
                continue;
            }
            MahjongPhase::Playing => game.current_player(),
            MahjongPhase::WaitingForClaims(pending) => pending.waiting_for()[0],
            MahjongPhase::Finished(result) => {
                metrics.draws += usize::from(result.exhaustive_draw);
                metrics.wins += usize::from(
                    result
                        .winners
                        .iter()
                        .any(|win| win.player.index() == seed % 4),
                );
                metrics.score += i64::from(result.deltas[seed % 4]);
                println!(
                    "mahjong seed={seed}: winners={:?}, draw={}",
                    result
                        .winners
                        .iter()
                        .map(|win| (win.player.0, win.score.total_points))
                        .collect::<Vec<_>>(),
                    result.exhaustive_draw
                );
                return;
            }
        };
        let action = if mixed && player.index() != seed % 4 {
            baseline(&game, player)
        } else {
            let observation = MahjongBotObservation::from_game(&game, player).unwrap();
            metrics.decide(budget, |budget| MahjongBot::choose(&observation, budget))
        };
        match action {
            MahjongBotAction::Discard(tile) => game.discard(player, tile),
            MahjongBotAction::Respond(claim) => game.respond_to_claim(player, claim),
            MahjongBotAction::SelfDraw => game.declare_self_draw(player),
            MahjongBotAction::ConcealedKong(tile) => game.declare_concealed_kong(player, tile),
            MahjongBotAction::AddedKong(tile) => game.declare_added_kong(player, tile),
        }
        .unwrap_or_else(|error| {
            panic!("mahjong seed={seed} player={player:?} action={action:?}: {error}")
        });
    }
    panic!("mahjong did not finish");
}

fn baseline(game: &MahjongGameState, player: PlayerId) -> MahjongBotAction {
    let observation = MahjongBotObservation::from_game(game, player).unwrap();
    if let Some(action) = observation.immediate_action() {
        return action;
    }
    if let MahjongPhase::WaitingForClaims(pending) = game.phase() {
        let options = pending.options_for(player).unwrap();
        let claim = if options.contains(&MahjongClaimOption::Kong) {
            MahjongClaim::Kong
        } else if options.contains(&MahjongClaimOption::Pung) {
            MahjongClaim::Pung
        } else if let Some(start) = options.iter().find_map(|option| match option {
            MahjongClaimOption::Chow { start } => Some(*start),
            _ => None,
        }) {
            MahjongClaim::Chow { start }
        } else {
            MahjongClaim::Pass
        };
        return MahjongBotAction::Respond(claim);
    }
    let state = game.player(player).unwrap();
    let hand = state.hand();
    if game.wall_len() > 0 {
        for tile in hand {
            if hand
                .iter()
                .filter(|held| held.kind() == tile.kind())
                .count()
                == 4
            {
                return MahjongBotAction::ConcealedKong(tile.kind());
            }
            if state
                .melds()
                .iter()
                .any(|meld| meld.kind() == MahjongMeldKind::Pung && meld.tile() == tile.kind())
            {
                return MahjongBotAction::AddedKong(*tile);
            }
        }
    }
    let tile = hand
        .iter()
        .min_by_key(|tile| {
            let index = tile.kind().index34().unwrap();
            let identical = hand
                .iter()
                .filter(|held| held.kind() == tile.kind())
                .count()
                .saturating_sub(1);
            let neighbors = hand
                .iter()
                .filter_map(|held| held.kind().index34())
                .filter(|other| index < 27 && *other < 27 && *other / 9 == index / 9)
                .map(|other| match index.abs_diff(other) {
                    1 => 4,
                    2 => 2,
                    _ => 0,
                })
                .sum::<usize>();
            (identical * 7 + neighbors, tile.kind(), tile.copy())
        })
        .unwrap();
    MahjongBotAction::Discard(*tile)
}
