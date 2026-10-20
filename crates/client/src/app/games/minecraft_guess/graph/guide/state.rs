use super::super::super::MinecraftGuessAssets;
use super::view::render_guide;
use crate::app::presentation::UiComposer;
use crate::app::runtime::{ClientResource, UiAssets};
use crate::app::shell::{CozyModalKind, ModalAnimations, advance_modal};
use bevy::input_focus::InputFocus;
use bevy::picking::Pickable;
use bevy::prelude::*;

#[derive(Resource, Default)]
pub(in super::super::super) struct GraphGuideState {
    open: bool,
    progress: f32,
}

impl GraphGuideState {
    pub fn toggle(&mut self) {
        self.open = !self.open;
    }

    pub fn close(&mut self) {
        self.open = false;
    }

    pub fn visible(&self) -> bool {
        self.open || self.progress > 0.0
    }
}

#[derive(Component)]
pub(super) struct GraphGuideScroll;

#[derive(Component)]
pub(super) struct GraphGuideRoot;

pub(super) fn advance_guide(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    client: Option<Res<ClientResource>>,
    mut guide: ResMut<GraphGuideState>,
    mut animations: ResMut<ModalAnimations>,
    mut focus: ResMut<InputFocus>,
) {
    let active = client
        .as_deref()
        .and_then(|client| client.0.model().minecraft_guess_game())
        .is_some_and(|game| game.graph.is_some());
    if active {
        if keyboard.just_pressed(KeyCode::Escape) {
            guide.close();
        }
        if guide.open {
            focus.clear();
        }
        let open = guide.open;
        advance_modal(&mut guide.progress, open, time.delta_secs());
    } else {
        *guide = GraphGuideState::default();
    }
    animations.set(CozyModalKind::MinecraftGraphGuide, guide.progress);
}

pub(super) fn sync_guide(
    mut commands: UiComposer,
    guide: Res<GraphGuideState>,
    roots: Query<Entity, With<GraphGuideRoot>>,
    assets: Res<UiAssets>,
    game_assets: Res<MinecraftGuessAssets>,
) {
    if !guide.visible() {
        for root in &roots {
            commands.entity(root).despawn();
        }
        return;
    }
    if !roots.is_empty() {
        return;
    }
    let root = commands
        .spawn((
            GraphGuideRoot,
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                right: px(0),
                top: px(0),
                bottom: px(0),
                ..default()
            },
            GlobalZIndex(2200),
            Pickable::IGNORE,
        ))
        .id();
    render_guide(&mut commands, root, guide.progress, &assets, &game_assets);
}
