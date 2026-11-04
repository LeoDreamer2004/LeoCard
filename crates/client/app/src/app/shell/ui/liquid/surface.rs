use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;
use leocard_ui::{TransitionOpacity, UiComposer};

#[derive(Clone, Copy)]
pub(crate) enum LiquidKind {
    Card,
    Accent,
    JoinSweep,
}

#[derive(Component)]
pub(crate) struct LiquidSurface {
    pub(super) owner: Entity,
    pub(super) hit: Entity,
    pub(super) texture: Handle<Image>,
    pub(super) tint: Color,
    pub(super) kind: LiquidKind,
    pub(super) phase: f32,
}

impl LiquidSurface {
    pub(crate) fn mount(
        commands: &mut UiComposer,
        parent: Entity,
        owner: Entity,
        texture: Handle<Image>,
        tint: Color,
        kind: LiquidKind,
        phase: f32,
    ) -> Entity {
        commands
            .entity(owner)
            .insert(RelativeCursorPosition::default());
        if parent != owner {
            commands
                .entity(parent)
                .insert(RelativeCursorPosition::default());
        }
        commands.child(
            parent,
            (
                Node {
                    position_type: PositionType::Absolute,
                    left: px(0),
                    right: px(0),
                    top: px(0),
                    bottom: px(0),
                    ..default()
                },
                Self {
                    owner,
                    hit: parent,
                    texture,
                    tint,
                    kind,
                    phase,
                },
                TransitionOpacity::default(),
                UiTransform::IDENTITY,
                Pickable::IGNORE,
            ),
        )
    }

    pub(crate) fn set_rect(commands: &mut UiComposer, surface: Entity, rect: Vec4) {
        commands.entity(surface).insert(LiquidUvRect(rect));
    }
}

#[derive(Component)]
pub(super) struct LiquidUvRect(pub Vec4);
