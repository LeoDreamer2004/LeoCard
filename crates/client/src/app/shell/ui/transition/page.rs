//! Shared page motion, including a full-screen fade when returning from a game.

use super::layer::{add_screen_layer, shade_color};
use super::lobby_game::LobbyGameShade;
use crate::app::presentation::{TransitionVisuals, UiRoot, ease_out_cubic, fade_panel};
use crate::app::runtime::UiAssets;
use crate::app::shell::UiState;
use bevy::prelude::*;

const PAGE_STAGGER: f32 = 0.09;
const PAGE_EXIT_DURATION: f32 = 0.24;
const PAGE_ENTER_DURATION: f32 = 0.38;
const RETURN_DARKEN_DURATION: f32 = 0.17;

/// Publish the page's current transform and opacity before animated visuals read them.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, SystemSet)]
pub(crate) struct PageTransitionSet;

#[derive(Clone, Copy, Default, PartialEq, Eq)]
enum PageStage {
    #[default]
    Idle,
    Exiting,
    Darkening,
    Entering,
}

#[derive(Resource, Default)]
pub(crate) struct PageMotion {
    stage: PageStage,
    elapsed: f32,
    played_sounds: u64,
    returning: bool,
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
        matches!(self.stage, PageStage::Exiting | PageStage::Darkening)
    }

    pub(crate) fn begin_return(&mut self) {
        *self = Self {
            stage: PageStage::Darkening,
            returning: true,
            ..default()
        };
    }

    pub(crate) fn cancel(&mut self) {
        *self = Self::default();
    }

    fn shade_alpha(&self) -> f32 {
        if !self.returning {
            return 0.0;
        }
        match self.stage {
            PageStage::Darkening => {
                ease_out_cubic((self.elapsed / RETURN_DARKEN_DURATION).clamp(0.0, 1.0))
            }
            PageStage::Entering => {
                1.0 - ease_out_cubic((self.elapsed / PAGE_ENTER_DURATION).clamp(0.0, 1.0))
            }
            _ => 0.0,
        }
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
pub(super) struct PageTransitionBlocker;

#[derive(Component)]
pub(super) struct PageTransitionShade;

pub(super) fn page_content_can_animate(motion: Res<PageMotion>) -> bool {
    motion.stage != PageStage::Darkening
}

pub(crate) fn add_page_transition_shade(
    commands: &mut Commands,
    root: Entity,
    motion: &PageMotion,
) {
    if motion.returning {
        // Cover game effects, dialogs and notifications as well as the table.
        let shade = add_screen_layer(commands, root, 3000, PageTransitionShade);
        commands
            .entity(shade)
            .insert(shade_color(motion.shade_alpha()));
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "Bevy injects independent page, audio, and visual resources"
)]
pub(super) fn animate_page_motion(
    time: Res<Time>,
    mut motion: ResMut<PageMotion>,
    mut ui: ResMut<UiState>,
    assets: Res<UiAssets>,
    mut commands: Commands,
    roots: Query<Entity, With<UiRoot>>,
    blockers: Query<Entity, With<PageTransitionBlocker>>,
    mut shades: Query<(Entity, &mut BackgroundColor), With<PageTransitionShade>>,
    mut elements: Query<(Entity, &PageTransitionElement, &mut UiTransform)>,
    children: Query<&Children>,
    mut visuals: TransitionVisuals<(Without<LobbyGameShade>, Without<PageTransitionShade>)>,
) {
    if !motion.active() {
        for entity in &blockers {
            commands.entity(entity).despawn();
        }
        for (entity, _) in &mut shades {
            commands.entity(entity).despawn();
        }
        return;
    }
    if blockers.is_empty()
        && let Some(root) = roots.iter().next()
    {
        add_screen_layer(&mut commands, root, 2000, PageTransitionBlocker);
    }
    motion.elapsed += time.delta_secs();
    if shades.is_empty()
        && let Some(root) = roots.iter().next()
    {
        add_page_transition_shade(&mut commands, root, &motion);
    }
    for (_, mut background) in &mut shades {
        *background = shade_color(motion.shade_alpha());
    }
    if motion.stage == PageStage::Darkening {
        if motion.elapsed >= RETURN_DARKEN_DURATION {
            motion.stage = PageStage::Entering;
            motion.elapsed = 0.0;
            ui.dirty = true;
        }
        return;
    }
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
            PageStage::Idle | PageStage::Darkening => 1.0,
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
            motion.cancel();
            ui.dirty = true;
        }
        _ => {}
    }
}
