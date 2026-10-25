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
    for index in 0..5 {
        state.start_hand();
        assert!(state.award_statistics(seed).is_none());
        state.finish_hand(&[pot(&[0])], &mut stacks);
        let (player, statistics) = state.award_statistics(seed).unwrap();
        assert_eq!(player, TexasHoldemPlayerId(0));
        assert_eq!(statistics.owned, index as u8 + 1);
        assert_eq!(
            statistics.deepseek,
            deepseek_squid_marker(seed, 1, index).is_some()
        );
        assert_eq!(
            usize::from(statistics.deepseek_owned),
            (0..=index)
                .filter(|&i| deepseek_squid_marker(seed, 1, i).is_some())
                .count()
        );
    }
    assert!(state.settled);
    state.start_hand();
    assert_eq!(state.round, 2);
    assert!(state.markers.iter().all(Option::is_none));
    assert!(state.award_statistics(seed).is_none());
    state.finish_hand(&[pot(&[1])], &mut stacks);
    let (player, statistics) = state.award_statistics(seed).unwrap();
    assert_eq!(player, TexasHoldemPlayerId(1));
    assert_eq!(statistics.owned, 1);
    assert_eq!(
        statistics.deepseek_owned,
        u8::from(deepseek_squid_marker(seed, 2, 0).is_some())
    );

    state.start_hand();
    state.finish_hand(&[pot(&[0, 1]), pot(&[0])], &mut stacks);
    assert!(state.award_statistics(seed).is_none());
    assert_eq!(state.markers.iter().flatten().count(), 1);
    let mut disabled = SquidState::new(0);
    disabled.finish_hand(&[pot(&[0])], &mut stacks);
    assert!(disabled.award_statistics(seed).is_none());
}
