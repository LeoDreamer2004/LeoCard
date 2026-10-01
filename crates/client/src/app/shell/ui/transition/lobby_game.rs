//! Lobby panels leave before the game is revealed through a dark screen.

use super::layer::{add_screen_layer, shade_color};
use crate::app::presentation::{TransitionVisuals, UiRoot, ease_out_cubic, fade_panel};
use crate::app::runtime::ClientResource;
use crate::app::shell::UiState;
use bevy::prelude::*;

const GAME_EXIT_DURATION: f32 = 0.30;
const GAME_DARKEN_DURATION: f32 = 0.17;
const GAME_REVEAL_DURATION: f32 = 0.34;

#[derive(Component)]
pub(crate) struct LobbyTransitionPanel(pub u8);

#[derive(Component)]
pub(crate) struct LobbyGameHeader;

#[derive(Component)]
pub(crate) struct LobbyGameShade;

#[derive(Clone, Copy, Default, PartialEq, Eq)]
enum LobbyGameStage {
    #[default]
    Idle,
    Exiting,
    Darkening,
    Revealing,
}

#[derive(Resource, Default)]
pub(crate) struct LobbyGameMotion {
    stage: LobbyGameStage,
    elapsed: f32,
}

impl LobbyGameMotion {
    pub(crate) fn begin(&mut self) {
        self.stage = LobbyGameStage::Exiting;
        self.elapsed = 0.0;
    }

    pub(crate) fn active(&self) -> bool {
        self.stage != LobbyGameStage::Idle
    }

    pub(crate) fn cancel(&mut self) {
        *self = Self::default();
    }

    pub(crate) fn hold_lobby(&self) -> bool {
        matches!(
            self.stage,
            LobbyGameStage::Exiting | LobbyGameStage::Darkening
        )
    }

    fn shade_alpha(&self) -> f32 {
        match self.stage {
            LobbyGameStage::Idle | LobbyGameStage::Exiting => 0.0,
            LobbyGameStage::Darkening => {
                ease_out_cubic((self.elapsed / GAME_DARKEN_DURATION).clamp(0.0, 1.0))
            }
            LobbyGameStage::Revealing => {
                1.0 - ease_out_cubic((self.elapsed / GAME_REVEAL_DURATION).clamp(0.0, 1.0))
            }
        }
    }
}

pub(crate) fn add_lobby_game_shade(
    commands: &mut Commands,
    root: Entity,
    motion: &LobbyGameMotion,
) {
    let shade = add_screen_layer(commands, root, 1200, LobbyGameShade);
    commands
        .entity(shade)
        .insert(shade_color(motion.shade_alpha()));
}

#[expect(
    clippy::type_complexity,
    reason = "lobby panels, header and shade have disjoint mutable transforms"
)]
#[expect(
    clippy::too_many_arguments,
    reason = "Bevy injects independent screen and animation resources"
)]
pub(super) fn animate_lobby_game_motion(
    time: Res<Time>,
    mut motion: ResMut<LobbyGameMotion>,
    client: Option<Res<ClientResource>>,
    mut ui: ResMut<UiState>,
    mut commands: Commands,
    roots: Query<Entity, With<UiRoot>>,
    mut panels: Query<
        (Entity, &LobbyTransitionPanel, &mut UiTransform),
        (Without<LobbyGameHeader>, Without<LobbyGameShade>),
    >,
    mut headers: Query<
        (Entity, &mut UiTransform),
        (
            With<LobbyGameHeader>,
            Without<LobbyTransitionPanel>,
            Without<LobbyGameShade>,
        ),
    >,
    mut shades: Query<(Entity, &mut BackgroundColor), With<LobbyGameShade>>,
    children: Query<&Children>,
    mut visuals: TransitionVisuals<Without<LobbyGameShade>>,
) {
    if !motion.active() {
        for (entity, _) in &mut shades {
            commands.entity(entity).despawn();
        }
        return;
    }
    let game_ready = client
        .as_deref()
        .is_some_and(|client| client.0.model().active_game_meta().is_some());
    if !game_ready {
        motion.stage = LobbyGameStage::Idle;
        ui.dirty = true;
    } else {
        motion.elapsed += time.delta_secs();
        match motion.stage {
            LobbyGameStage::Exiting if motion.elapsed >= GAME_EXIT_DURATION => {
                motion.stage = LobbyGameStage::Darkening;
                motion.elapsed = 0.0;
            }
            LobbyGameStage::Darkening if motion.elapsed >= GAME_DARKEN_DURATION => {
                motion.stage = LobbyGameStage::Revealing;
                motion.elapsed = 0.0;
                ui.dirty = true;
            }
            LobbyGameStage::Revealing if motion.elapsed >= GAME_REVEAL_DURATION => {
                motion.stage = LobbyGameStage::Idle;
                ui.dirty = true;
            }
            _ => {}
        }
    }
    if motion.stage == LobbyGameStage::Idle {
        for (entity, _) in &mut shades {
            commands.entity(entity).despawn();
        }
        return;
    }
    if shades.is_empty()
        && let Some(root) = roots.iter().next()
    {
        add_lobby_game_shade(&mut commands, root, &motion);
    }
    for (_, mut background) in &mut shades {
        *background = shade_color(motion.shade_alpha());
    }
    if !motion.hold_lobby() {
        return;
    }
    let progress = if motion.stage == LobbyGameStage::Exiting {
        (motion.elapsed / GAME_EXIT_DURATION).clamp(0.0, 1.0)
    } else {
        1.0
    };
    let eased = ease_out_cubic(progress);
    for (entity, panel, mut transform) in &mut panels {
        let direction = if panel.0 == 0 { -1.0 } else { 1.0 };
        *transform = UiTransform::from_translation(Val2::px(direction * 420.0 * eased, 0.0));
        fade_panel(
            entity,
            1.0 - progress,
            &children,
            &mut visuals,
            &mut commands,
        );
    }
    for (entity, mut transform) in &mut headers {
        *transform = UiTransform::from_translation(Val2::px(0.0, -72.0 * eased));
        fade_panel(
            entity,
            1.0 - progress,
            &children,
            &mut visuals,
            &mut commands,
        );
    }
}
