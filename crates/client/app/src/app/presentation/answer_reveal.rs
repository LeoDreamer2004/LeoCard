use super::{ACCENT, TEXT, add_text, add_title, ease_out_cubic};
use crate::app::runtime::UiAssets;
use bevy::picking::Pickable;
use bevy::prelude::*;
use leocard_ui::{UiComposer, spawn_node};

pub(crate) struct AnswerRevealSpec<'a> {
    pub answer: &'a str,
    pub caption: &'a str,
    pub icon: Option<ImageNode>,
}

pub(crate) fn add_answer_reveal(
    commands: &mut UiComposer,
    parent: Entity,
    spec: AnswerRevealSpec<'_>,
    frame: ImageNode,
    assets: &UiAssets,
) -> Entity {
    let banner = spawn_node(
        commands,
        parent,
        Node {
            position_type: PositionType::Absolute,
            left: percent(50),
            top: percent(34),
            max_width: percent(90),
            min_width: px(280),
            padding: UiRect::axes(px(26), px(20)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            column_gap: px(22),
            ..default()
        },
        None,
    );
    commands.entity(banner).insert((
        frame,
        UiTransform::from_translation(Val2::new(percent(-50), px(0))),
        ZIndex(10),
        Pickable::IGNORE,
    ));
    let centered = spec.icon.is_none();
    if let Some(icon) = spec.icon {
        commands.child(
            banner,
            (
                icon,
                Node {
                    width: px(96),
                    height: px(96),
                    flex_shrink: 0.0,
                    ..default()
                },
                Pickable::IGNORE,
            ),
        );
    }
    let labels = spawn_node(
        commands,
        banner,
        Node {
            flex_direction: FlexDirection::Column,
            row_gap: px(8),
            max_width: px(330),
            min_width: px(0),
            align_items: if centered {
                AlignItems::Center
            } else {
                AlignItems::Start
            },
            ..default()
        },
        None,
    );
    let answer = add_title(commands, labels, spec.answer, 30.0, ACCENT, assets);
    if centered {
        commands
            .entity(answer)
            .insert(TextLayout::justify(Justify::Center));
    }
    add_text(commands, labels, spec.caption, 18.0, TEXT, assets);
    banner
}

pub(crate) struct AnswerRevealMotion {
    pub transform: UiTransform,
    pub opacity: f32,
}

impl AnswerRevealMotion {
    pub fn new(elapsed: Option<f32>, duration: f32) -> Self {
        let elapsed = elapsed.unwrap_or(-1.0);
        let progress = ease_out_cubic((elapsed / 0.24).clamp(0.0, 1.0));
        Self {
            transform: UiTransform {
                translation: Val2::new(percent(-50), px(12.0 * (1.0 - progress))),
                scale: Vec2::splat(0.94 + 0.06 * progress),
                ..default()
            },
            opacity: (elapsed / 0.20).clamp(0.0, 1.0)
                * ((duration - elapsed) / 0.30).clamp(0.0, 1.0),
        }
    }
}
