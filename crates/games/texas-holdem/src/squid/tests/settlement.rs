use crate::{SquidState, TexasHoldemPlayerId, deepseek_squid_marker};

fn mixed_pool(double: bool, round: u32) -> SquidState {
    let mut state = SquidState::new(5, [17; 16], double);
    state.round = round;
    for (index, owner) in state.markers.iter_mut().enumerate() {
        *owner = Some(TexasHoldemPlayerId(usize::from(
            deepseek_squid_marker([17; 16], round, index).is_some(),
        )));
    }
    assert!(state.markers.contains(&Some(TexasHoldemPlayerId(0))));
    assert!(state.markers.contains(&Some(TexasHoldemPlayerId(1))));
    state
}

#[test]
fn deepseek_markers_double_payments_and_receipts_only_when_enabled() {
    for double in [false, true] {
        for round in [1, 2] {
            let mut state = mixed_pool(double, round);
            let ordinary = state
                .markers
                .iter()
                .filter(|owner| **owner == Some(TexasHoldemPlayerId(0)))
                .count() as u32;
            let deepseek = 5 - ordinary;
            let deepseek_payment = deepseek * if double { 4 } else { 2 };
            let ordinary_payment = ordinary * 2;
            let total = ordinary_payment + deepseek_payment;
            let mut stacks = [100; 3];
            state.finish_hand(&[], &mut stacks, 0);
            assert!(state.settled);
            assert_eq!(
                stacks,
                [100 + ordinary_payment, 100 + deepseek_payment, 100 - total]
            );
            assert_eq!(stacks.iter().sum::<u32>(), 300);
            let settled = stacks;
            state.finish_hand(&[], &mut stacks, 0);
            assert_eq!(stacks, settled);
        }
    }
}

#[test]
fn short_stacks_pay_weighted_shares_without_debt_or_lost_chips() {
    let mut state = mixed_pool(true, 1);
    let ordinary = state
        .markers
        .iter()
        .filter(|owner| **owner == Some(TexasHoldemPlayerId(0)))
        .count() as u32;
    let deepseek_weight = (5 - ordinary) * 2;
    let total_weight = ordinary + deepseek_weight;
    let mut stacks = [100, 100, 3];
    state.finish_hand(&[], &mut stacks, 0);
    assert_eq!(stacks[2], 0);
    assert_eq!(stacks.iter().sum::<u32>(), 203);
    for (index, weight) in [ordinary, deepseek_weight].into_iter().enumerate() {
        let share = stacks[index] - 100;
        assert!(share == 3 * weight / total_weight || share == (3 * weight).div_ceil(total_weight));
    }
    state.start_hand();
    assert_eq!(state.round, 2);
    assert!(state.markers.iter().all(Option::is_none));
}

#[test]
fn early_settlement_only_charges_for_markers_already_awarded() {
    let mut state = SquidState::new(5, [17; 16], true);
    let index = (0..5)
        .find(|&index| deepseek_squid_marker([17; 16], 1, index).is_some())
        .unwrap();
    state.markers[index] = Some(TexasHoldemPlayerId(0));
    let mut stacks = [100, 100, 0];
    state.finish_hand(&[], &mut stacks, 0);
    assert_eq!(stacks, [104, 96, 0]);
    let mut disabled = SquidState::new(0, [17; 16], true);
    let mut stacks = [100; 3];
    disabled.finish_hand(&[], &mut stacks, 0);
    assert_eq!(stacks, [100; 3]);
}
