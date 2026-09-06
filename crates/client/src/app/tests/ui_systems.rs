use super::*;
use leocard_protocol::{PlayerId, ShengjiThrowFailureStage};
use leocard_qigui523::TimeControl;
use std::collections::HashMap;

#[test]
fn summary_animation_system_queries_initialize_without_conflicts() {
    let mut app = App::new();
    app.insert_resource(GameSummaryAnimation::default());
    app.add_systems(Update, animate_game_summary_visuals);

    app.update();
}

#[test]
fn time_control_options_follow_the_configured_order() {
    assert_eq!(previous_time_control(TimeControl::FivePlusTen), None);
    assert_eq!(
        next_time_control(TimeControl::FivePlusTen),
        Some(TimeControl::FivePlusThirty)
    );
    assert_eq!(
        next_time_control(TimeControl::FivePlusThirty),
        Some(TimeControl::FifteenPlusThirty)
    );
    assert_eq!(
        next_time_control(TimeControl::FifteenPlusThirty),
        Some(TimeControl::ThirtyPlusSixty)
    );
    assert_eq!(
        next_time_control(TimeControl::ThirtyPlusSixty),
        Some(TimeControl::Unlimited)
    );
    assert_eq!(
        previous_time_control(TimeControl::Unlimited),
        Some(TimeControl::ThirtyPlusSixty)
    );
    assert_eq!(next_time_control(TimeControl::Unlimited), None);
}

#[test]
fn play_error_toast_enters_upward_and_fades_out() {
    let mut toast = PlayErrorToast {
        active: true,
        entering: true,
        ..default()
    };
    let start = play_error_toast_visual(&toast);
    toast.elapsed = PLAY_ERROR_TOAST_ENTRY_DURATION;
    let entered = play_error_toast_visual(&toast);
    toast.elapsed = PLAY_ERROR_TOAST_DURATION;
    let finished = play_error_toast_visual(&toast);

    assert_eq!(start.opacity, 0.0);
    assert!(start.y > entered.y);
    assert!(entered.opacity > 0.99);
    assert_eq!(finished.opacity, 0.0);
}

#[test]
fn failed_throw_cards_reveal_split_rebound_and_return_to_the_player() {
    let direction = Vec2::new(0.0, 92.0);
    let stacked =
        shengji_failed_throw_card_visual(ShengjiThrowFailureStage::Showing, 0, 5, 0.0, direction);
    let revealed =
        shengji_failed_throw_card_visual(ShengjiThrowFailureStage::Showing, 0, 5, 0.24, direction);
    let split =
        shengji_failed_throw_card_visual(ShengjiThrowFailureStage::Showing, 0, 5, 0.54, direction);
    let rebounded =
        shengji_failed_throw_card_visual(ShengjiThrowFailureStage::Showing, 0, 5, 0.82, direction);
    let returned = shengji_failed_throw_card_visual(
        ShengjiThrowFailureStage::Returning,
        0,
        5,
        0.42,
        direction,
    );

    assert!(stacked.translation.x.abs() > revealed.translation.x.abs());
    assert!(split.translation.length() > revealed.translation.length());
    assert!(rebounded.translation.length() < split.translation.length());
    assert!(returned.translation.y > 80.0);
    assert!(!returned.visible);
}

#[test]
fn retriggered_play_error_toast_shakes_with_decay() {
    let mut toast = PlayErrorToast {
        active: true,
        shake_elapsed: Some(0.04),
        ..default()
    };
    let shaking = play_error_toast_visual(&toast);
    toast.shake_elapsed = Some(PLAY_ERROR_TOAST_SHAKE_DURATION);
    let settled = play_error_toast_visual(&toast);

    assert!(shaking.x.abs() > 1.0);
    assert_eq!(settled.x, 0.0);
}

#[test]
fn start_game_seats_smoothly_move_to_their_final_rectangles() {
    let start = start_game_seat_transition_visual(0.0);
    let moving = start_game_seat_transition_visual(START_GAME_SEAT_MOVE_DURATION * 0.5);
    let finished = start_game_seat_transition_visual(START_GAME_SEAT_MOVE_DURATION);

    assert_eq!(start.movement, 0.0);
    assert!(moving.movement > 0.0 && moving.movement < 1.0);
    assert!((moving.movement - 0.5).abs() < 0.00001);
    assert_eq!(finished.movement, 1.0);
}

#[test]
fn start_game_transition_can_be_attached_to_any_game_player_panel() {
    fn setup(mut commands: Commands) {
        let active = commands.spawn(Node::default()).id();
        attach_start_game_seat_transition(&mut commands, active, PlayerId(1), true);
        let settled = commands.spawn(Node::default()).id();
        attach_start_game_seat_transition(&mut commands, settled, PlayerId(2), false);
    }

    let mut app = App::new();
    app.add_systems(Startup, setup);
    app.update();

    let mut targets = app.world_mut().query::<(
        &GameSeatTransitionTarget,
        &GameSeatTransitionPose,
        &UiTransform,
        &Visibility,
    )>();
    let panels = targets
        .iter(app.world())
        .map(|(target, pose, transform, visibility)| {
            (target.0, (pose.initialized, *transform, *visibility))
        })
        .collect::<HashMap<_, _>>();
    assert_eq!(panels.len(), 2);
    assert_eq!(
        panels[&PlayerId(1)],
        (false, UiTransform::IDENTITY, Visibility::Hidden)
    );
    assert_eq!(
        panels[&PlayerId(2)],
        (false, UiTransform::IDENTITY, Visibility::Inherited)
    );
}

#[test]
fn start_game_seat_transition_system_queries_initialize_without_conflicts() {
    let mut app = App::new();
    app.insert_resource(Time::<()>::default());
    app.insert_resource(StartGameSeatTransition::default());
    app.add_systems(Update, animate_start_game_seat_transition);

    app.update();
}

#[test]
fn turn_border_trace_eases_in_and_out_around_the_whole_perimeter() {
    let perimeter = 100.0;
    let start = turn_border_visible_interval(0.0, perimeter);
    let early = turn_border_visible_interval(0.095, perimeter);
    let halfway_grown = turn_border_visible_interval(0.475, perimeter);
    let full = turn_border_visible_interval(1.0, perimeter);
    let halfway_shrunk = turn_border_visible_interval(1.565, perimeter);
    let gap = turn_border_visible_interval(2.1, perimeter);

    assert_eq!(start, (0.0, 0.0));
    assert!(early.1 > 0.0 && early.1 < 10.0);
    assert!((halfway_grown.1 - 50.0).abs() < 0.001);
    assert_eq!(full, (0.0, perimeter));
    assert!((halfway_shrunk.0 - 50.0).abs() < 0.001);
    assert_eq!(halfway_shrunk.1, perimeter);
    assert_eq!(gap, (perimeter, perimeter));
}

#[test]
fn turn_border_animation_system_queries_initialize_without_conflicts() {
    let mut app = App::new();
    app.insert_resource(Time::<()>::default());
    app.insert_resource(Assets::<TurnBorderMaterial>::default());
    app.insert_resource(TurnBorderAnimationState::default());
    app.add_systems(Update, animate_turn_border_traces);

    app.update();
}
