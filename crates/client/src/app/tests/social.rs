use super::*;
use leocard_protocol::PlayerInteractionKind;

#[test]
fn player_interaction_system_queries_initialize_without_conflicts() {
    let mut app = App::new();
    app.insert_resource(Time::<()>::default());
    app.insert_resource(UiAssets::default());
    app.insert_resource(UiState::default());
    app.insert_resource(ChatPanelState::default());
    app.insert_resource(ConnectionForm::default());
    app.insert_resource(PlayerInteractionCooldown::default());
    app.insert_resource(ScoreCaptureEffectState::default());
    app.insert_resource(ButtonInput::<MouseButton>::default());
    app.add_systems(
        Update,
        (
            tick_player_interaction_cooldown,
            close_interaction_menu_on_outside_click,
            sync_opponent_badge_popups,
            sync_interaction_cooldown_masks,
            sync_chat_messages,
            sync_player_interactions,
            sync_score_capture_effect,
            animate_player_interactions,
            animate_score_capture_effects,
            animate_chat_bubbles,
        )
            .chain(),
    );

    app.update();
}

#[test]
fn chat_bubbles_choose_the_inside_of_each_table_edge() {
    let layer = Vec2::new(1280.0, 720.0);
    let width = 200.0;
    let left = chat_bubble_position(Vec2::new(80.0, 360.0), layer, width);
    let right = chat_bubble_position(Vec2::new(1200.0, 360.0), layer, width);
    let top = chat_bubble_position(Vec2::new(640.0, 30.0), layer, width);

    assert!(left.x > 80.0);
    assert!(right.x + width < 1200.0);
    assert_eq!(top.x, 540.0);
    assert!(top.y >= 8.0);
}

#[test]
fn persistent_interaction_layer_is_not_owned_by_the_rebuilt_ui_root() {
    let mut app = App::new();
    app.add_systems(Startup, setup_camera);
    app.update();

    let layer = app
        .world_mut()
        .query_filtered::<Entity, With<PlayerInteractionLayer>>()
        .single(app.world())
        .unwrap();
    assert!(app.world().get::<ChildOf>(layer).is_none());
}

#[test]
fn shengji_settlement_animation_systems_initialize_without_conflicts() {
    let mut app = App::new();
    app.insert_resource(Time::<()>::default());
    app.insert_resource(UiAssets::default());
    app.insert_resource(ShengjiSettlementAnimation::default());
    app.add_systems(
        Update,
        (
            animate_shengji_settlement_visuals,
            spawn_shengji_settlement_absorption,
            animate_shengji_score_absorbs,
        )
            .chain(),
    );
    app.update();
}

#[test]
fn button_feedback_system_queries_initialize_without_conflicts() {
    let mut app = App::new();
    app.insert_resource(Time::<()>::default());
    app.insert_resource(UiAssets::default());
    app.add_systems(
        Update,
        (
            update_button_tints,
            play_button_click_sounds,
            animate_button_presses,
        )
            .chain(),
    );
    app.update();
}

#[test]
fn idle_button_animation_stops_writing_its_transform() {
    let mut app = App::new();
    app.insert_resource(Time::<()>::default());
    app.add_systems(Update, animate_button_presses);
    let button = app
        .world_mut()
        .spawn((Button, Interaction::None, UiTransform::IDENTITY))
        .id();

    app.update();
    app.world_mut().clear_trackers();
    app.update();

    assert!(
        !app.world()
            .entity(button)
            .get_ref::<UiTransform>()
            .unwrap()
            .is_changed()
    );
}

#[test]
fn settled_chat_panel_stops_writing_its_transform() {
    let mut app = App::new();
    app.insert_resource(Time::<()>::default());
    app.insert_resource(UiAssets::default());
    app.insert_resource(ChatPanelState::default());
    app.add_systems(Update, animate_chat_panel);
    let panel = app
        .world_mut()
        .spawn((ChatPanel, UiTransform::IDENTITY))
        .id();

    app.update();
    app.world_mut().clear_trackers();
    app.update();

    assert!(
        !app.world()
            .entity(panel)
            .get_ref::<UiTransform>()
            .unwrap()
            .is_changed()
    );
}

#[test]
fn interaction_cooldown_mask_reveals_itself_as_a_radial_sector() {
    let transparent = interaction_cooldown_mask_image(20, 20, 0.0);
    let half = interaction_cooldown_mask_image(20, 20, 0.5);
    let full = interaction_cooldown_mask_image(20, 20, 1.0);
    let covered = |image: &Image| {
        image
            .data
            .as_deref()
            .unwrap()
            .as_chunks::<4>()
            .0
            .iter()
            .filter(|pixel| pixel[3] != 0)
            .count()
    };

    assert_eq!(covered(&transparent), 0);
    assert_eq!(covered(&full), 400);
    assert!((190..=210).contains(&covered(&half)));
}

#[test]
fn every_player_interaction_has_an_independent_cooldown() {
    let mut cooldown = PlayerInteractionCooldown::default();
    cooldown.start(PlayerInteractionKind::Shoe, 5.0);

    assert!(cooldown.is_active(PlayerInteractionKind::Shoe));
    assert!(!cooldown.is_active(PlayerInteractionKind::Flower));
    assert!(!cooldown.is_active(PlayerInteractionKind::Egg));
    assert!(!cooldown.is_active(PlayerInteractionKind::Wine));

    cooldown.start(PlayerInteractionKind::Egg, 0.5);
    cooldown.tick(0.5);
    assert!(!cooldown.is_active(PlayerInteractionKind::Egg));
    assert!(cooldown.is_active(PlayerInteractionKind::Shoe));
    assert!((cooldown.fraction(PlayerInteractionKind::Shoe) - 0.9).abs() < f32::EPSILON);
}
