//! 无牌可压时，悬浮在出牌按钮上方的共用提示。

use crate::app::presentation::{TEXT, add_text};
use crate::app::runtime::UiAssets;
use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy::ui::VisualBox;
use leocard_ui::{UiComposer, sliced_image, spawn_node};

#[derive(Component)]
pub(crate) struct NoLegalResponseHint;

pub(crate) fn add_no_legal_response_hint(
    commands: &mut UiComposer,
    parent: Entity,
    assets: &UiAssets,
) -> Entity {
    let hint = spawn_node(
        commands,
        parent,
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            right: px(0),
            bottom: px(61),
            height: px(38),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        None,
    );
    commands.entity(hint).insert((
        NoLegalResponseHint,
        UiTransform::IDENTITY,
        GlobalZIndex(900),
        Pickable::IGNORE,
    ));
    let plate = spawn_node(
        commands,
        hint,
        Node {
            width: px(280),
            height: px(38),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        None,
    );
    let mut image = sliced_image(assets.home.game_card.clone(), 22.0, 0.42);
    image.visual_box = VisualBox::BorderBox;
    commands.entity(plate).insert((image, Pickable::IGNORE));
    let label = add_text(commands, plate, "没有牌能大过上家", 15.0, TEXT, assets);
    commands.entity(label).insert(Pickable::IGNORE);
    hint
}

pub(crate) fn animate_no_legal_response_hint(
    time: Res<Time>,
    mut hints: Query<&mut UiTransform, With<NoLegalResponseHint>>,
) {
    let offset = (time.elapsed_secs() * 3.2).sin() * 3.0;
    for mut transform in &mut hints {
        transform.translation = Val2::px(0.0, offset);
    }
}
