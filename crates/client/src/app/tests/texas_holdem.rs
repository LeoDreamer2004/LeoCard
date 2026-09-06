use super::*;

#[test]
fn held_raise_adjustment_stops_exactly_at_both_boundaries() {
    assert_eq!(texas_raise_repeat_value(6, -1, 4, 5, 20), 5);
    assert_eq!(texas_raise_repeat_value(19, 1, 4, 5, 20), 20);
    assert_eq!(texas_raise_repeat_value(12, -1, 3, 5, 20), 9);
    assert_eq!(texas_raise_repeat_value(12, 1, 3, 5, 20), 15);
}
