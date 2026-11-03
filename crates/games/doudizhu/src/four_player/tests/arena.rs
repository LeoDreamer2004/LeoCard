use crate::{
    DoudizhuAction, DoudizhuGameState, DoudizhuMode, DoudizhuPhase, DoudizhuRuleSet, FourPlayerBot,
    FourPlayerObservation, bot_action, build_deck,
};
use leocard_game_common::{PlayerId, SearchBudget, SearchRandom};
use std::time::{Duration, Instant};

/// Paired deals exercise every seat against the old policy; eight games are only a smoke benchmark.
#[test]
#[ignore = "run explicitly in release mode to evaluate simulation time and paired outcomes"]
fn four_player_simulation_arena() {
    let mut times = Vec::new();
    let mut sweeps = Vec::new();
    let mut wins = [0, 0];
    for number in 0..8 {
        let mut random = SearchRandom::new(52300 + number);
        let mut deck = build_deck(DoudizhuMode::Four);
        for end in (1..deck.len()).rev() {
            deck.swap(end, random.index(end + 1));
        }
        let actor = PlayerId((number % 4) as u8);
        for enhanced in [false, true] {
            let rules = DoudizhuRuleSet {
                mode: DoudizhuMode::Four,
                allow_single_wings: number >= 4,
                ..Default::default()
            };
            let mut game = DoudizhuGameState::new(rules, deck.clone(), PlayerId(0)).unwrap();
            game.act(PlayerId(0), DoudizhuAction::Bid(3)).unwrap();
            for _ in 0..432 {
                if game.phase == DoudizhuPhase::Finished {
                    break;
                }
                let action = if enhanced && game.current == actor {
                    let started = Instant::now();
                    let input = FourPlayerObservation::from_game(&game).unwrap();
                    let mut updates = 0;
                    let action = FourPlayerBot::choose_with_progress(
                        &input,
                        &SearchBudget::new(Duration::from_secs(2)),
                        &mut |_| {
                            updates += 1;
                        },
                    );
                    times.push(started.elapsed().as_secs_f64() * 1000.0);
                    sweeps.push(updates - 1);
                    action
                } else {
                    bot_action(&game).unwrap()
                };
                game.act(game.current, action).unwrap();
            }
            let result = game.result.expect("legal game must finish");
            wins[usize::from(enhanced)] +=
                usize::from(result.landlord_won == (actor == PlayerId(0)));
        }
    }
    times.sort_by(f64::total_cmp);
    sweeps.sort_unstable();
    println!(
        "8 paired deals: old team wins {}, simulated team wins {}; {} decisions, median {:.2} ms, p95 {:.2} ms, max {:.2} ms; median completed sweeps {}",
        wins[0],
        wins[1],
        times.len(),
        times[times.len() / 2],
        times[times.len() * 95 / 100],
        times[times.len() - 1],
        sweeps[sweeps.len() / 2]
    );
}
