use super::*;
use leocard_mahjong::{MahjongClaim, MahjongRuleSet, MahjongSuit, MahjongTile, MahjongTileKind};
use leocard_qigui523::QiGuiRuleSet;
use leocard_shengji::{
    Component, ShengjiBidKind, ShengjiBidTrump, ShengjiCard, ShengjiRank, ShengjiRuleSet,
    ShengjiSuit, ShengjiTrump, build_deck,
};
use leocard_texas_holdem::{TexasHoldemAction, TexasHoldemRuleSet};
use leocard_uno::{UnoCard, UnoColor, UnoFace};

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
fn texas_holdem_commands_round_trip_through_the_common_protocol() {
    let rules = TexasHoldemRuleSet {
        player_count: 6,
        starting_chips: 40,
        short_deck: true,
        ignore_kickers: true,
        omaha: true,
    };
    for command in [
        GameCommand::TexasHoldem(TexasHoldemCommand::SetAutoPlay { enabled: true }),
        GameCommand::TexasHoldem(TexasHoldemCommand::UpdateRules { rules }),
        GameCommand::TexasHoldem(TexasHoldemCommand::Act {
            action: TexasHoldemAction::RaiseTo(12),
        }),
    ] {
        let message = ClientMessage::new(
            RoomId(42),
            RequestId(8),
            ClientCommand::Game(command.clone()),
        );
        let decoded: ClientMessage = decode_frame(&encode_frame(&message).unwrap()).unwrap();
        assert_eq!(decoded.command, ClientCommand::Game(command));
    }
}

#[test]
fn shengji_commands_round_trip_through_the_common_protocol() {
    let first_ten = ShengjiCard::suited(0, ShengjiSuit::Heart, ShengjiRank::Ten);
    let second_ten = ShengjiCard::suited(1, ShengjiSuit::Heart, ShengjiRank::Ten);
    for command in [
        GameCommand::Shengji(ShengjiCommand::SetAutoPlay { enabled: true }),
        GameCommand::Shengji(ShengjiCommand::UpdateRules {
            rules: ShengjiRuleSet::default(),
        }),
        GameCommand::Shengji(ShengjiCommand::Declare {
            cards: vec![first_ten, second_ten],
        }),
        GameCommand::Shengji(ShengjiCommand::ConfirmBidPass),
        GameCommand::Shengji(ShengjiCommand::Bury {
            cards: build_deck()[..8].to_vec(),
        }),
        GameCommand::Shengji(ShengjiCommand::ChooseBottomCopy {
            cards: Some(vec![first_ten, second_ten]),
        }),
        GameCommand::Shengji(ShengjiCommand::ChooseBottomCopy { cards: None }),
        GameCommand::Shengji(ShengjiCommand::ChooseFiveTrumpCrossing {
            cards: Some(vec![first_ten, second_ten]),
        }),
        GameCommand::Shengji(ShengjiCommand::ChooseFiveTrumpCrossing { cards: None }),
        GameCommand::Shengji(ShengjiCommand::ReturnFiveTrumpCrossing {
            cards: vec![first_ten, second_ten],
        }),
        GameCommand::Shengji(ShengjiCommand::PlayCards {
            cards: vec![first_ten],
        }),
    ] {
        let message = ClientMessage::new(
            RoomId(42),
            RequestId(9),
            ClientCommand::Game(command.clone()),
        );
        let decoded: ClientMessage = decode_frame(&encode_frame(&message).unwrap()).unwrap();
        assert_eq!(decoded.command, ClientCommand::Game(command));
    }
}

#[test]
fn mahjong_commands_and_public_events_round_trip_through_the_common_protocol() {
    let tile = MahjongTile::new(MahjongTileKind::suited(MahjongSuit::Characters, 5), 2);
    for command in [
        GameCommand::Mahjong(MahjongCommand::UpdateRules {
            rules: MahjongRuleSet::default(),
        }),
        GameCommand::Mahjong(MahjongCommand::SetDeveloperHand {
            tiles: vec![tile.kind()],
        }),
        GameCommand::Mahjong(MahjongCommand::Discard { tile }),
        GameCommand::Mahjong(MahjongCommand::RespondToClaim {
            claim: MahjongClaim::Pung,
        }),
        GameCommand::Mahjong(MahjongCommand::DeclareSelfDraw),
        GameCommand::Mahjong(MahjongCommand::DeclareConcealedKong { tile: tile.kind() }),
        GameCommand::Mahjong(MahjongCommand::DeclareAddedKong { tile }),
    ] {
        let message = ClientMessage::new(
            RoomId(42),
            RequestId(14),
            ClientCommand::Game(command.clone()),
        );
        let decoded: ClientMessage = decode_frame(&encode_frame(&message).unwrap()).unwrap();
        assert_eq!(decoded.command, ClientCommand::Game(command));
    }

    for event in [
        MahjongEvent::ClaimResolved {
            player: PlayerId(3),
            source: PlayerId(1),
            tile,
            claim: MahjongClaim::Pung,
        },
        MahjongEvent::KongDeclared {
            player: PlayerId(2),
            tile: tile.kind(),
            added: true,
        },
        MahjongEvent::FlowerReplaced {
            player: PlayerId(1),
        },
    ] {
        let message = ServerMessage {
            protocol_version: PROTOCOL_VERSION,
            room_id: RoomId(42),
            revision: Revision(15),
            in_reply_to: None,
            event: ServerEvent::GameEvent(GameEvent::Mahjong(event)),
        };
        let decoded: ServerMessage = decode_frame(&encode_frame(&message).unwrap()).unwrap();
        assert_eq!(decoded, message);
    }
}

#[test]
fn uno_extension_commands_and_events_round_trip_through_the_common_protocol() {
    let first = UnoCard::number(UnoColor::Red, 7, 0);
    let second = UnoCard::number(UnoColor::Red, 7, 1);
    for command in [
        GameCommand::Uno(UnoCommand::PlayCards {
            cards: vec![first, second],
            chosen_color: None,
        }),
        GameCommand::Uno(UnoCommand::JumpIn { card: second }),
        GameCommand::Uno(UnoCommand::ChooseSwapOneTarget {
            target: PlayerId(2),
        }),
        GameCommand::Uno(UnoCommand::ChooseSevenSwapTarget {
            target: PlayerId(2),
        }),
        GameCommand::Uno(UnoCommand::GiveSwapOneCard { card: first }),
        GameCommand::Uno(UnoCommand::ForceTradeHands {
            first: PlayerId(0),
            second: PlayerId(2),
        }),
    ] {
        let message = ClientMessage::new(
            RoomId(42),
            RequestId(10),
            ClientCommand::Game(command.clone()),
        );
        let decoded: ClientMessage = decode_frame(&encode_frame(&message).unwrap()).unwrap();
        assert_eq!(decoded.command, ClientCommand::Game(command));
    }

    let message = ServerMessage {
        protocol_version: PROTOCOL_VERSION,
        room_id: RoomId(42),
        revision: Revision(11),
        in_reply_to: None,
        event: ServerEvent::GameEvent(GameEvent::Uno(UnoEvent::CardPlayed {
            player: PlayerId(1),
            card: second,
            chosen_color: None,
            play_index: 1,
            play_count: 2,
        })),
    };
    let decoded: ServerMessage = decode_frame(&encode_frame(&message).unwrap()).unwrap();
    assert_eq!(decoded, message);

    let roulette = ServerMessage {
        protocol_version: PROTOCOL_VERSION,
        room_id: RoomId(42),
        revision: Revision(12),
        in_reply_to: None,
        event: ServerEvent::GameEvent(GameEvent::Uno(UnoEvent::ColorRouletteResolved {
            player: PlayerId(1),
            color: UnoColor::Blue,
            count: 6,
            card_backs: Vec::new(),
        })),
    };
    let decoded: ServerMessage = decode_frame(&encode_frame(&roulette).unwrap()).unwrap();
    assert_eq!(decoded, roulette);

    let reflected = ServerMessage {
        protocol_version: PROTOCOL_VERSION,
        room_id: RoomId(42),
        revision: Revision(12),
        in_reply_to: None,
        event: ServerEvent::GameEvent(GameEvent::Uno(UnoEvent::DrawPenaltyReflected {
            player: PlayerId(2),
            target: PlayerId(1),
            count: 8,
            card_backs: Vec::new(),
        })),
    };
    let decoded: ServerMessage = decode_frame(&encode_frame(&reflected).unwrap()).unwrap();
    assert_eq!(decoded, reflected);

    let revealed = ServerMessage {
        protocol_version: PROTOCOL_VERSION,
        room_id: RoomId(42),
        revision: Revision(13),
        in_reply_to: None,
        event: ServerEvent::GameEvent(GameEvent::Uno(UnoEvent::StackNumberRevealed {
            player: PlayerId(2),
            cards: vec![
                UnoCard::action(UnoColor::Blue, UnoFace::Skip, 0),
                UnoCard::number(UnoColor::Yellow, 6, 0),
            ],
            value: 6,
        })),
    };
    let decoded: ServerMessage = decode_frame(&encode_frame(&revealed).unwrap()).unwrap();
    assert_eq!(decoded, revealed);
}

#[test]
fn active_shengji_snapshot_round_trips_private_hand_and_own_bottom() {
    let your_card = ShengjiCard::suited(0, ShengjiSuit::Spade, ShengjiRank::Ace);
    let exposed = ShengjiCard::suited(0, ShengjiSuit::Heart, ShengjiRank::Ten);
    let players = (0..4)
        .map(|id| ShengjiPlayerState {
            id: PlayerId(id),
            profile_id: ProfileId([id; 32]),
            name: format!("玩家{id}"),
            avatar: None,
            seat: SeatId(id),
            hand_len: 25,
            ready: false,
            connected: true,
            auto_play: false,
            reference_points: 0,
            completed_games: 0,
            game_profiles: PlayerGameProfiles::default(),
        })
        .collect();
    let snapshot = GameSnapshot::Shengji(ShengjiSnapshot {
        match_id: MatchId([8; 16]),
        hand_number: 1,
        host_port: 52300,
        you: PlayerId(2),
        host: PlayerId(0),
        rules: ShengjiRuleSet {
            deck_count: 3,
            bid_with_joker: true,
            constant_trump: true,
            ..ShengjiRuleSet::default()
        },
        players,
        your_hand: vec![your_card],
        your_exposed_cards: vec![exposed],
        levels: [ShengjiRank::Ten, ShengjiRank::Nine],
        bidding_level: ShengjiRank::Ten,
        dealer: Some(PlayerId(0)),
        trump: Some(
            ShengjiTrump::new(ShengjiRank::Ten, Some(ShengjiSuit::Heart))
                .unwrap()
                .with_constant_trump(true),
        ),
        declaration: Some(ShengjiDeclarationView {
            player: PlayerId(0),
            trump: ShengjiBidTrump::Suit(ShengjiSuit::Heart),
            kind: ShengjiBidKind::Initial,
            protected: false,
            cards: vec![exposed],
        }),
        current_player: Some(PlayerId(0)),
        trick: None,
        throw_failure: None,
        collecting_score: 15,
        buried_count: 8,
        your_buried: vec![exposed],
        phase: ShengjiPhaseView::Playing,
    });

    let decoded: GameSnapshot = decode_frame(&encode_frame(&snapshot).unwrap()).unwrap();
    let decoded = decoded.into_shengji().unwrap();
    assert_eq!(decoded.you, PlayerId(2));
    assert_eq!(decoded.your_hand, vec![your_card]);
    assert_eq!(decoded.your_exposed_cards, vec![exposed]);
    assert_eq!(decoded.your_buried, vec![exposed]);
    assert_eq!(
        decoded
            .players
            .iter()
            .map(|player| player.hand_len)
            .sum::<u8>(),
        100
    );
    assert_eq!(decoded.buried_count, 8);
    assert!(decoded.rules.constant_trump);
    assert!(decoded.rules.bid_with_joker);
    assert_eq!(decoded.rules.deck_count, 3);
    assert!(decoded.trump.unwrap().constant_trump);
    assert!(matches!(decoded.phase, ShengjiPhaseView::Playing));
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
