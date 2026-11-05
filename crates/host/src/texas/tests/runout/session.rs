use super::super::{
    HOST_CONNECTION, ROOM, SECOND_CONNECTION, THIRD_CONNECTION, connection_for, join_command,
    message,
};
use super::fixtures::*;
use crate::texas::TexasSession;
use leocard_protocol::{
    ClientCommand, GameEvent, GameRules, GameViolation, RejectReason, SeatId, ServerEvent,
    TexasCommand, TexasEvent, TexasPhaseView, TexasViolation,
};
use leocard_texas_holdem::{TexasAction, TexasRuleSet, build_deck};

#[test]
fn room_rules_broadcast_both_toggles_and_reject_incompatible_options_atomically() {
    let mut session =
        TexasSession::new(ROOM, 52300, TexasRuleSet::default(), build_deck(false)).unwrap();
    session.handle(HOST_CONNECTION, message(1, join_command("host", 1)));
    session.handle(SECOND_CONNECTION, message(1, join_command("peer", 2)));
    let deliveries = session.handle(
        HOST_CONNECTION,
        message(
            2,
            ClientCommand::Game(TexasCommand::UpdateRules { rules: rules() }.into()),
        ),
    );
    for connection in [HOST_CONNECTION, SECOND_CONNECTION] {
        assert!(deliveries.iter().any(|d| d.recipient == connection && matches!(&d.message.event,
            ServerEvent::LobbySnapshot(s) if matches!(s.rules, GameRules::Texas(r) if !r.side_pots && r.run_it_twice))));
    }
    let revision = session.revision();
    let invalid = TexasRuleSet {
        side_pots: true,
        ..rules()
    };
    let deliveries = session.handle(
        HOST_CONNECTION,
        message(
            3,
            ClientCommand::Game(TexasCommand::UpdateRules { rules: invalid }.into()),
        ),
    );
    assert!(
        deliveries
            .iter()
            .any(|d| matches!(d.message.event, ServerEvent::Rejected { .. }))
    );
    assert_eq!(session.revision(), revision);
    assert!(!session.rules().side_pots);
    assert!(session.rules().run_it_twice);
}

#[test]
fn final_manual_vote_broadcasts_completion_once_and_replay_is_atomic() {
    for tournament_complete in [false, true] {
        let mut session = if tournament_complete {
            voting_session_with_deck(winning_deck(false))
        } else {
            voting_session()
        };
        vote(&mut session, 20, true);
        vote(&mut session, 21, true);
        let player = session.game().unwrap().current_player().unwrap();
        let connection = connection_for(&session, player);
        let deliveries = vote(&mut session, 22, true);
        let events = deliveries
            .iter()
            .filter(|d| d.recipient == connection)
            .filter_map(|d| {
                if let ServerEvent::GameEvent(GameEvent::Texas(event)) = &d.message.event {
                    Some(event)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        assert_eq!(
            events
                .iter()
                .filter(|e| matches!(
                    e,
                    TexasEvent::RunoutChosen {
                        next_player: None,
                        ..
                    }
                ))
                .count(),
            1
        );
        assert_eq!(
            events
                .iter()
                .filter(|e| matches!(e, TexasEvent::HandAnalyzed { .. }))
                .count(),
            1
        );
        assert_eq!(
            events
                .iter()
                .filter(|e| matches!(e, TexasEvent::HandFinished { .. }))
                .count(),
            1
        );
        let snapshot = session.game_snapshot(player);
        assert!(matches!(
            snapshot.phase,
            TexasPhaseView::HandComplete { .. }
        ));
        assert_hand_completion(&session, tournament_complete);
        let tracked = session.match_profile_stats.clone();
        let before = session.game().unwrap().game().clone();
        let revision = session.revision();
        let deliveries = command(
            &mut session,
            22,
            player,
            TexasCommand::ChooseRunout { twice: false },
        );
        assert!(
            deliveries
                .iter()
                .any(|d| matches!(d.message.event, ServerEvent::Rejected { .. }))
        );
        assert_eq!(session.game().unwrap().game(), &before);
        assert_eq!(session.revision(), revision);
        assert_eq!(session.game_snapshot(player), snapshot);
        assert_eq!(session.match_profile_stats, tracked);
    }
}

#[test]
fn room_seats_map_connections_and_refunds_in_every_table_order() {
    let connections = [HOST_CONNECTION, SECOND_CONNECTION, THIRD_CONNECTION];
    for (seats, order) in [
        ([1, 4, 7], [0, 1, 2]),
        ([1, 7, 4], [0, 2, 1]),
        ([4, 1, 7], [1, 0, 2]),
        ([4, 7, 1], [2, 0, 1]),
        ([7, 1, 4], [1, 2, 0]),
        ([7, 4, 1], [2, 1, 0]),
    ] {
        let mut session = session_with_seats(seats.map(SeatId));
        let expected = order.map(|index| session.room.player_id(connections[index]).unwrap());
        let game = session.game().unwrap();
        assert_eq!(
            game.players().iter().map(|p| p.id).collect::<Vec<_>>(),
            expected
        );
        assert_eq!(
            game.players().iter().map(|p| p.seat).collect::<Vec<_>>(),
            [SeatId(1), SeatId(4), SeatId(7)]
        );
        for (index, player) in game.players().iter().enumerate() {
            assert_eq!(
                connection_for(&session, player.id),
                connections[order[index]]
            );
        }
        let dealer = game.game().dealer().index();
        let clockwise = (0..3)
            .map(|offset| expected[(dealer + offset) % 3])
            .collect::<Vec<_>>();
        unequal_stacks(session.game.as_mut().unwrap());
        for action in [TexasAction::AllIn, TexasAction::AllIn, TexasAction::Call] {
            let game = session.game.as_mut().unwrap();
            game.act(game.current_player().unwrap(), action).unwrap();
        }
        let snapshot = session.game_snapshot(clockwise[0]);
        assert_eq!(snapshot.phase, TexasPhaseView::ChoosingRunout);
        assert_eq!(snapshot.current_player, Some(clockwise[2]));
        assert_eq!(snapshot.pot, 42);
        for (player, stack) in clockwise.iter().zip([13, 5, 0]) {
            let view = snapshot
                .players
                .iter()
                .find(|p| p.player.id == *player)
                .unwrap();
            assert_eq!(view.stack, stack);
            assert_eq!(view.committed_total, 14);
        }
    }
}

#[test]
fn refunded_all_in_vote_preserves_turn_and_state_after_invalid_actions() {
    let mut session = session();
    unequal_stacks(session.game.as_mut().unwrap());
    for (request, action) in [
        (10, TexasAction::AllIn),
        (11, TexasAction::AllIn),
        (12, TexasAction::Call),
    ] {
        let player = session.game().unwrap().current_player().unwrap();
        let deliveries = command(&mut session, request, player, TexasCommand::Act { action });
        assert!(
            !deliveries
                .iter()
                .any(|d| matches!(d.message.event, ServerEvent::Rejected { .. }))
        );
    }
    // The first voter is the short all-in (0 chips); next is the refunded long caller.
    let first = session.game().unwrap().current_player().unwrap();
    vote(&mut session, 20, true);
    let voter = session.game().unwrap().current_player().unwrap();
    let view = session.game_snapshot(voter);
    assert_eq!(view.phase, TexasPhaseView::ChoosingRunout);
    assert_eq!(view.current_player, Some(voter));
    assert_eq!(view.runout_votes.len(), 1);
    assert_eq!(view.pot, 42);
    assert!(
        view.players
            .iter()
            .find(|p| p.player.id == voter)
            .unwrap()
            .stack
            > 0
    );
    let last = view
        .players
        .iter()
        .find(|p| p.player.id != voter && p.player.id != first)
        .unwrap()
        .player
        .id;
    let before = session.game().unwrap().game().clone();
    let tracked = session.match_profile_stats.clone();
    let revision = session.revision();
    let history_len = before.betting_history().len();
    for (request, player, invalid, violation) in [
        (
            21,
            last,
            TexasCommand::ChooseRunout { twice: true },
            TexasViolation::NotPlayersTurn,
        ),
        (
            22,
            voter,
            TexasCommand::Act {
                action: TexasAction::Fold,
            },
            TexasViolation::MustChooseRunout,
        ),
        (
            23,
            first,
            TexasCommand::ChooseRunout { twice: false },
            TexasViolation::RunoutAlreadyChosen,
        ),
    ] {
        let rejected = command(&mut session, request, player, invalid);
        assert!(rejected.iter().any(|d| matches!(
            &d.message.event,
            ServerEvent::Rejected {
                reason: RejectReason::Game(GameViolation::Texas(actual)),
            } if actual == &violation
        )));
        assert_eq!(session.game().unwrap().game(), &before);
        assert_eq!(session.game_snapshot(voter), view);
        assert_eq!(session.revision(), revision);
        assert_eq!(session.match_profile_stats, tracked);
    }
    vote(&mut session, 24, true);
    let view = session.game_snapshot(voter);
    assert_eq!(view.phase, TexasPhaseView::ChoosingRunout);
    assert_eq!(view.current_player, Some(last));
    assert_eq!(view.runout_votes.len(), 2);
    vote(&mut session, 25, true);
    let view = session.game_snapshot(voter);
    let TexasPhaseView::HandComplete { awards, .. } = view.phase else {
        panic!("the last vote must settle both boards");
    };
    assert_eq!(view.current_player, None);
    assert_eq!(view.runout_votes.len(), 3);
    assert_eq!(view.second_community.len(), 5);
    assert_eq!(awards.iter().map(|award| award.amount).sum::<u32>(), 42);
    assert_eq!(view.players.iter().map(|p| p.stack).sum::<u32>(), 60);
    assert_eq!(
        session.game().unwrap().game().betting_history().len(),
        history_len
    );
}
