use super::*;

#[test]
fn distinct_scores_follow_every_configured_tier() {
    assert_eq!(reference_point_deltas(&[20, 10]), Some(vec![2, -2]));
    assert_eq!(reference_point_deltas(&[30, 20, 10]), Some(vec![2, 0, -2]));
    assert_eq!(
        reference_point_deltas(&[40, 30, 20, 10]),
        Some(vec![3, 1, -1, -3])
    );
    assert_eq!(
        reference_point_deltas(&[50, 40, 30, 20, 10]),
        Some(vec![3, 1, 0, -1, -3])
    );
    assert_eq!(
        reference_point_deltas(&[60, 50, 40, 30, 20, 10]),
        Some(vec![5, 3, 1, -1, -3, -5])
    );
}

#[test]
fn ties_take_the_higher_tier_and_keep_original_player_order() {
    assert_eq!(
        reference_point_deltas(&[10, 30, 20, 20]),
        Some(vec![-3, 3, 1, 1])
    );
    assert_eq!(reference_point_deltas(&[20, 20]), Some(vec![2, 2]));
}
