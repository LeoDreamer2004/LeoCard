use super::{ActionClock, ActionTimeStats, ActionTimingSnapshot, GameActionTimes};
use crate::{GameKind, MatchId};
use leocard_game_common::PlayerId;

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

#[test]
fn overflowing_totals_preserve_weighted_counters_but_still_update_maximum() {
    let mut times = GameActionTimes::default();
    times.record(GameKind::Texas, u64::MAX - 1);
    times.record(GameKind::Texas, u64::MAX);
    assert_eq!(
        times.get(GameKind::Texas),
        ActionTimeStats {
            total_millis: u64::MAX - 1,
            completed_actions: 1,
            longest_millis: u64::MAX,
        }
    );
    let decoded: GameActionTimes =
        crate::decode_frame(&crate::encode_frame(&times).unwrap()).unwrap();
    assert_eq!(decoded, times);
    assert_eq!(decoded.longest_millis(), u64::MAX);
}

#[test]
fn public_clocks_serialize_only_identity_scope_and_elapsed_time() {
    let timing = ActionTimingSnapshot {
        game: GameKind::Guandan,
        match_id: MatchId([19; 16]),
        clocks: vec![ActionClock {
            player: PlayerId(2),
            elapsed_millis: u64::MAX,
        }],
    };
    let decoded: ActionTimingSnapshot =
        postcard::from_bytes(&postcard::to_allocvec(&timing).unwrap()).unwrap();
    assert_eq!(decoded, timing);
}
