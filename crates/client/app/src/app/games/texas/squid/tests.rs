use super::super::chips::tests::pot_test_snapshot;
use super::state::SquidTableState;
use leocard_game_common::PlayerId;

fn assert_same_pool(left: &SquidTableState, right: &SquidTableState) {
    assert_eq!(left.markers.len(), right.markers.len());
    for (left, right) in left.markers.iter().zip(&right.markers) {
        assert_eq!(left.deepseek, right.deepseek);
        assert!(left.position.abs_diff_eq(right.position, 0.0001));
        assert!((left.rotation - right.rotation).abs() < 0.0001);
    }
}

#[test]
fn every_completed_round_regenerates_the_central_pool_and_matches_reconnecting_viewers() {
    let mut snapshot = pot_test_snapshot(&[(0, false, false); 6]);
    snapshot.squid.round = 1;
    snapshot.squid.markers = vec![None; 5];
    let mut state = SquidTableState::default();
    state.observe(&snapshot);
    for round in 2..=10 {
        let appearance: Vec<_> = state.markers.iter().map(|marker| marker.deepseek).collect();
        snapshot.squid.markers.fill(Some(PlayerId(0)));
        state.observe(&snapshot);
        state.tick(1.0);
        assert_eq!(
            appearance,
            state
                .markers
                .iter()
                .map(|marker| marker.deepseek)
                .collect::<Vec<_>>()
        );
        // The next hand starts the next squid round after all markers were distributed.
        snapshot.hand_number += 1;
        snapshot.squid.round = round;
        snapshot.squid.markers.fill(None);
        state.observe(&snapshot);
        state.tick(0.31);
        let mut restored = SquidTableState::default();
        restored.observe(&snapshot);
        assert!(
            state
                .markers
                .iter()
                .zip(&restored.markers)
                .any(|(moving, central)| !moving.position.abs_diff_eq(central.position, 0.001))
        );
        state.tick(0.5);
        assert_same_pool(&state, &restored);
        state.observe(&snapshot);
        state.tick(0.1);
        assert_same_pool(&state, &restored);
        let mut other_view = snapshot.clone();
        other_view.you = PlayerId(1);
        let mut other_player = SquidTableState::default();
        other_player.observe(&other_view);
        assert_same_pool(&state, &other_player);
    }
}
