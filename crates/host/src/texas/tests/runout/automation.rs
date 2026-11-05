use super::super::{HOST_CONNECTION, connection_for};
use super::fixtures::*;
use leocard_protocol::{ServerEvent, TexasCommand, TexasPhaseView};
use std::time::Duration;

#[test]
fn bot_votes_twice_but_human_auto_play_votes_once() {
    for bot in [false, true] {
        let mut session = voting_session();
        let player = session.game().unwrap().current_player().unwrap();
        let participant = session
            .room
            .players
            .iter_mut()
            .find(|p| p.id == player)
            .unwrap();
        if bot {
            participant.is_bot = true;
        } else {
            let result = command(
                &mut session,
                20,
                player,
                TexasCommand::SetAutoPlay { enabled: true },
            );
            assert!(
                !result
                    .iter()
                    .any(|d| matches!(d.message.event, ServerEvent::Rejected { .. }))
            );
            let rejected = command(
                &mut session,
                21,
                player,
                TexasCommand::ChooseRunout { twice: true },
            );
            assert!(
                rejected
                    .iter()
                    .any(|d| matches!(d.message.event, ServerEvent::Rejected { .. }))
            );
            assert!(session.game().unwrap().game().runout_votes().is_empty());
        }
        let history_len = session.game().unwrap().game().betting_history().len();
        session.advance_time(Duration::from_secs(10));
        let snapshot = session.game_snapshot(player);
        assert_eq!(snapshot.phase, TexasPhaseView::ChoosingRunout);
        assert!(snapshot.current_player.is_some_and(|next| next != player));
        assert_eq!(snapshot.runout_votes.len(), 1);
        assert_eq!(snapshot.runout_votes[0].player, player);
        assert_eq!(snapshot.runout_votes[0].twice, bot);
        assert_eq!(
            session.game().unwrap().game().betting_history().len(),
            history_len
        );
    }
}

#[test]
fn final_bot_vote_runs_normal_hand_completion_processing() {
    for tournament_complete in [false, true] {
        let mut session = if tournament_complete {
            voting_session_with_deck(winning_deck(false))
        } else {
            voting_session()
        };
        vote(&mut session, 20, true);
        vote(&mut session, 21, true);
        let player = session.game().unwrap().current_player().unwrap();
        let participant = session
            .room
            .players
            .iter_mut()
            .find(|p| p.id == player)
            .unwrap();
        participant.is_bot = true;
        session.advance_time(Duration::from_secs(10));
        let snapshot = session.game_snapshot(player);
        assert!(matches!(
            snapshot.phase,
            TexasPhaseView::HandComplete { .. }
        ));
        assert_eq!(snapshot.second_community.len(), 5);
        assert_hand_completion(&session, tournament_complete);
        assert_eq!(snapshot.current_player, None);
        let settled = session.game().unwrap().game().clone();
        let tracked = session.match_profile_stats.clone();
        session.advance_time(Duration::from_secs(10));
        assert_eq!(session.game().unwrap().game(), &settled);
        assert_eq!(session.game_snapshot(player), snapshot);
        assert_eq!(session.match_profile_stats, tracked);
    }
}

#[test]
fn disconnected_all_in_voters_choose_once_without_folding_or_deadlock() {
    let mut session = voting_session();
    let first = session.game().unwrap().current_player().unwrap();
    // Startup chooses a random dealer. Disconnect a peer, not the host:
    // the latter intentionally closes the entire room instead of auto-voting.
    if connection_for(&session, first) == HOST_CONNECTION {
        vote(&mut session, 19, true);
    }
    let votes_before = session.game().unwrap().game().runout_votes().len();
    let player = session.game().unwrap().current_player().unwrap();
    assert_ne!(connection_for(&session, player), HOST_CONNECTION);
    let history_len = session.game().unwrap().game().betting_history().len();
    let connection = connection_for(&session, player);
    session.disconnect(connection);
    let snapshot = session.game_snapshot(player);
    assert_eq!(snapshot.runout_votes.len(), votes_before + 1);
    let disconnected_vote = snapshot.runout_votes.last().unwrap();
    assert_eq!(disconnected_vote.player, player);
    assert!(!disconnected_vote.twice);
    assert!(
        !snapshot
            .players
            .iter()
            .find(|p| p.player.id == player)
            .unwrap()
            .folded
    );
    assert_eq!(snapshot.phase, TexasPhaseView::ChoosingRunout);
    assert!(snapshot.current_player.is_some_and(|next| next != player));
    assert_eq!(
        session.game().unwrap().game().betting_history().len(),
        history_len
    );
    for request in 20..(22 - votes_before as u64) {
        vote(&mut session, request, true);
    }
    let snapshot = session.game_snapshot(player);
    assert!(matches!(
        snapshot.phase,
        TexasPhaseView::HandComplete { .. }
    ));
    assert_eq!(snapshot.current_player, None);
    assert_eq!(snapshot.runout_votes.len(), 3);
    assert!(snapshot.second_community.is_empty());
    assert_eq!(snapshot.players.iter().map(|p| p.stack).sum::<u32>(), 60);
}
