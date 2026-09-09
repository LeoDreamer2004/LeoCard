use super::prelude::*;

#[test]
fn mahjong_action_animation_system_queries_initialize_without_conflicts() {
    let mut app = App::new();
    app.insert_resource(MahjongClaimPresentationState::default());
    app.insert_resource(Assets::<MahjongTileMaterial>::default());
    app.insert_resource(GameSummaryAnimation::default());
    app.add_systems(
        Update,
        (
            animate_mahjong_claim_presentation,
            animate_mahjong_flower_presentations,
            animate_mahjong_win_effects,
        ),
    );

    app.update();
}

#[test]
fn shengji_presentation_system_queries_initialize_without_conflicts() {
    let mut app = App::new();
    app.insert_resource(ShengjiPresentationState::default());
    app.add_systems(
        Update,
        (
            animate_shengji_presentation,
            animate_shengji_bottom_flip_markers,
            animate_shengji_power_outage_markers,
        ),
    );

    app.update();
}
