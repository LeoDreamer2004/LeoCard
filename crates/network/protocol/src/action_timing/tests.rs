use super::{ActionTimeStats, GameActionTimes};
use crate::GameKind;

#[test]
fn ten_games_keep_independent_weighted_totals_maxima_and_round_trip_without_loss() {
    let mut times = GameActionTimes::default();
    let games = [
        GameKind::QiGui523,
        GameKind::Texas,
        GameKind::Shengji,
        GameKind::Uno,
        GameKind::Mahjong,
        GameKind::McGuess,
        GameKind::Blackjack,
        GameKind::Doudizhu,
        GameKind::Guandan,
        GameKind::MahjongLink,
    ];
    for (index, game) in games.into_iter().enumerate() {
        times.record(game, 60_000 + index as u64 * 1000);
        times.record(game, 500);
    }
    for (index, game) in games.into_iter().enumerate() {
        assert_eq!(
            times.get(game),
            ActionTimeStats {
                total_millis: 60_500 + index as u64 * 1000,
                completed_actions: 2,
                longest_millis: 60_000 + index as u64 * 1000,
            }
        );
    }
    assert_eq!(times.longest_millis(), 69_000);
    let decoded: GameActionTimes =
        postcard::from_bytes(&postcard::to_allocvec(&times).unwrap()).unwrap();
    assert_eq!(decoded, times);
}
