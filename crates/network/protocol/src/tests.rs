use super::*;
use leocard_achievements::TrophyCounts;
use leocard_blackjack::BlackjackProfileStats;
use leocard_doudizhu::DoudizhuProfileStats;
use leocard_game_common::PlayerId;
use leocard_mc_guess::{
    McGuessCategoryCount, McGuessCategoryStatistics, McGuessGameState, McGuessGraphView,
    McGuessMode, McGuessProfileStats, McGuessRuleSet,
};

use leocard_qigui523::{QiGui523ProfileStats, QiGuiRuleSet};

use leocard_shengji::{
    ShengjiCard, ShengjiComponent, ShengjiProfileStats, ShengjiRank, ShengjiRuleSet, ShengjiSuit,
};

use leocard_texas_holdem::{TexasProfileStats, TexasRuleSet};

use leocard_uno::UnoProfileStats;

#[test]
fn graph_combo_targets_and_success_counts_round_trip_through_protocol_codec() {
    let rules = McGuessRuleSet {
        mode: McGuessMode::Graph,
        ..Default::default()
    };
    let mut game = McGuessGameState::new_graph(vec![PlayerId(0), PlayerId(1)], rules, 5).unwrap();
    let mut revealed = game.clone();
    revealed.reveal_graph().unwrap();
    let answer = revealed
        .graph_view()
        .unwrap()
        .selected_node()
        .entry
        .as_ref()
        .unwrap()
        .id
        .clone();
    game.guess(PlayerId(0), &answer).unwrap();
    let graph = game.graph_view().unwrap();
    let decoded: McGuessGraphView = decode_frame(&encode_frame(&graph).unwrap()).unwrap();
    assert_eq!(graph, decoded);
    assert_eq!(graph.guesses.last().unwrap().combo, 1);
    let combo = graph.combo.as_ref().unwrap();
    assert_eq!(combo.count, 1);
    assert!(!combo.targets.is_empty());
    assert!(
        combo
            .targets
            .iter()
            .all(|&number| graph.can_guess_node(number))
    );
}

#[test]
fn four_deck_spaceship_component_round_trips_through_protocol_codec() {
    let cards = [ShengjiRank::Three, ShengjiRank::Four]
        .into_iter()
        .flat_map(|rank| {
            (0..4).map(move |deck| ShengjiCard::suited(deck, ShengjiSuit::Spade, rank))
        })
        .collect::<Vec<_>>();
    let component = ShengjiComponent::Spaceship {
        cards,
        quad_count: 2,
        top_strength: 2,
    };
    let decoded: ShengjiComponent = decode_frame(&encode_frame(&component).unwrap()).unwrap();
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
                blackjack: Some(BlackjackProfileStats {
                    completed_games: 3,
                    total_chips_delta: 42,
                    ..Default::default()
                }),
                achievements: TrophyCounts {
                    gold: 2,
                    silver: 7,
                    bronze: 11,
                },
                coins: 300,
                mc_guess: Some(McGuessProfileStats {
                    completed_games: 2,
                    total_score: 38,
                    categories: McGuessCategoryStatistics {
                        block: McGuessCategoryCount { games: 4, wins: 1 },
                        item: McGuessCategoryCount { games: 2, wins: 1 },
                        ..Default::default()
                    },
                    ..Default::default()
                }),
                gender: PlayerGender::Female,
                qigui523: Some(QiGui523ProfileStats {
                    completed_games: 2,
                    total_score: 180,
                    total_reference_delta: 4,
                    straight_plays: 3,
                    longest_straight: 7,
                    ..QiGui523ProfileStats::default()
                }),
                texas: Some(TexasProfileStats {
                    completed_games: 3,
                    total_reference_delta: -1,
                    total_final_chips: 240,
                    ..TexasProfileStats::default()
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
                mahjong: None,
                doudizhu: Some(DoudizhuProfileStats {
                    completed_games: 4,
                    total_match_score: -12,
                    bid_actions: 7,
                    total_bid_score: 9,
                    bomb_plays: 3,
                    ..Default::default()
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
        game_context: None,
        protocol_version: PROTOCOL_VERSION,
        room_id: RoomId(42),
        revision: Revision(9),
        in_reply_to: Some(RequestId(7)),
        event: ServerEvent::PlayerInteraction(PlayerInteraction {
            source: PlayerId(1),
            target: PlayerId(3),
            kind: PlayerInteractionKind::Wine,
            seed: 523,
            elapsed_millis: 100,
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
            game_context: None,
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

    let texas_rules = TexasRuleSet::default();
    let texas_lobby = LobbySnapshot {
        game: GameKind::Texas,
        rules: GameRules::Texas(texas_rules),
        host_port: 52301,
        host: None,
        players: Vec::new(),
    };
    assert_eq!(texas_lobby.game, texas_lobby.rules.kind());
    assert_eq!(texas_lobby.rules.texas(), Some(&texas_rules));

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
