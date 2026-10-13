use super::metrics::Metrics;
use leocard_texas_holdem::{
    GameState, Phase, TexasHoldemAction, TexasHoldemBot, TexasHoldemBotObservation,
    TexasHoldemPlayerId, TexasHoldemRuleSet, TexasHoldemStreet, build_deck,
};
use std::time::Duration;

pub fn run(seed: usize, budget: Duration, mixed: bool, starting_chips: u16, metrics: &mut Metrics) {
    let rules = TexasHoldemRuleSet {
        player_count: 3 + (seed % 4) as u8,
        starting_chips,
        short_deck: seed / 4 % 4 == 1,
        omaha: seed / 4 % 4 == 2,
        ignore_kickers: seed / 4 % 4 == 3,
    };
    let mut deck = build_deck(rules.short_deck);
    fastrand::Rng::with_seed(seed as u64).shuffle(&mut deck);
    let mut game = GameState::new_with_deck(
        rules,
        TexasHoldemPlayerId(seed % usize::from(rules.player_count)),
        deck,
    )
    .unwrap();
    let mut reached_flop = false;
    for _ in 0..300 {
        if let Phase::Complete(result) = game.phase() {
            metrics.score += i64::from(result.final_stacks[0]) - i64::from(starting_chips);
            println!(
                "texas seed={seed}: stacks={:?} showdown={}",
                result.final_stacks, result.showdown
            );
            return;
        }
        let player = game.current_player().unwrap();
        let observation = TexasHoldemBotObservation::from_game(&game, player).unwrap();
        let action = if mixed && player.0 != 0 {
            observation
                .immediate_action()
                .unwrap_or_else(|| observation.fallback())
        } else {
            metrics.decide(budget, |budget| {
                TexasHoldemBot::choose(&observation, budget)
            })
        };
        if game.phase() == &Phase::Betting(TexasHoldemStreet::PreFlop)
            && action != TexasHoldemAction::PostBlind
        {
            metrics.preflop_actions += 1;
            metrics.preflop_folds += usize::from(action == TexasHoldemAction::Fold);
        }
        if let TexasHoldemAction::RaiseTo(target) = action {
            metrics.off_step_raises +=
                usize::from(!(target - game.current_bet()).is_multiple_of(game.minimum_raise()));
        }
        match action {
            TexasHoldemAction::RaiseTo(_) | TexasHoldemAction::AllIn => metrics.gains += 1,
            TexasHoldemAction::Call => metrics.calls += 1,
            TexasHoldemAction::Fold => metrics.folds += 1,
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
