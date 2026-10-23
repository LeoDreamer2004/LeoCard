use crate::app::presentation::{TEXT, UiComposer, add_text, spawn_node};
use crate::app::runtime::UiAssets;
use bevy::picking::Pickable;
use bevy::prelude::*;

#[derive(Component)]
pub(in super::super) struct GenerationSpinner;

pub(super) fn render_generation_notice(commands: &mut UiComposer, pane: Entity, assets: &UiAssets) {
    let content = spawn_node(
        commands,
        pane,
        Node {
            width: percent(100),
            height: percent(100),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            column_gap: px(16),
            ..default()
        },
        None,
    );
    commands.entity(content).insert(Pickable::IGNORE);
    let spinner = spawn_node(
        commands,
        content,
        Node {
            width: px(26),
            height: px(26),
            border: UiRect::all(px(3)),
            border_radius: BorderRadius::all(percent(50)),
            flex_shrink: 0.0,
            ..default()
        },
        None,
    );
    commands.entity(spinner).insert((
        GenerationSpinner,
        Pickable::IGNORE,
        BorderColor {
            top: Color::srgb(0.74, 0.81, 0.59),
            right: Color::srgba(0.4, 0.55, 0.43, 0.3),
            bottom: Color::srgba(0.4, 0.55, 0.43, 0.3),
            left: Color::srgba(0.4, 0.55, 0.43, 0.3),
        },
    ));
    add_text(commands, content, "图谱生成中…", 21.0, TEXT, assets);
}

pub(in super::super) fn animate_generation_notice(
    time: Res<Time>,
    mut spinners: Query<&mut UiTransform, With<GenerationSpinner>>,
) {
    for mut spinner in &mut spinners {
        spinner.rotation = Rot2::radians(time.elapsed_secs() * 3.0);
    }
}
