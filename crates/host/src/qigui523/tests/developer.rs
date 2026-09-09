use super::*;
#[cfg(feature = "developer")]
use leocard_protocol::ServerEvent;
use leocard_protocol::{ClientCommand, GameCommand, QiGui523Command};
#[cfg(not(feature = "developer"))]
use leocard_protocol::{GameViolation, RejectReason};
#[cfg(feature = "developer")]
use leocard_qigui523::Phase;
use leocard_qigui523::{QiGuiCard, QiGuiRank, QiGuiSuit, build_deck};

#[cfg(feature = "developer")]
#[test]
fn player_can_replace_only_their_own_hand() {
    let mut session = QiGui523Session::new(ROOM, rules(), build_deck(1)).unwrap();
    join_three(&mut session);
    ready_and_start(&mut session);
    let player = session.player_id(SECOND).unwrap();
    let other = session.player_id(THIRD).unwrap();
    let other_hand = session
        .game()
        .unwrap()
        .player(to_core_player(other))
        .unwrap()
        .hand()
        .to_vec();
    let replacement = vec![
        QiGuiCard::suited(0, QiGuiSuit::Club, QiGuiRank::Joker),
        QiGuiCard::suited(0, QiGuiSuit::Club, QiGuiRank::Joker),
        QiGuiCard::suited(9, QiGuiSuit::Spade, QiGuiRank::Seven),
    ];

    let deliveries = send_next(
        &mut session,
        SECOND,
        ClientCommand::Game(GameCommand::QiGui523(QiGui523Command::SetDeveloperHand {
            cards: replacement.clone(),
        })),
    );

    assert_eq!(deliveries.len(), 3);
    let hand = session
        .game()
        .unwrap()
        .player(to_core_player(player))
        .unwrap()
        .hand();
    assert_eq!(hand.len(), replacement.len());
    assert_eq!(
        hand.iter().filter(|card| **card == replacement[0]).count(),
        2
    );
    assert!(hand.contains(&replacement[2]));
    assert_eq!(
        session
            .game()
            .unwrap()
            .player(to_core_player(other))
            .unwrap()
            .hand(),
        other_hand
    );
}

#[cfg(feature = "developer")]
#[test]
fn player_can_play_a_semantic_pair_from_impossible_physical_copies() {
    let mut session = QiGui523Session::new(ROOM, rules(), build_deck(1)).unwrap();
    join_three(&mut session);
    ready_and_start(&mut session);

    let current = from_core_player(session.game().unwrap().trick().unwrap().current_player());
    let connection = connection_for_player(&session, current);
    let pair = vec![
        QiGuiCard::suited(0, QiGuiSuit::Club, QiGuiRank::Joker),
        QiGuiCard::suited(1, QiGuiSuit::Club, QiGuiRank::Joker),
    ];

    send_next(
        &mut session,
        connection,
        ClientCommand::Game(GameCommand::QiGui523(QiGui523Command::SetDeveloperHand {
            cards: pair.clone(),
        })),
    );
    let deliveries = send_next(
        &mut session,
        connection,
        ClientCommand::Game(GameCommand::QiGui523(QiGui523Command::PlayCards {
            cards: pair,
        })),
    );

    assert!(
        deliveries
            .iter()
            .all(|delivery| !matches!(delivery.message.event, ServerEvent::Rejected { .. }))
    );
    assert!(matches!(
        session.game().unwrap().phase(),
        Phase::Finished(_)
    ));
}

#[cfg(not(feature = "developer"))]
#[test]
fn normal_build_rejects_developer_hand_commands() {
    let mut session = QiGui523Session::new(ROOM, rules(), build_deck(1)).unwrap();
    join_three(&mut session);
    ready_and_start(&mut session);

    let rejected = send_next(
        &mut session,
        SECOND,
        ClientCommand::Game(GameCommand::QiGui523(QiGui523Command::SetDeveloperHand {
            cards: vec![QiGuiCard::suited(0, QiGuiSuit::Spade, QiGuiRank::Seven)],
        })),
    );
    assert_eq!(
        rejection(&rejected),
        Some(&RejectReason::Game(
            GameViolation::DeveloperFeatureUnavailable
        ))
    );
}
