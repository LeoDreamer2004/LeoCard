use super::metrics::Metrics;
use leocard_game_common::PlayerId;

use leocard_texas_holdem::{
    TexasAction, TexasBot, TexasBotObservation, TexasGameState, TexasPhase, TexasRuleSet,
    TexasStreet, build_deck,
};
use std::time::Duration;

pub fn run(seed: usize, budget: Duration, mixed: bool, starting_chips: u16, metrics: &mut Metrics) {
    let rules = TexasRuleSet {
        player_count: 3 + (seed % 4) as u8,
        starting_chips,
        short_deck: seed / 4 % 4 == 1,
        omaha: seed / 4 % 4 == 2,
        ignore_kickers: seed / 4 % 4 == 3,
        ..TexasRuleSet::default()
    };
    let mut deck = build_deck(rules.short_deck);
    fastrand::Rng::with_seed(seed as u64).shuffle(&mut deck);
    let mut game = TexasGameState::new_with_deck(
        rules,
        PlayerId::from_index(seed % usize::from(rules.player_count)),
        deck,
    )
    .unwrap();
    let mut reached_flop = false;
    for _ in 0..300 {
        if let TexasPhase::Complete(result) = game.phase() {
            metrics.score += i64::from(result.final_stacks[0]) - i64::from(starting_chips);
            println!(
                "texas seed={seed}: stacks={:?} showdown={}",
                result.final_stacks, result.showdown
            );
            return;
        }
        let player = game.current_player().unwrap();
        let observation = TexasBotObservation::from_game(&game, player).unwrap();
        let action = if mixed && player.0 != 0 {
            observation
                .immediate_action()
                .unwrap_or_else(|| observation.fallback())
        } else {
            metrics.decide(budget, |budget| TexasBot::choose(&observation, budget))
        };
        if game.phase() == &TexasPhase::Betting(TexasStreet::PreFlop)
            && action != TexasAction::PostBlind
        {
            metrics.preflop_actions += 1;
            metrics.preflop_folds += usize::from(action == TexasAction::Fold);
        }
        if let TexasAction::RaiseTo(target) = action {
            metrics.off_step_raises +=
                usize::from(!(target - game.current_bet()).is_multiple_of(game.minimum_raise()));
        }
        match action {
            TexasAction::RaiseTo(_) | TexasAction::AllIn => metrics.gains += 1,
            TexasAction::Call => metrics.calls += 1,
            TexasAction::Fold => metrics.folds += 1,
            _ => {}
        }
        game.act(player, action)
            .unwrap_or_else(|error| panic!("texas seed={seed} action={action:?}: {error}"));
        if !reached_flop && game.community().len() >= 3 {
            metrics.flops += 1;
            reached_flop = true;
        }
    }
    panic!("texas did not finish");
}
