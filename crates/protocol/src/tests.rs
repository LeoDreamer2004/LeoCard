use super::*;
use leocard_qigui523::QiGuiRuleSet;
use leocard_shengji::{Component, ShengjiCard, ShengjiRank, ShengjiRuleSet, ShengjiSuit};
use leocard_texas_holdem::TexasHoldemRuleSet;

#[test]
fn four_deck_spaceship_component_round_trips_through_protocol_codec() {
    let cards = [ShengjiRank::Three, ShengjiRank::Four]
        .into_iter()
        .flat_map(|rank| {
            (0..4).map(move |deck| ShengjiCard::suited(deck, ShengjiSuit::Spade, rank))
        })
        .collect::<Vec<_>>();
    let component = Component::Spaceship {
        cards,
        quad_count: 2,
        top_strength: 2,
    };
    let decoded: Component = decode_frame(&encode_frame(&component).unwrap()).unwrap();
    assert_eq!(decoded, component);
}

#[test]
fn client_message_round_trips_through_tcp_frame() {
    let message = ClientMessage::new(
        RoomId(42),
        RequestId(7),
        ClientCommand::join(JoinRequest {
            name: "玩家一".to_owned(),
            reconnect_token: ReconnectToken(123),
            profile_id: ProfileId([7; 32]),
            reference_points: 12,
            completed_games: 8,
            game_profiles: PlayerGameProfiles {
                qigui523: Some(QiGui523ProfileStats {
                    completed_games: 2,
                    total_score: 180,
                    total_reference_delta: 4,
                    straight_plays: 3,
                    longest_straight: 7,
                    ..QiGui523ProfileStats::default()
                }),
                texas_holdem: Some(TexasHoldemProfileStats {
                    completed_games: 3,
                    total_reference_delta: -1,
                    total_final_chips: 240,
                    ..TexasHoldemProfileStats::default()
                }),
                shengji: Some(ShengjiProfileStats {
                    completed_games: 4,
                    total_reference_delta: 2,
                    declaration_games: 1,
                    longest_tractor: 3,
                    ..ShengjiProfileStats::default()
                }),
                uno: Some(UnoProfileStats {
                    completed_games: 5,
                    total_reference_delta: 7,
                    max_hand_cards: 14,
                    uno_calls: 3,
                    ..UnoProfileStats::default()
                }),
                interactions: Some(PlayerInteractionStats {
                    flowers_received: 14,
                    eggs_received: 23,
                }),
            },
            identity_signature: vec![9; 64],
        }),
    );
    let frame = encode_frame(&message).unwrap();
    let decoded: ClientMessage = decode_frame(&frame).unwrap();
    assert_eq!(decoded, message);
    assert_eq!(
        u32::from_be_bytes(frame[..4].try_into().unwrap()) as usize,
        frame.len() - 4
    );
}

#[test]
fn interaction_event_round_trips_with_its_shared_animation_seed() {
    let message = ServerMessage {
        protocol_version: PROTOCOL_VERSION,
        room_id: RoomId(42),
        revision: Revision(9),
        in_reply_to: Some(RequestId(7)),
        event: ServerEvent::PlayerInteraction(PlayerInteraction {
            source: PlayerId(1),
            target: PlayerId(3),
            kind: PlayerInteractionKind::Wine,
            seed: 523,
        }),
    };

    let frame = encode_frame(&message).unwrap();
    let decoded: ServerMessage = decode_frame(&frame).unwrap();
    assert_eq!(decoded, message);
}

#[test]
fn chat_events_round_trip_for_text_quick_voice_and_emoji() {
    for content in [
        ChatContent::Text("大家好".to_owned()),
        ChatContent::QuickVoice(7),
        ChatContent::Emoji(ChatEmoji::Laugh),
    ] {
        let message = ServerMessage {
            protocol_version: PROTOCOL_VERSION,
            room_id: RoomId(42),
            revision: Revision(9),
            in_reply_to: Some(RequestId(7)),
            event: ServerEvent::ChatMessage(ChatMessage {
                source: PlayerId(1),
                content,
            }),
        };
        let frame = encode_frame(&message).unwrap();
        let decoded: ServerMessage = decode_frame(&frame).unwrap();
        assert_eq!(decoded, message);
    }
}

#[test]
fn lobby_game_kind_and_rules_remain_consistent() {
    let rules = QiGuiRuleSet::default();
    let lobby = LobbySnapshot {
        game: GameKind::QiGui523,
        rules: GameRules::QiGui523(rules),
        host_port: 52300,
        host: None,
        players: Vec::new(),
    };

    assert_eq!(lobby.game, lobby.rules.kind());
    assert_eq!(lobby.rules.qigui523(), Some(&rules));

    let texas_rules = TexasHoldemRuleSet::default();
    let texas_lobby = LobbySnapshot {
        game: GameKind::TexasHoldem,
        rules: GameRules::TexasHoldem(texas_rules),
        host_port: 52301,
        host: None,
        players: Vec::new(),
    };
    assert_eq!(texas_lobby.game, texas_lobby.rules.kind());
    assert_eq!(texas_lobby.rules.texas_holdem(), Some(&texas_rules));

    let shengji_rules = ShengjiRuleSet::default();
    let shengji_lobby = LobbySnapshot {
        game: GameKind::Shengji,
        rules: GameRules::Shengji(shengji_rules),
        host_port: 52302,
        host: None,
        players: Vec::new(),
    };
    assert_eq!(shengji_lobby.game, shengji_lobby.rules.kind());
    assert_eq!(shengji_lobby.rules.shengji(), Some(&shengji_rules));
}

#[test]
fn rejects_truncated_and_oversized_frames_before_deserialization() {
    assert!(matches!(
        decode_frame::<ClientMessage>(&[0, 1, 2]),
        Err(FrameError::FrameTooShort)
    ));
    assert!(matches!(
        decode_frame::<ClientMessage>(&[0, 0, 0, 2, 1]),
        Err(FrameError::LengthMismatch {
            declared: 2,
            actual: 1
        })
    ));

    let too_large = u32::try_from(MAX_FRAME_PAYLOAD + 1).unwrap().to_be_bytes();
    assert!(matches!(
        decode_frame::<ClientMessage>(&too_large),
        Err(FrameError::PayloadTooLarge { .. })
    ));
}
