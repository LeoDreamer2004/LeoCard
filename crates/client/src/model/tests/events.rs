use super::*;
use crate::model::uno::uno_event_notice;
use leocard_protocol::{ChatMessage, PlayerInteraction, PublicPlay, QiGui523Event, UnoEvent};
use leocard_qigui523::{QiGuiCard, QiGuiPlayKind, QiGuiRank, QiGuiSuit};
use leocard_uno::UnoChallengeResult;

#[test]
fn uno_challenge_and_report_events_create_readable_notices() {
    let challenge = UnoEvent::ChallengeResolved {
        challenger: PlayerId(1),
        offender: PlayerId(0),
        result: UnoChallengeResult::Successful,
        penalized: PlayerId(0),
        count: 8,
        card_backs: Vec::new(),
    };
    assert_eq!(
        uno_event_notice(None, &challenge).as_deref(),
        Some("玩家 2 质疑成功，玩家 1 摸 8 张")
    );
    let report = UnoEvent::UnoReported {
        reporter: PlayerId(0),
        target: PlayerId(1),
        card_backs: Vec::new(),
    };
    assert_eq!(
        uno_event_notice(None, &report).as_deref(),
        Some("玩家 1 检举了 玩家 2，罚摸 2 张")
    );
    let skip = UnoEvent::SkipResolved {
        player: PlayerId(1),
        remaining: 2,
        drew_card: true,
        card_back: None,
    };
    assert_eq!(uno_event_notice(None, &skip), None);
}

#[test]
fn play_effect_events_advance_an_independent_animation_serial() {
    let mut model = ClientModel::new(RoomId(7));
    let play = PublicPlay {
        kind: QiGuiPlayKind::Straight { card_count: 3 },
        cards: vec![
            QiGuiCard::suited(0, QiGuiSuit::Diamond, QiGuiRank::Four),
            QiGuiCard::suited(0, QiGuiSuit::Diamond, QiGuiRank::Six),
            QiGuiCard::suited(0, QiGuiSuit::Diamond, QiGuiRank::Eight),
        ],
    };
    assert!(model.apply(ServerMessage {
        protocol_version: PROTOCOL_VERSION,
        room_id: RoomId(7),
        revision: Revision(1),
        in_reply_to: None,
        event: ServerEvent::GameEvent(GameEvent::QiGui523(QiGui523Event::PlayEffect {
            player: PlayerId(2),
            play: play.clone(),
        })),
    }));

    assert_eq!(model.play_effect_serial(), 1);
    assert_eq!(model.last_play_effect(), Some(&(PlayerId(2), play)));
}

#[test]
fn player_interactions_are_queued_in_receive_order_and_drained_once() {
    let mut model = ClientModel::new(RoomId(7));
    let flower = PlayerInteraction {
        source: PlayerId(0),
        target: PlayerId(2),
        kind: PlayerInteractionKind::Flower,
        seed: 11,
    };
    let shoe = PlayerInteraction {
        source: PlayerId(1),
        target: PlayerId(0),
        kind: PlayerInteractionKind::Shoe,
        seed: 22,
    };
    for interaction in [flower, shoe] {
        assert!(model.apply(ServerMessage {
            protocol_version: PROTOCOL_VERSION,
            room_id: RoomId(7),
            revision: Revision(1),
            in_reply_to: None,
            event: ServerEvent::PlayerInteraction(interaction),
        }));
    }

    assert_eq!(model.take_player_interactions(), vec![flower, shoe]);
    assert!(model.take_player_interactions().is_empty());
}

#[test]
fn chat_messages_are_queued_in_receive_order_and_drained_once() {
    let mut model = ClientModel::new(RoomId(7));
    let text = ChatMessage {
        source: PlayerId(0),
        content: ChatContent::Text("你好".to_owned()),
    };
    let voice = ChatMessage {
        source: PlayerId(2),
        content: ChatContent::QuickVoice(3),
    };
    for chat in [text.clone(), voice.clone()] {
        assert!(model.apply(ServerMessage {
            protocol_version: PROTOCOL_VERSION,
            room_id: RoomId(7),
            revision: Revision(1),
            in_reply_to: None,
            event: ServerEvent::ChatMessage(chat),
        }));
    }

    assert_eq!(model.take_chat_messages(), vec![text, voice]);
    assert!(model.take_chat_messages().is_empty());
}
