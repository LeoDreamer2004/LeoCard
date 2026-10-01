//! Shared page exits and entrances, plus the lobby's darker game entrance.

use super::super::{UiState, update_achievement_category_hover};
use crate::app::presentation::{
    TransitionVisuals, UiRoot, animate_button_arrows, ease_out_cubic, fade_panel,
    update_button_highlights,
};
use crate::app::runtime::{ClientResource, ClientUpdateSet, UiAssets};
use bevy::prelude::*;
use bevy::ui::FocusPolicy;

const PAGE_STAGGER: f32 = 0.09;
const PAGE_EXIT_DURATION: f32 = 0.24;
const PAGE_ENTER_DURATION: f32 = 0.38;
const GAME_EXIT_DURATION: f32 = 0.30;
const GAME_DARKEN_DURATION: f32 = 0.17;
const GAME_REVEAL_DURATION: f32 = 0.34;

pub(crate) struct PageTransitionPlugin;

impl Plugin for PageTransitionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PageMotion>()
            .init_resource::<LobbyGameMotion>()
            .add_systems(
                Update,
                (
                    animate_page_motion
                        .after(update_achievement_category_hover)
                        .after(update_button_highlights)
                        .after(animate_button_arrows),
                    animate_lobby_game_motion,
                )
                    .in_set(ClientUpdateSet::Animate),
            );
    }
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
enum PageStage {
    #[default]
    Idle,
    Exiting,
    Entering,
}

#[derive(Resource, Default)]
pub(crate) struct PageMotion {
    stage: PageStage,
    elapsed: f32,
    played_sounds: u64,
}

impl PageMotion {
    pub(crate) fn begin(&mut self) {
        if !self.active() {
            self.stage = PageStage::Exiting;
            self.elapsed = 0.0;
            self.played_sounds = 0;
        }
    }

    pub(crate) fn active(&self) -> bool {
        self.stage != PageStage::Idle
    }

    pub(crate) fn hold_page(&self) -> bool {
        self.stage == PageStage::Exiting
    }

    pub(crate) fn cancel(&mut self) {
        *self = Self::default();
    }
}

/// Mark independent page groups; their complete visual subtree fades together.
#[derive(Component)]
pub(crate) struct PageTransitionElement {
    order: u8,
    exit_offset: Vec2,
    sound: bool,
}

impl PageTransitionElement {
    pub(crate) fn left(order: u8) -> Self {
        Self {
            order,
            exit_offset: Vec2::new(-390.0, 0.0),
            sound: true,
        }
    }

    pub(crate) fn right(order: u8) -> Self {
        Self {
            order,
            exit_offset: Vec2::new(390.0, 0.0),
            sound: true,
        }
    }

    pub(crate) fn header() -> Self {
        Self {
            order: 0,
            exit_offset: Vec2::new(0.0, -72.0),
            sound: false,
        }
    }
}

#[derive(Component)]
struct PageTransitionBlocker;

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
    let shade = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                right: px(0),
                top: px(0),
                bottom: px(0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.025, 0.022, 0.045, motion.shade_alpha())),
            GlobalZIndex(1200),
            FocusPolicy::Block,
            LobbyGameShade,
        ))
        .id();
    commands.entity(root).add_child(shade);
}

#[expect(
    clippy::too_many_arguments,
    reason = "Bevy injects independent page, audio, and visual resources"
)]
fn animate_page_motion(
    time: Res<Time>,
    mut motion: ResMut<PageMotion>,
    mut ui: ResMut<UiState>,
    assets: Res<UiAssets>,
    mut commands: Commands,
    roots: Query<Entity, With<UiRoot>>,
    blockers: Query<Entity, With<PageTransitionBlocker>>,
    mut elements: Query<(Entity, &PageTransitionElement, &mut UiTransform)>,
    children: Query<&Children>,
    mut visuals: TransitionVisuals<Without<LobbyGameShade>>,
) {
    if !motion.active() {
        for entity in &blockers {
            commands.entity(entity).despawn();
        }
        return;
    }
    if blockers.is_empty()
        && let Some(root) = roots.iter().next()
    {
        let blocker = commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: px(0),
                    right: px(0),
                    top: px(0),
                    bottom: px(0),
                    ..default()
                },
                GlobalZIndex(2000),
                FocusPolicy::Block,
                PageTransitionBlocker,
            ))
            .id();
        commands.entity(root).add_child(blocker);
    }
    motion.elapsed += time.delta_secs();
    let mut exit_end = PAGE_EXIT_DURATION;
    for (entity, element, mut transform) in &mut elements {
        let delay = f32::from(element.order) * PAGE_STAGGER;
        exit_end = exit_end.max(delay + PAGE_EXIT_DURATION);
        let opacity = match motion.stage {
            PageStage::Exiting => {
                if element.sound && motion.elapsed >= delay {
                    let bit = 1_u64 << element.order;
                    if motion.played_sounds & bit == 0 {
                        if let Some(sound) =
                            assets.audio.deal_sounds.get(usize::from(element.order))
                        {
                            commands.spawn((
                                AudioPlayer::new(sound.clone()),
                                PlaybackSettings::DESPAWN,
                            ));
                        }
                        motion.played_sounds |= bit;
                    }
                }
                let progress = ((motion.elapsed - delay) / PAGE_EXIT_DURATION).clamp(0.0, 1.0);
                *transform = UiTransform::from_translation(Val2::px(
                    element.exit_offset.x * ease_out_cubic(progress),
                    element.exit_offset.y * ease_out_cubic(progress),
                ));
                1.0 - progress
            }
            PageStage::Entering => {
                let eased = ease_out_cubic((motion.elapsed / PAGE_ENTER_DURATION).clamp(0.0, 1.0));
                *transform = UiTransform::from_translation(Val2::px(0.0, -135.0 * (1.0 - eased)));
                eased
            }
            PageStage::Idle => 1.0,
        };
        fade_panel(entity, opacity, &children, &mut visuals, &mut commands);
    }
    // Switch only after this frame's old elements have fully disappeared.
    match motion.stage {
        PageStage::Exiting if motion.elapsed >= exit_end => {
            motion.stage = PageStage::Entering;
            motion.elapsed = 0.0;
            ui.dirty = true;
        }
        PageStage::Entering if motion.elapsed >= PAGE_ENTER_DURATION => {
            motion.stage = PageStage::Idle;
            ui.dirty = true;
        }
        _ => {}
    }
}

#[expect(
    clippy::type_complexity,
    reason = "lobby panels, header and shade have disjoint mutable transforms"
)]
#[expect(
    clippy::too_many_arguments,
    reason = "Bevy injects independent screen and animation resources"
)]
fn animate_lobby_game_motion(
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
        background.0 = Color::srgba(0.025, 0.022, 0.045, motion.shade_alpha());
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
