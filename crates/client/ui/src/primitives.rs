use crate::UiComposer;
use bevy::prelude::*;

#[track_caller]
pub fn spawn_node(
    commands: &mut UiComposer,
    parent: Entity,
    node: Node,
    background: Option<Color>,
) -> Entity {
    let background = background.map(|color| bsn! { BackgroundColor(color) });
    commands.node(parent, bsn! { node @background })
}
