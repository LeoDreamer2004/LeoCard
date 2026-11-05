use super::clock::ActionTimingClock;
use leocard_game_common::PlayerId;
use leocard_protocol::{ActionClock, ActionTimingSnapshot, GameKind, MatchId, RoomId};
use std::time::Duration;

#[test]
fn interpolate_multiple_actors_recalibrate_and_clear() {
    let mut clock = ActionTimingClock::default();
    let mut snapshot = ActionTimingSnapshot {
        game: GameKind::Mahjong,
        match_id: MatchId([1; 16]),
        clocks: vec![
            ActionClock {
                player: PlayerId(1),
                elapsed_millis: 1_200,
            },
            ActionClock {
                player: PlayerId(2),
                elapsed_millis: 700,
            },
        ],
    };
    clock.synchronize(RoomId(1), 1, &snapshot, Duration::from_secs(10));
    // Re-reading the same model every frame must not move the interpolation anchor.
    clock.synchronize(RoomId(1), 1, &snapshot, Duration::from_millis(10_400));
    assert_eq!(
        clock.elapsed(PlayerId(1), Duration::from_millis(10_500)),
        Some(Duration::from_millis(1_700))
    );
    assert_eq!(
        clock.elapsed(PlayerId(2), Duration::from_millis(10_500)),
        Some(Duration::from_millis(1_200))
    );
    assert_eq!(
        clock.elapsed(PlayerId(3), Duration::from_millis(10_500)),
        None
    );
    // Identical payload, new accepted heartbeat: authority resets the local anchor.
    clock.synchronize(RoomId(1), 2, &snapshot, Duration::from_secs(11));
    assert_eq!(
        clock.elapsed(PlayerId(1), Duration::from_millis(11_100)),
        Some(Duration::from_millis(1_300))
    );
    snapshot.clocks[0].elapsed_millis = 2_400;
    clock.synchronize(RoomId(1), 3, &snapshot, Duration::from_secs(12));
    assert_eq!(
        clock.elapsed(PlayerId(1), Duration::from_millis(12_100)),
        Some(Duration::from_millis(2_500))
    );
    snapshot.clocks.clear();
    clock.synchronize(RoomId(1), 4, &snapshot, Duration::from_secs(13));
    assert_eq!(clock.elapsed(PlayerId(1), Duration::from_secs(14)), None);
    snapshot.clocks.push(ActionClock {
        player: PlayerId(1),
        elapsed_millis: 0,
    });
    clock.synchronize(RoomId(1), 5, &snapshot, Duration::from_secs(15));
    clock.clear();
    assert_eq!(clock.elapsed(PlayerId(1), Duration::from_secs(16)), None);
}
