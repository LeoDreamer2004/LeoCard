use super::metrics::Metrics;
use leocard_shengji::{
    GameState, Phase, ShengjiBot, ShengjiBotAction, ShengjiBotObservation, ShengjiGreedyBot,
    ShengjiGreedyBotRequest, ShengjiPlayerId, ShengjiRuleSet, TeamProgress, build_deck_for,
};
use std::time::Duration;

pub fn run(seed: usize, budget: Duration, mixed: bool, metrics: &mut Metrics) {
    let rules = ShengjiRuleSet {
        deck_count: 2 + (seed % 3) as u8,
        constant_trump: seed % 2 == 1,
        bottom_copy: seed % 3 == 1,
        five_trump_crossing: seed % 3 == 2,
        bid_with_joker: seed % 4 == 3,
        ..ShengjiRuleSet::default()
    };
    let mut deck = build_deck_for(rules.deck_count);
    fastrand::Rng::with_seed(seed as u64).shuffle(&mut deck);
    let mut game = GameState::new(
        rules,
        TeamProgress::for_rules(&rules),
        Some(ShengjiPlayerId(if mixed { 0 } else { seed as u8 % 4 })),
        ShengjiPlayerId(0),
        deck,
    )
    .unwrap();
    game.deal_all().unwrap();
    for index in 0..4 {
        let player = ShengjiPlayerId(index);
        if let Some(cards) = ShengjiBot::declaration(
            game.bidding(),
            player,
            &game.players()[usize::from(index)].hand,
            rules,
            true,
        ) {
            game.declare(player, &cards).unwrap();
        }
    }
    game.close_bidding_and_take_kitty().unwrap();
    for _ in 0..600 {
        let player = match game.phase() {
            Phase::Burying => game.dealer().unwrap(),
            Phase::BottomCopying => game.bottom_copy().unwrap().current().unwrap(),
            Phase::BottomCopyBurying => game.bottom_copy().unwrap().bottom_holder().unwrap(),
            Phase::FiveTrumpCrossing => game.five_trump_crossing().unwrap().pending_players()[0],
            Phase::Playing => game.current_player().unwrap(),
            Phase::Finished(result) => {
                metrics.wins += usize::from(result.promoted_team.0 == (seed % 2) as u8);
                metrics.score += if result.collecting_team.0 == (seed % 2) as u8 {
                    i64::from(result.collecting_score)
                } else {
                    -i64::from(result.collecting_score)
                };
                println!(
                    "shengji seed={seed}: collecting={} promoted={:?} steps={}",
                    result.collecting_score, result.promoted_team, result.promoted_steps
                );
                return;
            }
            phase => panic!("unexpected shengji phase {phase:?}"),
        };
        let observation = ShengjiBotObservation::from_game(&game, player).unwrap();
        let action = if mixed && player.0 % 2 != (seed % 2) as u8 {
            if game.phase() == &Phase::Playing {
                let trick = game.current_trick();
                ShengjiBotAction::Play(
                    ShengjiGreedyBot::choose(ShengjiGreedyBotRequest {
                        hand: &game.players()[usize::from(player.0)].hand,
                        lead: trick.as_ref().map(|trick| &trick.plays[0].1),
                        trump: game.trump().unwrap(),
                    })
                    .unwrap()
                    .cards,
                )
            } else {
                observation.fallback()
            }
        } else {
            metrics.decide(budget, |budget| ShengjiBot::choose(&observation, budget))
        };
        let result = match &action {
            ShengjiBotAction::Play(cards) => game.play_cards(player, cards),
            ShengjiBotAction::Bury(cards) => game.bury(player, cards),
            ShengjiBotAction::Copy(cards) => game.choose_bottom_copy(player, cards.as_deref()),
            ShengjiBotAction::Cross(cards) => {
                game.choose_five_trump_crossing(player, cards.as_deref())
            }
            ShengjiBotAction::Return(cards) => game.return_five_trump_crossing(player, cards),
        };
        result.unwrap_or_else(|error| panic!("shengji seed={seed} action={action:?}: {error}"));
    }
    panic!("shengji did not finish");
}
