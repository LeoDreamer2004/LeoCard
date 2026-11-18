use super::DrawGuessSession;
use crate::features::GameSnapshotView;
use crate::{ConnectionId, Delivery};
use ed25519_dalek::{Signer, SigningKey};
use leocard_draw_guess::{CanvasPoint, DrawAction, DrawGuessPhase, DrawGuessRuleSet, Ink};
use leocard_protocol::{
    ClientCommand, ClientMessage, DrawGuessCommand, GameEvent, GameSnapshot, JoinRequest,
    PlayerGameProfiles, ProfileId, ReconnectToken, RequestId, RoomId, SeatId, ServerEvent,
    ServerMessage, decode_frame, encode_frame,
};
use std::time::Duration;

mod bots;
mod messaging;
mod settlement;

fn send(session: &mut DrawGuessSession, connection: u64, command: ClientCommand) -> Vec<Delivery> {
    let connection = ConnectionId(connection);
    let request = session
        .room
        .last_requests
        .get(&connection)
        .map_or(1, |request| request.0 + 1);
    session.handle(
        connection,
        ClientMessage::new(RoomId(99), RequestId(request), command),
    )
}

fn command(
    session: &mut DrawGuessSession,
    connection: u64,
    command: DrawGuessCommand,
) -> Vec<Delivery> {
    send(session, connection, ClientCommand::Game(command.into()))
}

fn join(session: &mut DrawGuessSession, connection: u64) {
    let key = SigningKey::from_bytes(&[connection as u8; 32]);
    let mut request = JoinRequest {
        name: format!("玩家{connection}"),
        reconnect_token: ReconnectToken(connection),
        profile_id: ProfileId(key.verifying_key().to_bytes()),
        reference_points: 0,
        completed_games: 0,
        game_profiles: PlayerGameProfiles::default(),
        identity_signature: Vec::new(),
        avatar_png: None,
    };
    request.identity_signature = key
        .sign(&request.identity_payload(RoomId(99)))
        .to_bytes()
        .to_vec();
    send(session, connection, ClientCommand::join(request));
    send(
        session,
        connection,
        ClientCommand::SelectSeat {
            seat: SeatId(connection as u8 - 1),
        },
    );
    send(session, connection, ClientCommand::SetReady { ready: true });
}

fn started() -> DrawGuessSession {
    let mut session = DrawGuessSession::new(
        RoomId(99),
        52300,
        DrawGuessRuleSet {
            rounds: 1,
            drawing_seconds: 30,
        },
    )
    .unwrap();
    join(&mut session, 1);
    join(&mut session, 2);
    send(&mut session, 1, ClientCommand::StartGame);
    session
}

#[test]
fn live_canvas_uses_deltas_and_private_words_stay_out_of_other_views() {
    let mut session = started();
    let artist = session.order[0];
    let guesser = session.order[1];
    assert_eq!(session.game_view(artist).state.choices.len(), 3);
    assert!(session.game_view(guesser).state.choices.is_empty());
    command(
        &mut session,
        1,
        DrawGuessCommand::ChooseWord { turn: 0, choice: 0 },
    );
    assert!(session.game_view(artist).state.word.is_some());
    assert!(session.game_view(guesser).state.word.is_none());
    let replies = command(
        &mut session,
        1,
        DrawGuessCommand::Draw {
            turn: 0,
            action: DrawAction::Begin {
                point: CanvasPoint { x: 20, y: 20 },
                color: Ink::BLACK,
                size: 8,
                eraser: false,
            },
        },
    );
    assert_eq!(replies.len(), 2);
    assert!(replies.iter().all(|delivery| matches!(&delivery.message.event, ServerEvent::GameEvent(GameEvent::DrawGuess(event)) if event.revision == 1)));
    let encoded = encode_frame(&replies[0].message).unwrap();
    assert_eq!(
        decode_frame::<ServerMessage>(&encoded).unwrap(),
        replies[0].message
    );
    let revision = session.game.as_ref().unwrap().canvas().revision();
    let rejected = command(
        &mut session,
        2,
        DrawGuessCommand::Draw {
            turn: 0,
            action: DrawAction::Clear,
        },
    );
    assert!(
        rejected
            .iter()
            .any(|delivery| matches!(delivery.message.event, ServerEvent::Rejected { .. }))
    );
    assert_eq!(session.game.as_ref().unwrap().canvas().revision(), revision);
    command(
        &mut session,
        1,
        DrawGuessCommand::Draw {
            turn: 0,
            action: DrawAction::End,
        },
    );
    join(&mut session, 3);
    let replies = send(
        &mut session,
        3,
        ClientCommand::SpectatePlayer { player: artist },
    );
    let view = replies
        .iter()
        .find_map(|delivery| match &delivery.message.event {
            ServerEvent::GameSnapshot(GameSnapshot::DrawGuess(view))
                if delivery.recipient == ConnectionId(3) =>
            {
                Some(view)
            }
            _ => None,
        })
        .unwrap();
    assert!(view.state.word.is_none());
    assert!(view.state.choices.is_empty());
    assert_eq!(view.state.canvas.operations().len(), 1);
    assert!(
        !view
            .players
            .iter()
            .any(|player| player.player.id == view.you)
    );
}

#[test]
fn complete_hosted_match_reveals_then_rotates_and_rematches_with_a_fresh_canvas() {
    let mut session = started();
    let first_match = session.match_id;
    for turn in 0..2 {
        let artist_connection = u64::from(turn + 1);
        command(
            &mut session,
            artist_connection,
            DrawGuessCommand::ChooseWord { turn, choice: 0 },
        );
        let word = session
            .game_view(session.order[turn as usize])
            .state
            .word
            .unwrap();
        command(
            &mut session,
            if turn == 0 { 2 } else { 1 },
            DrawGuessCommand::Guess { turn, text: word },
        );
        assert_eq!(
            session.game.as_ref().unwrap().phase(),
            DrawGuessPhase::Reveal
        );
        session.advance_time(Duration::from_secs(5));
    }
    assert!(session.game_view(session.order[0]).match_complete);
    send(&mut session, 1, ClientCommand::PlayAgain);
    send(&mut session, 2, ClientCommand::PlayAgain);
    assert_ne!(session.match_id, first_match);
    let game = session.game_view(session.order[0]);
    assert_eq!(game.state.phase, DrawGuessPhase::Choosing);
    assert!(game.state.players.iter().all(|player| player.score == 0));
    assert!(game.state.canvas.operations().is_empty());
    let first = session
        .game
        .as_ref()
        .unwrap()
        .view(session.order[0])
        .remaining_ms;
    session.advance_time(Duration::from_millis(500));
    assert_eq!(
        session
            .game
            .as_ref()
            .unwrap()
            .view(session.order[0])
            .remaining_ms,
        first - 500
    );
}

#[test]
fn eight_players_have_seats_and_a_turn_each() {
    let mut session = DrawGuessSession::new(
        RoomId(99),
        52300,
        DrawGuessRuleSet {
            rounds: 1,
            drawing_seconds: 30,
        },
    )
    .unwrap();
    for connection in 1..=8 {
        join(&mut session, connection);
    }
    let rejected = send(
        &mut session,
        1,
        ClientCommand::SelectSeat { seat: SeatId(8) },
    );
    assert!(
        rejected
            .iter()
            .any(|delivery| matches!(delivery.message.event, ServerEvent::Rejected { .. }))
    );
    send(&mut session, 1, ClientCommand::StartGame);
    assert_eq!(session.order.len(), 8);
    assert_eq!(session.game_view(session.order[0]).state.total_turns, 8);
    session.advance_time(Duration::from_secs(8 * 50));
    assert!(session.game_view(session.order[0]).match_complete);
}

#[test]
fn uploaded_libraries_are_private_require_host_acceptance_and_drive_the_actual_game() {
    use leocard_draw_guess::WordLibrary;
    use leocard_protocol::{DrawGuessLibraryEvent, GameViolation, RejectReason, RoomViolation};
    let mut session =
        DrawGuessSession::new(RoomId(99), 52300, DrawGuessRuleSet::default()).unwrap();
    for connection in 1..=3 {
        join(&mut session, connection);
    }
    let library = WordLibrary::from_lines("动物", "小猫\n小狗\n小熊").unwrap();
    let refused = command(
        &mut session,
        2,
        DrawGuessCommand::SelectLibrary {
            library: Some(library.clone()),
        },
    );
    assert!(refused.iter().any(|delivery| matches!(
        delivery.message.event,
        ServerEvent::Rejected {
            reason: RejectReason::Room(RoomViolation::OnlyHostCanConfigure)
        }
    )));
    let uploaded = command(
        &mut session,
        2,
        DrawGuessCommand::UploadLibrary {
            library: library.clone(),
        },
    );
    for delivery in &uploaded {
        if let ServerEvent::GameEvent(GameEvent::DrawGuessLibrary(DrawGuessLibraryEvent::State {
            offers,
            ..
        })) = &delivery.message.event
        {
            assert_eq!(
                offers.len(),
                usize::from(delivery.recipient != ConnectionId(3))
            );
            assert_eq!(
                decode_frame::<ServerMessage>(&encode_frame(&delivery.message).unwrap()).unwrap(),
                delivery.message
            );
        }
    }
    assert_ne!(session.libraries.selected, library);
    let id = session.libraries.offers[0].id;
    let refused = command(
        &mut session,
        2,
        DrawGuessCommand::ResolveLibraryUpload {
            offer_id: id,
            accept: true,
        },
    );
    assert!(
        refused
            .iter()
            .any(|delivery| matches!(delivery.message.event, ServerEvent::Rejected { .. }))
    );
    let repeated = command(
        &mut session,
        2,
        DrawGuessCommand::UploadLibrary {
            library: library.clone(),
        },
    );
    assert!(repeated.iter().any(|delivery| matches!(
        delivery.message.event,
        ServerEvent::Rejected {
            reason: RejectReason::Game(GameViolation::DrawGuess(_))
        }
    )));
    let accepted = command(
        &mut session,
        1,
        DrawGuessCommand::ResolveLibraryUpload {
            offer_id: id,
            accept: true,
        },
    );
    assert!(
        accepted
            .iter()
            .any(|delivery| delivery.recipient == ConnectionId(2)
                && matches!(
                    delivery.message.event,
                    ServerEvent::GameEvent(GameEvent::DrawGuessLibrary(
                        DrawGuessLibraryEvent::UploadResolved { accepted: true, .. }
                    ))
                ))
    );
    assert!(session.libraries.offers.is_empty());
    assert_ne!(session.libraries.selected, library);
    command(
        &mut session,
        1,
        DrawGuessCommand::SelectLibrary {
            library: Some(library.clone()),
        },
    );
    for connection in 1..=3 {
        send(
            &mut session,
            connection,
            ClientCommand::SetReady { ready: true },
        );
    }
    send(&mut session, 1, ClientCommand::StartGame);
    let choices = session.game_view(session.order[0]).state.choices;
    assert_eq!(choices.len(), 3);
    assert!(choices.iter().all(|choice| library.words.contains(choice)));
    let rejected = command(&mut session, 2, DrawGuessCommand::UploadLibrary { library });
    assert!(
        rejected
            .iter()
            .any(|delivery| matches!(delivery.message.event, ServerEvent::Rejected { .. }))
    );
}
