use super::super::client::DrawGuessClient;
use crate::app::runtime::GameProjection;
use leocard_draw_guess::{
    CanvasPoint, DrawAction, DrawGuessGame, DrawGuessRuleSet, Ink, word_bank,
};
use leocard_game_common::PlayerId;
use leocard_protocol::{DrawGuessEvent, DrawGuessSnapshot, MatchId, ServerEvent};

pub(super) fn snapshot() -> DrawGuessSnapshot {
    let mut game = DrawGuessGame::new(
        DrawGuessRuleSet::default(),
        vec![PlayerId(0), PlayerId(1)],
        word_bank(),
    )
    .unwrap();
    game.choose_word(PlayerId(0), 0).unwrap();
    DrawGuessSnapshot {
        match_id: MatchId([23; 16]),
        host_port: 52300,
        you: PlayerId(0),
        host: PlayerId(0),
        rules: DrawGuessRuleSet::default(),
        players: Vec::new(),
        state: game.view(PlayerId(0)),
        match_complete: false,
        reference_changes: Vec::new(),
    }
}

fn actions() -> [DrawAction; 3] {
    [
        DrawAction::Begin {
            point: CanvasPoint { x: 40, y: 30 },
            color: Ink::BLACK,
            size: 4,
            eraser: false,
        },
        DrawAction::Extend {
            points: vec![CanvasPoint { x: 60, y: 80 }],
        },
        DrawAction::End,
    ]
}

#[test]
fn echoed_predictions_are_reconciled_once_and_other_rounds_are_ignored() {
    let snapshot = snapshot();
    let mut client = DrawGuessClient::default();
    client.apply(&ServerEvent::GameSnapshot(snapshot.clone().into()));
    let actions = actions();
    for action in &actions {
        assert!(client.predict(action.clone()));
    }
    let predicted = client.predicted();
    for (index, action) in actions.into_iter().enumerate() {
        let event = ServerEvent::GameEvent(
            DrawGuessEvent {
                match_id: snapshot.match_id,
                turn: 0,
                revision: index as u64 + 1,
                action,
            }
            .into(),
        );
        client.apply(&event);
        client.apply(&event);
        assert_eq!(client.predicted(), predicted);
    }
    assert!(client.pending.is_empty());
    client.apply(&ServerEvent::GameEvent(
        DrawGuessEvent {
            match_id: snapshot.match_id,
            turn: 1,
            revision: 4,
            action: DrawAction::Clear,
        }
        .into(),
    ));
    assert_eq!(client.predicted(), predicted);
}

#[test]
fn revision_gaps_request_recovery_and_snapshot_discards_unconfirmed_prediction() {
    let mut snapshot = snapshot();
    let mut client = DrawGuessClient::default();
    client.apply(&ServerEvent::GameSnapshot(snapshot.clone().into()));
    assert!(client.predict(actions()[0].clone()));
    client.apply(&ServerEvent::GameEvent(
        DrawGuessEvent {
            match_id: snapshot.match_id,
            turn: 0,
            revision: 3,
            action: DrawAction::End,
        }
        .into(),
    ));
    assert!(client.needs_snapshot);
    for action in actions() {
        snapshot.state.canvas.apply(&action).unwrap();
    }
    client.apply(&ServerEvent::GameSnapshot(snapshot.clone().into()));
    assert!(!client.needs_snapshot);
    assert!(client.pending.is_empty());
    assert_eq!(client.predicted(), snapshot.state.canvas);
    assert!(client.predict(DrawAction::Undo));
    snapshot.state.turn = 1;
    snapshot.state.canvas = Default::default();
    client.apply(&ServerEvent::GameSnapshot(snapshot.clone().into()));
    assert!(client.pending.is_empty());
    assert_eq!(client.predicted(), snapshot.state.canvas);
}
