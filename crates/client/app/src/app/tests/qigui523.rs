use super::prelude::*;
use crate::app::games::qigui523::{next_time_control, previous_time_control};
use leocard_game_common::PlayerId;
use leocard_protocol::{
    MatchId, PlayerGameProfiles, PlayerPublicInfo, ProfileId, QiGui523PhaseView,
    QiGui523PlayerView, QiGui523Snapshot, QiGui523StartingCardView, QiGui523TrickView,
    QiGui523TurnTimerView, SeatId,
};
use leocard_qigui523::{
    QiGui523Bot, QiGuiCard, QiGuiRank, QiGuiRuleSet, QiGuiSuit, QiGuiTimeControl, classify,
};

#[test]
fn time_control_options_follow_the_configured_order() {
    assert_eq!(previous_time_control(QiGuiTimeControl::FivePlusTen), None);
    assert_eq!(
        next_time_control(QiGuiTimeControl::FivePlusTen),
        Some(QiGuiTimeControl::FivePlusThirty)
    );
    assert_eq!(
        next_time_control(QiGuiTimeControl::FivePlusThirty),
        Some(QiGuiTimeControl::FifteenPlusThirty)
    );
    assert_eq!(
        next_time_control(QiGuiTimeControl::FifteenPlusThirty),
        Some(QiGuiTimeControl::ThirtyPlusSixty)
    );
    assert_eq!(
        next_time_control(QiGuiTimeControl::ThirtyPlusSixty),
        Some(QiGuiTimeControl::Unlimited)
    );
    assert_eq!(
        previous_time_control(QiGuiTimeControl::Unlimited),
        Some(QiGuiTimeControl::ThirtyPlusSixty)
    );
    assert_eq!(next_time_control(QiGuiTimeControl::Unlimited), None);
}

#[test]
fn cards_are_displayed_from_high_to_low() {
    let mut cards = vec![
        QiGuiCard::suited(0, QiGuiSuit::Diamond, QiGuiRank::Four),
        QiGuiCard::suited(0, QiGuiSuit::Heart, QiGuiRank::Seven),
        QiGuiCard::suited(0, QiGuiSuit::Spade, QiGuiRank::Seven),
        QiGuiCard::suited(0, QiGuiSuit::Spade, QiGuiRank::Five),
    ];
    sort_cards_high_to_low(&mut cards);

    assert_eq!(
        cards[0],
        QiGuiCard::suited(0, QiGuiSuit::Spade, QiGuiRank::Seven)
    );
    assert_eq!(
        cards[1],
        QiGuiCard::suited(0, QiGuiSuit::Heart, QiGuiRank::Seven)
    );
    assert_eq!(
        cards[2],
        QiGuiCard::suited(0, QiGuiSuit::Spade, QiGuiRank::Five)
    );
    assert_eq!(
        cards[3],
        QiGuiCard::suited(0, QiGuiSuit::Diamond, QiGuiRank::Four)
    );
}

#[test]
fn greedy_hint_cycles_and_passes_when_no_response_exists() {
    let rules = QiGuiRuleSet::default();
    let current_card = QiGuiCard::suited(0, QiGuiSuit::Diamond, QiGuiRank::Eight);
    let current = classify(&[current_card], &rules).unwrap();
    let hand = [
        QiGuiCard::suited(0, QiGuiSuit::Diamond, QiGuiRank::Nine),
        QiGuiCard::suited(0, QiGuiSuit::Diamond, QiGuiRank::Ten),
    ];
    let mut strategy = QiGui523Bot::new();

    assert_eq!(
        next_greedy_hint(&mut strategy, &hand, &current, &[current_card], &rules,),
        HintDecision::Select(vec![hand[0]])
    );
    assert_eq!(
        next_greedy_hint(&mut strategy, &hand, &current, &[current_card], &rules,),
        HintDecision::Select(vec![hand[1]])
    );
    assert_eq!(
        next_greedy_hint(&mut strategy, &hand, &current, &[current_card], &rules,),
        HintDecision::Select(vec![hand[0]])
    );

    let no_response = [QiGuiCard::suited(0, QiGuiSuit::Diamond, QiGuiRank::Six)];
    assert_eq!(
        next_greedy_hint(
            &mut strategy,
            &no_response,
            &current,
            &[current_card],
            &rules,
        ),
        HintDecision::Pass
    );
}

#[test]
fn auto_playing_current_player_does_not_show_a_turn_clock() {
    let player = PlayerId(0);
    let mut game = QiGui523Snapshot {
        match_id: MatchId([1; 16]),
        host_port: 52300,
        you: player,
        host: player,
        players: vec![QiGui523PlayerView {
            player: PlayerPublicInfo {
                id: player,
                profile_id: ProfileId([1; 32]),
                name: "玩家".to_owned(),
                avatar: None,
                reference_points: 0,
                completed_games: 0,
                game_profiles: PlayerGameProfiles::default(),
            },
            seat: SeatId(0),
            hand_len: 1,
            score: 0,
            ready: false,
            connected: true,
            auto_play: false,
        }],
        played_cards: Vec::new(),
        your_hand: Vec::new(),
        draw_pile_len: 0,
        starting_card: QiGui523StartingCardView {
            player,
            card: QiGuiCard::suited(0, QiGuiSuit::Diamond, QiGuiRank::Four),
        },
        trick: Some(QiGui523TrickView {
            leader: player,
            current_player: player,
            winning_player: None,
            winning_play: None,
            records: Vec::new(),
            table_points: 0,
        }),
        turn_timer: Some(QiGui523TurnTimerView {
            player,
            base_seconds: 5,
            reserve_seconds: 30,
        }),
        phase: QiGui523PhaseView::Playing,
    };

    assert!(turn_clock_visible(&game, player));
    game.players[0].auto_play = true;
    assert!(!turn_clock_visible(&game, player));
}
