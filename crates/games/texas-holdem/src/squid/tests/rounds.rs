use crate::{PotAward, SquidState, TexasHoldemPlayerId, deepseek_squid_marker};

fn pot(winners: &[usize]) -> PotAward {
    PotAward {
        amount: 10,
        winners: winners.iter().copied().map(TexasHoldemPlayerId).collect(),
        winning_hand: None,
    }
}

#[test]
fn squid_awards_count_only_the_recipient_and_reset_with_the_pool() {
    let mut state = SquidState::new(5);
    let seed = [17; 16];
    let mut stacks = [100; 6];
    assert!(state.award_statistics(seed).is_none());
    for count in 1..=5 {
        state.start_hand();
        assert!(state.award_statistics(seed).is_none());
        let previous = state.markers.clone();
        state.finish_hand(&[pot(&[0])], &mut stacks, count);
        let drawn = previous
            .iter()
            .zip(&state.markers)
            .position(|(before, after)| before != after)
            .unwrap();
        assert!(previous[drawn].is_none());
        let (player, statistics) = state.award_statistics(seed).unwrap();
        assert_eq!(player, TexasHoldemPlayerId(0));
        assert_eq!(statistics.owned, count as u8);
        assert_eq!(
            statistics.deepseek,
            deepseek_squid_marker(seed, 1, drawn).is_some()
        );
        assert_eq!(
            usize::from(statistics.deepseek_owned),
            state
                .markers
                .iter()
                .enumerate()
                .filter(|(index, owner)| owner.is_some()
                    && deepseek_squid_marker(seed, 1, *index).is_some())
                .count()
        );
    }
    assert!(state.settled);
    state.start_hand();
    assert_eq!(state.round, 2);
    assert!(state.markers.iter().all(Option::is_none));
    assert!(state.award_statistics(seed).is_none());
    state.finish_hand(&[pot(&[1])], &mut stacks, 6);
    let drawn = state.markers.iter().position(Option::is_some).unwrap();
    let (player, statistics) = state.award_statistics(seed).unwrap();
    assert_eq!(player, TexasHoldemPlayerId(1));
    assert_eq!(statistics.owned, 1);
    assert_eq!(
        statistics.deepseek_owned,
        u8::from(deepseek_squid_marker(seed, 2, drawn).is_some())
    );

    state.start_hand();
    state.finish_hand(&[pot(&[0, 1]), pot(&[0])], &mut stacks, 7);
    assert!(state.award_statistics(seed).is_none());
    assert_eq!(state.markers.iter().flatten().count(), 1);
    let mut disabled = SquidState::new(0);
    disabled.finish_hand(&[pot(&[0])], &mut stacks, 8);
    assert!(disabled.award_statistics(seed).is_none());
}

#[test]
fn draws_can_select_every_remaining_marker_and_preserve_existing_owners() {
    let mut pool = SquidState::new(5);
    pool.markers[1] = Some(TexasHoldemPlayerId(1));
    pool.markers[3] = Some(TexasHoldemPlayerId(2));
    let mut draws = [0; 5];
    for seed in 0..128 {
        let mut state = pool.clone();
        let mut stacks = [100; 6];
        state.finish_hand(&[pot(&[0])], &mut stacks, seed);
        let drawn = state
            .markers
            .iter()
            .position(|owner| *owner == Some(TexasHoldemPlayerId(0)))
            .unwrap();
        draws[drawn] += 1;
        assert_eq!(state.markers[1], pool.markers[1]);
        assert_eq!(state.markers[3], pool.markers[3]);
        assert_eq!(state.markers.iter().flatten().count(), 3);
        let mut replay = pool.clone();
        replay.finish_hand(&[pot(&[0])], &mut [100; 6], seed);
        assert_eq!(state, replay);
    }
    assert_eq!(draws[1], 0);
    assert_eq!(draws[3], 0);
    assert!(draws[0] > 0 && draws[2] > 0 && draws[4] > 0);
}
