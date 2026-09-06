use super::*;
use leocard_protocol::{
    GameEvent, GameSnapshot, MatchId, PROTOCOL_VERSION, PlayerId, Revision, RoomId, ServerEvent,
    ServerMessage,
};
use leocard_protocol::{
    PlayerGameProfiles, ProfileId, SeatId, ShengjiEvent, ShengjiPhaseView, ShengjiPlayerState,
    ShengjiPublicPlay, ShengjiSnapshot, ShengjiTrickView,
};
use leocard_shengji::{
    Category, ShengjiCard, ShengjiClassifiedPlay, ShengjiRank, ShengjiRuleSet, ShengjiSuit,
    ShengjiTrump,
};

fn shengji_score_snapshot(trick: Option<ShengjiTrickView>) -> ShengjiSnapshot {
    ShengjiSnapshot {
        match_id: MatchId([6; 16]),
        hand_number: 1,
        host_port: 52300,
        you: PlayerId(0),
        host: PlayerId(0),
        rules: ShengjiRuleSet::default(),
        players: (0..4)
            .map(|id| ShengjiPlayerState {
                id: PlayerId(id),
                profile_id: ProfileId([id; 32]),
                name: format!("玩家{id}"),
                avatar: None,
                seat: SeatId(id),
                hand_len: 24,
                ready: false,
                connected: true,
                auto_play: false,
                reference_points: 0,
                completed_games: 0,
                game_profiles: PlayerGameProfiles::default(),
            })
            .collect(),
        your_hand: Vec::new(),
        your_exposed_cards: Vec::new(),
        levels: [ShengjiRank::Ten; 2],
        bidding_level: ShengjiRank::Ten,
        dealer: Some(PlayerId(0)),
        trump: Some(ShengjiTrump::new(ShengjiRank::Ten, Some(ShengjiSuit::Heart)).unwrap()),
        declaration: None,
        current_player: Some(PlayerId(0)),
        trick,
        throw_failure: None,
        collecting_score: 15,
        buried_count: 8,
        your_buried: Vec::new(),
        phase: ShengjiPhaseView::Playing,
    }
}

#[test]
fn client_keeps_the_collecting_sides_public_shengji_score_cards() {
    let five = ShengjiCard::suited(0, ShengjiSuit::Club, ShengjiRank::Five);
    let ten = ShengjiCard::suited(0, ShengjiSuit::Spade, ShengjiRank::Ten);
    let previous = shengji_score_snapshot(Some(ShengjiTrickView {
        leader: PlayerId(0),
        current_player: PlayerId(0),
        winning_player: PlayerId(1),
        plays: vec![ShengjiPublicPlay {
            player: PlayerId(1),
            play: ShengjiClassifiedPlay {
                cards: vec![five, ten],
                category: Category::Suit(ShengjiSuit::Club),
                components: Vec::new(),
            },
            throw_penalty: 0,
        }],
        table_points: 15,
    }));
    let next = shengji_score_snapshot(None);
    let mut model = ClientModel::new(RoomId(7));

    assert!(model.apply(ServerMessage {
        protocol_version: PROTOCOL_VERSION,
        room_id: RoomId(7),
        revision: Revision(1),
        in_reply_to: None,
        event: ServerEvent::GameSnapshot(GameSnapshot::Shengji(previous)),
    }));
    assert!(model.apply(ServerMessage {
        protocol_version: PROTOCOL_VERSION,
        room_id: RoomId(7),
        revision: Revision(2),
        in_reply_to: None,
        event: ServerEvent::GameSnapshot(GameSnapshot::Shengji(next)),
    }));

    assert_eq!(model.shengji_collected_score_cards(), &[five, ten]);
}

#[test]
fn shengji_trick_events_keep_the_fourth_players_score_card() {
    let five = ShengjiCard::suited(0, ShengjiSuit::Club, ShengjiRank::Five);
    let ten = ShengjiCard::suited(0, ShengjiSuit::Club, ShengjiRank::Ten);
    let mut model = ClientModel::new(RoomId(7));
    assert!(model.apply(ServerMessage {
        protocol_version: PROTOCOL_VERSION,
        room_id: RoomId(7),
        revision: Revision(1),
        in_reply_to: None,
        event: ServerEvent::GameSnapshot(GameSnapshot::Shengji(shengji_score_snapshot(None,))),
    }));
    for (revision, player, card) in [(2, 0, five), (3, 3, ten)] {
        assert!(model.apply(ServerMessage {
            protocol_version: PROTOCOL_VERSION,
            room_id: RoomId(7),
            revision: Revision(revision),
            in_reply_to: None,
            event: ServerEvent::GameEvent(GameEvent::Shengji(ShengjiEvent::CardsPlayed {
                play: ShengjiPublicPlay {
                    player: PlayerId(player),
                    play: ShengjiClassifiedPlay {
                        cards: vec![card],
                        category: Category::Suit(ShengjiSuit::Club,),
                        components: Vec::new(),
                    },
                    throw_penalty: 0,
                },
                is_lead: player == 0,
            })),
        }));
    }
    assert!(model.apply(ServerMessage {
        protocol_version: PROTOCOL_VERSION,
        room_id: RoomId(7),
        revision: Revision(4),
        in_reply_to: None,
        event: ServerEvent::GameEvent(GameEvent::Shengji(ShengjiEvent::TrickFinished {
            winner: PlayerId(3),
            points: 15,
            collecting_score: 15,
        })),
    }));

    assert_eq!(model.shengji_collected_score_cards(), &[five, ten]);
}

#[test]
fn shengji_finish_event_prevents_an_incomplete_snapshot_from_awarding_the_wrong_team() {
    let five = ShengjiCard::suited(0, ShengjiSuit::Club, ShengjiRank::Five);
    let play = ShengjiPublicPlay {
        player: PlayerId(1),
        play: ShengjiClassifiedPlay {
            cards: vec![five],
            category: Category::Suit(ShengjiSuit::Club),
            components: Vec::new(),
        },
        throw_penalty: 0,
    };
    let mut model = ClientModel::new(RoomId(7));
    for (revision, event) in [
        (
            1,
            ServerEvent::GameSnapshot(GameSnapshot::Shengji(shengji_score_snapshot(None))),
        ),
        (
            2,
            ServerEvent::GameEvent(GameEvent::Shengji(ShengjiEvent::CardsPlayed {
                play: play.clone(),
                is_lead: true,
            })),
        ),
        (
            3,
            ServerEvent::GameSnapshot(GameSnapshot::Shengji(shengji_score_snapshot(Some(
                ShengjiTrickView {
                    leader: PlayerId(1),
                    current_player: PlayerId(0),
                    winning_player: PlayerId(1),
                    plays: vec![play],
                    table_points: 5,
                },
            )))),
        ),
        (
            4,
            ServerEvent::GameEvent(GameEvent::Shengji(ShengjiEvent::TrickFinished {
                winner: PlayerId(0),
                points: 5,
                collecting_score: 0,
            })),
        ),
        (
            5,
            ServerEvent::GameSnapshot(GameSnapshot::Shengji(shengji_score_snapshot(None))),
        ),
    ] {
        assert!(model.apply(ServerMessage {
            protocol_version: PROTOCOL_VERSION,
            room_id: RoomId(7),
            revision: Revision(revision),
            in_reply_to: None,
            event,
        }));
    }

    assert!(model.shengji_collected_score_cards().is_empty());
}
