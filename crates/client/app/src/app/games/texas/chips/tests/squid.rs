use super::super::TexasChipTableState;
use super::pot_test_snapshot;
use leocard_game_common::PlayerId;
use leocard_protocol::{TexasEvent, TexasPhaseView, TexasPotAwardView};
use leocard_texas_holdem::TexasSquidTransfer;

#[test]
fn squid_payments_preserve_visual_balances_on_settlement_refresh_and_reconnect() {
    let mut snapshot =
        pot_test_snapshot(&[(0, false, false), (1, false, false), (2, false, false)]);
    snapshot.squid.round = 1;
    snapshot.squid.markers = vec![None];
    let mut state = TexasChipTableState::default();
    state.initialize(&snapshot);
    snapshot.squid.markers[0] = Some(PlayerId(2));
    snapshot.squid.settled = true;
    snapshot.squid.transfers = vec![
        TexasSquidTransfer {
            from: PlayerId(0),
            to: PlayerId(2),
            amount: 2,
        },
        TexasSquidTransfer {
            from: PlayerId(1),
            to: PlayerId(2),
            amount: 2,
        },
    ];
    snapshot.current_player = None;
    for (player, stack) in snapshot.players.iter_mut().zip([18, 17, 25]) {
        player.stack = stack;
    }
    snapshot.phase = TexasPhaseView::HandComplete {
        showdown: false,
        awards: vec![TexasPotAwardView {
            amount: 3,
            winners: vec![PlayerId(2)],
            winning_category: None,
        }],
        table_winner: None,
        tournament_complete: false,
        reference_changes: Vec::new(),
    };
    state.observe(
        &snapshot,
        vec![TexasEvent::HandFinished { showdown: false }],
    );
    for refresh in 0..3 {
        if refresh == 1 {
            state.observe(&snapshot, Vec::new());
        }
        if refresh == 2 {
            state = TexasChipTableState::default();
            state.initialize(&snapshot);
        }
        for player in &snapshot.players {
            let visual: u32 = state
                .stack_counts(player.player.id)
                .iter()
                .map(|&(value, count)| u32::from(value) * count as u32)
                .sum();
            assert_eq!(visual, player.stack);
        }
    }
}
