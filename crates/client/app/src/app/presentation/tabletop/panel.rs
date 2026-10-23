use crate::app::presentation::{UiComposer, spawn_node};
use bevy::picking::Pickable;
use bevy::prelude::*;

pub(crate) fn add_table_play_area(
    commands: &mut UiComposer,
    parent: Entity,
    mut node: Node,
) -> Entity {
    node.border = UiRect::all(px(1));
    node.border_radius = BorderRadius::all(px(11));
    node.overflow = Overflow::clip();
    let area = spawn_node(
        commands,
        parent,
        node,
        Some(Color::srgba(0.005, 0.018, 0.014, 0.26)),
    );
    commands.entity(area).insert((
        BackgroundGradient::from(LinearGradient::to_bottom_right(vec![
            ColorStop::auto(Color::srgba(0.10, 0.16, 0.13, 0.12)),
            ColorStop::auto(Color::srgba(0.04, 0.08, 0.06, 0.08)),
            ColorStop::auto(Color::BLACK.with_alpha(0.18)),
        ])),
        BorderGradient::from(LinearGradient::to_bottom_right(vec![
            ColorStop::auto(Color::srgba(0.24, 0.34, 0.29, 0.20)),
            ColorStop::auto(Color::srgba(0.10, 0.16, 0.13, 0.14)),
            ColorStop::auto(Color::BLACK.with_alpha(0.24)),
        ])),
        BoxShadow::new(Color::BLACK.with_alpha(0.28), px(0), px(3), px(0), px(8)),
        Pickable::IGNORE,
    ));
    area
}
