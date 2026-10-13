mod hosted;
mod mahjong;
mod metrics;
mod shengji;
mod texas;

use metrics::Metrics;
use std::{env, time::Duration};

fn main() {
    let args: Vec<_> = env::args().collect();
    let game = args.get(1).map(String::as_str).unwrap_or("all");
    if game == "hosted" {
        hosted::run();
        return;
    }
    let games = args
        .get(2)
        .and_then(|value| value.parse().ok())
        .unwrap_or(8);
    let milliseconds = args
        .get(3)
        .and_then(|value| value.parse().ok())
        .unwrap_or(100);
    let budget = Duration::from_millis(milliseconds);
    let mixed = args.iter().skip(4).any(|value| value == "mixed");
    let starting_chips = args
        .iter()
        .skip(4)
        .find_map(|value| value.parse().ok())
        .unwrap_or(50);
    for name in ["mahjong", "texas", "shengji"] {
        if game != "all" && game != name {
            continue;
        }
        let mut metrics = Metrics::default();
        for seed in 0..games {
            match name {
                "mahjong" => mahjong::run(seed, budget, mixed, &mut metrics),
                "texas" => texas::run(seed, budget, mixed, starting_chips, &mut metrics),
                _ => shengji::run(seed, budget, mixed, &mut metrics),
            }
        }
        metrics.report(name);
    }
}
