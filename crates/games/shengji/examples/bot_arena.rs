//! Paired deals, rotated dealers/levels, and equal observations for both teams.
//! Usage: cargo run -r -p leocard-shengji --example bot_arena -- 6 32 40 rules 1000
use leocard_game_common::{PlayerId, SearchBudget, SearchRandom};
use leocard_shengji::{
    ShengjiBiddingObservation, ShengjiBot, ShengjiBotAction, ShengjiBotObservation,
    ShengjiGameState, ShengjiGreedyBot, ShengjiGreedyBotRequest, ShengjiMode, ShengjiPhase,
    ShengjiRank, ShengjiRuleSet, ShengjiTeamProgress, build_deck_for,
};
use std::env;
use std::time::{Duration, Instant};

#[derive(Default)]
struct Metrics {
    wins: usize,
    margins: Vec<f64>,
    times: [Vec<Duration>; 2],
}

fn main() {
    let args: Vec<_> = env::args().collect();
    let players = number(&args, 1, 6) as u8;
    assert!(matches!(players, 4 | 6), "players must be 4 or 6");
    let pairs = number(&args, 2, 16);
    assert!(pairs > 0, "at least one pair is required");
    let milliseconds = number(&args, 3, 40);
    let opponent = args.get(4).map(String::as_str).unwrap_or("rules");
    assert!(matches!(opponent, "rules" | "greedy"));
    let start = number(&args, 5, 1000);
    let mut metrics = Metrics::default();
    for seed in start..start + pairs {
        let game = setup(seed, players);
        let mut paired_margin = 0.0;
        for team in 0..2 {
            let mut game = game.clone();
            for _ in 0..400 {
                if matches!(game.phase(), ShengjiPhase::Finished(_)) {
                    break;
                }
                let player = game.current_player().expect("playing");
                let candidate = player.0 % 2 == team;
                let began = Instant::now();
                let cards = if !candidate && opponent == "greedy" {
                    let trick = game.current_trick();
                    ShengjiGreedyBot::choose(ShengjiGreedyBotRequest {
                        hand: &game.players()[usize::from(player.0)].hand,
                        lead: trick.as_ref().map(|trick| &trick.plays[0].1),
                        trump: game.trump().unwrap(),
                    })
                    .unwrap()
                    .cards
                } else {
                    let observation = ShengjiBotObservation::from_game(&game, player).unwrap();
                    let budget = SearchBudget::new(Duration::from_millis(if candidate {
                        milliseconds
                    } else {
                        0
                    }));
                    let ShengjiBotAction::Play(cards) = ShengjiBot::choose(&observation, &budget)
                    else {
                        panic!("play")
                    };
                    cards
                };
                metrics.times[usize::from(candidate)].push(began.elapsed());
                game.play_cards(player, &cards).unwrap_or_else(|error| {
                    panic!("seed={seed} team={team} player={player:?}: {error}")
                });
            }
            let ShengjiPhase::Finished(result) = game.phase() else {
                panic!("unfinished hand")
            };
            let won = result.promoted_team.0 == team;
            metrics.wins += usize::from(won);
            let sign = if result.collecting_team.0 == team {
                1.0
            } else {
                -1.0
            };
            let margin = (f64::from(result.collecting_score)
                - f64::from(game.rules().takeover_score()))
                * sign;
            paired_margin += margin / 2.0;
            println!("seed={seed} team={team} won={won} margin={margin:.1}");
        }
        metrics.margins.push(paired_margin);
    }
    let mean = metrics.margins.iter().sum::<f64>() / pairs as f64;
    println!(
        "players={players} pairs={pairs} budget_ms={milliseconds} opponent={opponent} wins={}/{} mean_paired_margin={mean:.2}",
        metrics.wins,
        pairs * 2
    );
    for (side, times) in metrics.times.iter_mut().enumerate() {
        times.sort_unstable();
        println!(
            "candidate={} decisions={} p95={:?} max={:?}",
            side == 1,
            times.len(),
            times[(times.len() - 1) * 95 / 100],
            times.last().unwrap()
        );
    }
}

fn number(args: &[String], index: usize, default: u64) -> u64 {
    args.get(index)
        .map_or(default, |value| value.parse().expect("unsigned integer"))
}

fn setup(seed: u64, players: u8) -> ShengjiGameState {
    let mut rules = ShengjiRuleSet::for_mode(if players == 6 {
        ShengjiMode::SixPlayer
    } else {
        ShengjiMode::Classic
    });
    if players == 4 {
        rules.deck_count = 2 + (seed % 3) as u8;
    }
    rules.constant_trump = seed % 2 == 1;
    let mut deck = build_deck_for(rules.deck_count);
    let mut random = SearchRandom::new(seed);
    for index in (1..deck.len()).rev() {
        deck.swap(index, random.index(index + 1));
    }
    let level = ShengjiRank::LEVELS[if rules.constant_trump {
        1 + seed as usize % 12
    } else {
        seed as usize % 13
    }];
    let mut game = ShengjiGameState::new(
        rules,
        ShengjiTeamProgress { levels: [level; 2] },
        Some(PlayerId((seed % u64::from(players)) as u8)),
        PlayerId(0),
        deck,
    )
    .unwrap();
    game.deal_all().unwrap();
    for seat in 0..players {
        let player = PlayerId(seat);
        if let Some(cards) =
            ShengjiBot::declaration(&ShengjiBiddingObservation::from_game(&game, player).unwrap())
        {
            game.declare(player, &cards).unwrap();
        }
    }
    game.close_bidding_and_take_kitty().unwrap();
    let player = game.dealer().unwrap();
    let ShengjiBotAction::Bury(cards) = ShengjiBot::choose(
        &ShengjiBotObservation::from_game(&game, player).unwrap(),
        &SearchBudget::new(Duration::ZERO),
    ) else {
        panic!("bury")
    };
    game.bury(player, &cards).unwrap();
    game
}
