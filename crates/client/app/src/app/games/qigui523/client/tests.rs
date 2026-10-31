use super::*;
use crate::app::runtime::GameProjection;
use leocard_game_common::PlayerId;
use leocard_protocol::{
    GameSnapshot, MatchId, QiGui523PhaseView, QiGui523PublicPlay, QiGui523PublicPlayRecord,
    QiGui523RevealedHand, QiGui523Snapshot, QiGui523StartingCardView, QiGui523TrickView,
    ServerEvent,
};
use leocard_qigui523::{QiGuiCard, QiGuiPlayKind, QiGuiRank, QiGuiSuit};

fn score_history_snapshot(
    trick: Option<QiGui523TrickView>,
    phase: QiGui523PhaseView,
) -> QiGui523Snapshot {
    let match_id = match &phase {
        QiGui523PhaseView::Finished { match_id, .. } => *match_id,
        QiGui523PhaseView::Playing => MatchId([0; 16]),
    };
    QiGui523Snapshot {
        match_id,
        host_port: 52300,
        you: PlayerId(0),
        host: PlayerId(0),
        players: Vec::new(),
        played_cards: Vec::new(),
        your_hand: Vec::new(),
        draw_pile_len: 0,
        starting_card: QiGui523StartingCardView {
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
    let mut model = QiGuiClientState::default();
    let completed = QiGui523TrickView {
        leader: PlayerId(0),
        current_player: PlayerId(0),
        winning_player: Some(PlayerId(1)),
        winning_play: None,
        records: vec![
            QiGui523PublicPlayRecord::Played {
                player: PlayerId(0),
                play: QiGui523PublicPlay {
                    kind: QiGuiPlayKind::Single,
                    cards: vec![five],
                },
            },
            QiGui523PublicPlayRecord::Played {
                player: PlayerId(1),
                play: QiGui523PublicPlay {
                    kind: QiGuiPlayKind::Single,
                    cards: vec![ten],
                },
            },
        ],
        table_points: 15,
    };
    model.observe_score_cards(&score_history_snapshot(
        Some(completed),
        QiGui523PhaseView::Playing,
    ));
    model.observe_score_cards(&score_history_snapshot(
        Some(QiGui523TrickView {
            leader: PlayerId(1),
            current_player: PlayerId(1),
            winning_player: None,
            winning_play: None,
            records: Vec::new(),
            table_points: 0,
        }),
        QiGui523PhaseView::Playing,
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
    let mut model = QiGuiClientState::default();
    model.captured_score_cards.insert(PlayerId(0), vec![five]);
    let finished = score_history_snapshot(
        None,
        QiGui523PhaseView::Finished {
            match_id: MatchId([1; 16]),
            finisher: PlayerId(1),
            scores: Vec::new(),
            remaining_hands: vec![QiGui523RevealedHand {
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
    let mut model = QiGuiClientState::default();
    let five = QiGuiCard::suited(0, QiGuiSuit::Diamond, QiGuiRank::Five);
    model.captured_score_cards.insert(PlayerId(0), vec![five]);
    let mut previous = score_history_snapshot(None, QiGui523PhaseView::Playing);
    previous.match_id = MatchId([2; 16]);
    model.snapshot = Some(previous);
    let mut next = score_history_snapshot(None, QiGui523PhaseView::Playing);
    next.match_id = MatchId([3; 16]);

    model.apply(&ServerEvent::GameSnapshot(GameSnapshot::QiGui523(next)));

    assert!(model.captured_score_cards(PlayerId(0)).is_empty());
    assert!(model.observed_trick.is_none());
    assert_eq!(model.snapshot.as_ref().unwrap().match_id, MatchId([3; 16]));
}

#[test]
fn switching_view_clears_previous_players_history_and_effects() {
    let mut state = QiGuiClientState::default();
    let mut snapshot = score_history_snapshot(None, QiGui523PhaseView::Playing);
    state.apply(&ServerEvent::GameSnapshot(snapshot.clone().into()));
    let five = QiGuiCard::suited(0, QiGuiSuit::Diamond, QiGuiRank::Five);
    state.captured_score_cards.insert(PlayerId(0), vec![five]);
    snapshot.you = PlayerId(1);
    state.apply(&ServerEvent::GameSnapshot(snapshot.into()));
    assert!(state.captured_score_cards(PlayerId(0)).is_empty());
    assert_eq!(state.view_key().unwrap().1, PlayerId(1));
    state.apply(&ServerEvent::LeftRoom);
    assert!(state.view_key().is_none());
    assert!(state.last_score_capture().is_none());
}
