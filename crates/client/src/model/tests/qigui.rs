use super::prelude::*;
use leocard_protocol::{
    GamePhaseView, PublicPlay, PublicPlayRecord, QiGui523Snapshot, RevealedHand, StartingCardView,
    TrickView,
};
use leocard_protocol::{
    GameSnapshot, MatchId, PROTOCOL_VERSION, PlayerId, Revision, RoomId, ServerEvent, ServerMessage,
};
use leocard_qigui523::{QiGuiCard, QiGuiPlayKind, QiGuiRank, QiGuiSuit};

pub(super) fn score_history_snapshot(
    trick: Option<TrickView>,
    phase: GamePhaseView,
) -> QiGui523Snapshot {
    let match_id = match &phase {
        GamePhaseView::Finished { match_id, .. } => *match_id,
        GamePhaseView::Playing => MatchId([0; 16]),
    };
    QiGui523Snapshot {
        match_id,
        host_port: 52300,
        you: PlayerId(0),
        host: PlayerId(0),
        players: Vec::new(),
        your_hand: Vec::new(),
        draw_pile_len: 0,
        starting_card: StartingCardView {
            player: PlayerId(0),
            card: QiGuiCard::suited(0, QiGuiSuit::Diamond, QiGuiRank::Four),
        },
        trick,
        turn_timer: None,
        phase,
    }
}

#[test]
fn client_records_all_score_cards_when_a_trick_changes() {
    let five = QiGuiCard::suited(0, QiGuiSuit::Diamond, QiGuiRank::Five);
    let ten = QiGuiCard::suited(0, QiGuiSuit::Club, QiGuiRank::Ten);
    let mut model = ClientModel::new(RoomId(7));
    let completed = TrickView {
        leader: PlayerId(0),
        current_player: PlayerId(0),
        winning_player: Some(PlayerId(1)),
        winning_play: None,
        records: vec![
            PublicPlayRecord::Played {
                player: PlayerId(0),
                play: PublicPlay {
                    kind: QiGuiPlayKind::Single,
                    cards: vec![five],
                },
            },
            PublicPlayRecord::Played {
                player: PlayerId(1),
                play: PublicPlay {
                    kind: QiGuiPlayKind::Single,
                    cards: vec![ten],
                },
            },
        ],
        table_points: 15,
    };
    model.observe_score_cards(&score_history_snapshot(
        Some(completed),
        GamePhaseView::Playing,
    ));
    model.observe_score_cards(&score_history_snapshot(
        Some(TrickView {
            leader: PlayerId(1),
            current_player: PlayerId(1),
            winning_player: None,
            winning_play: None,
            records: Vec::new(),
            table_points: 0,
        }),
        GamePhaseView::Playing,
    ));

    assert_eq!(model.captured_score_cards(PlayerId(1)), &[five, ten]);
    assert!(model.captured_score_cards(PlayerId(0)).is_empty());
    assert_eq!(model.score_capture_serial(), 1);
    assert_eq!(
        model.last_score_capture(),
        Some(&ScoreCaptureEffect {
            player: PlayerId(1),
            cards: vec![five, ten],
            source_players: vec![None, None],
            score_before: 0,
            score_after: 15,
        })
    );
}

#[test]
fn client_assigns_revealed_hand_score_cards_to_the_finisher_once() {
    let five = QiGuiCard::suited(0, QiGuiSuit::Diamond, QiGuiRank::Five);
    let ten = QiGuiCard::suited(0, QiGuiSuit::Club, QiGuiRank::Ten);
    let king = QiGuiCard::suited(0, QiGuiSuit::Heart, QiGuiRank::King);
    let mut model = ClientModel::new(RoomId(7));
    model
        .games
        .qigui523
        .captured_score_cards
        .insert(PlayerId(0), vec![five]);
    let finished = score_history_snapshot(
        None,
        GamePhaseView::Finished {
            match_id: MatchId([1; 16]),
            finisher: PlayerId(1),
            scores: Vec::new(),
            remaining_hands: vec![RevealedHand {
                player: PlayerId(0),
                cards: vec![ten, king],
            }],
            reference_changes: Vec::new(),
            captured_hand_points: 20,
        },
    );

    model.observe_score_cards(&finished);
    model.observe_score_cards(&finished);

    assert_eq!(model.captured_score_cards(PlayerId(0)), &[five]);
    assert_eq!(model.captured_score_cards(PlayerId(1)), &[ten, king]);
    assert_eq!(
        model
            .last_score_capture()
            .map(|effect| effect.source_players.as_slice()),
        Some(&[Some(PlayerId(0)), Some(PlayerId(0))][..])
    );
}

#[test]
fn client_clears_score_card_history_when_a_rematch_starts() {
    let mut model = ClientModel::new(RoomId(7));
    let five = QiGuiCard::suited(0, QiGuiSuit::Diamond, QiGuiRank::Five);
    model
        .games
        .qigui523
        .captured_score_cards
        .insert(PlayerId(0), vec![five]);
    model.active_match_id = Some(MatchId([2; 16]));
    let mut previous = score_history_snapshot(None, GamePhaseView::Playing);
    previous.match_id = MatchId([2; 16]);
    model.game = Some(GameSnapshot::QiGui523(previous));
    let mut next = score_history_snapshot(None, GamePhaseView::Playing);
    next.match_id = MatchId([3; 16]);

    assert!(model.apply(ServerMessage {
        protocol_version: PROTOCOL_VERSION,
        room_id: RoomId(7),
        revision: Revision(1),
        in_reply_to: None,
        event: ServerEvent::GameSnapshot(GameSnapshot::QiGui523(next)),
    }));

    assert!(model.captured_score_cards(PlayerId(0)).is_empty());
    assert!(model.games.qigui523.observed_trick.is_none());
    assert_eq!(model.active_match_id, Some(MatchId([3; 16])));
}
