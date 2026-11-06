use super::super::{ChipZone, TexasChipTableState};
use super::pot_test_snapshot;
use leocard_game_common::PlayerId;
use leocard_protocol::{TexasEvent, TexasPhaseView, TexasPotAwardView, TexasSnapshot};
use leocard_texas_holdem::{TexasCard, TexasRank, TexasSuit};

fn value(state: &TexasChipTableState, zone: ChipZone) -> u32 {
    state
        .chips
        .iter()
        .filter(|chip| chip.zone == zone)
        .map(|chip| u32::from(chip.denomination))
        .sum()
}

fn assert_balances(state: &TexasChipTableState, snapshot: &TexasSnapshot) {
    for player in &snapshot.players {
        assert_eq!(
            value(state, ChipZone::Stack(player.player.id)),
            player.stack
        );
        assert_eq!(value(state, ChipZone::Bet(player.player.id)), 0);
    }
    let total = state
        .chips
        .iter()
        .filter(|chip| chip.zone != ChipZone::Retired)
        .map(|chip| u32::from(chip.denomination))
        .sum::<u32>();
    assert_eq!(total, 66);
}

#[test]
fn refunds_and_flattened_runs_conserve_money_batched_or_sequentially() {
    for batched in [false, true] {
        let mut snapshot = pot_test_snapshot(&[
            (20, false, false),
            (6, true, false),
            (13, false, true),
            (12, false, false),
        ]);
        for (player, street) in snapshot.players.iter_mut().zip([3, 6, 4, 2]) {
            player.committed_street = street;
        }
        let mut state = TexasChipTableState::default();
        state.initialize(&snapshot, false);
        assert_eq!(state.pots.len(), 1);
        let mut events = vec![
            TexasEvent::ChipsReturned {
                player: PlayerId(0),
                amount: 14,
            },
            TexasEvent::ChipsReturned {
                player: PlayerId(3),
                amount: 6,
            },
            TexasEvent::RunoutVoteStarted {
                current_player: PlayerId(1),
            },
        ];
        for (index, stack) in [(0, 14), (3, 14)] {
            snapshot.players[index].stack = stack;
            snapshot.players[index].committed_total = 6;
            snapshot.players[index].committed_street = 0;
        }
        snapshot.pot = 31; // Folded money remains in the single pot.
        snapshot.phase = TexasPhaseView::ChoosingRunout;
        snapshot.current_player = Some(PlayerId(1));
        if !batched {
            state.observe(&snapshot, events, false);
            assert_balances(&state, &snapshot);
            assert_eq!(value(&state, ChipZone::Pot(0)), 31);
            state.observe(&snapshot, Vec::new(), false);
            assert_balances(&state, &snapshot);
            let mut reconnected = TexasChipTableState::default();
            reconnected.initialize(&snapshot, false);
            assert_balances(&reconnected, &snapshot);
            assert_eq!(value(&reconnected, ChipZone::Pot(0)), 31);
            events = Vec::new();
        }
        snapshot.second_community = vec![TexasCard::new(TexasSuit::Heart, TexasRank::Ace)];
        snapshot.phase = TexasPhaseView::HandComplete {
            showdown: true,
            awards: vec![
                TexasPotAwardView {
                    run: 1,
                    amount: 16,
                    winners: vec![PlayerId(0), PlayerId(1)],
                    winning_category: None,
                },
                TexasPotAwardView {
                    run: 2,
                    amount: 15,
                    winners: vec![PlayerId(3)],
                    winning_category: None,
                },
            ],
            table_winner: None,
            tournament_complete: false,
            reference_changes: Vec::new(),
        };
        snapshot.current_player = None;
        for (player, stack) in snapshot.players.iter_mut().zip([22, 8, 7, 29]) {
            player.stack = stack;
        }
        events.push(TexasEvent::HandFinished { showdown: true });
        state.observe(&snapshot, events, false);
        for refresh in 0..3 {
            if refresh == 1 {
                state.observe(&snapshot, Vec::new(), false);
            } else if refresh == 2 {
                state.initialize(&snapshot, false);
            }
            assert_balances(&state, &snapshot);
            assert_eq!(value(&state, ChipZone::Pot(0)), 0);
        }
    }
}
