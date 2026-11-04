//! A resizable page shell with independently laid out heading and inset content.

use crate::{UiComposer, decorate_panel, spawn_node};
use bevy::prelude::*;

pub struct PageFrameSkin {
    pub shell: ImageNode,
    pub heading: ImageNode,
    pub inset: ImageNode,
}

pub struct PageFrame {
    pub node: Node,
    pub heading: Node,
    pub content: Node,
    pub skin: PageFrameSkin,
}

pub struct PageFrameEntities {
    pub panel: Entity,
    pub heading: Entity,
    pub content: Entity,
}

impl PageFrame {
    /// Geometry is in logical UI units. Change `heading` and `content` independently
    /// of the shell size; neither section participates in the image's intrinsic layout.
    pub fn new(skin: PageFrameSkin) -> Self {
        Self {
            node: Node {
                min_width: px(0),
                min_height: px(0),
                padding: UiRect::all(px(8)),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            heading: Node {
                height: px(42),
                flex_shrink: 0.0,
                padding: UiRect::horizontal(px(8)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
            content: Node {
                min_width: px(0),
                min_height: px(0),
                flex_grow: 1.0,
                padding: UiRect::all(px(14)),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            skin,
        }
    }

    #[track_caller]
    pub fn mount(self, commands: &mut UiComposer, parent: Entity) -> PageFrameEntities {
        let panel = spawn_node(commands, parent, self.node, None);
        decorate_panel(commands, panel, self.skin.shell);
        let heading = spawn_node(commands, panel, self.heading, None);
        decorate_section(commands, heading, self.skin.heading);
        let content = spawn_node(commands, panel, self.content, None);
        decorate_section(commands, content, self.skin.inset);
        PageFrameEntities {
            panel,
            heading,
            content,
        }
    }
}

fn decorate_section(commands: &mut UiComposer, section: Entity, image: ImageNode) {
    let skin = decorate_panel(commands, section, image);
    // Overlap the chrome by one logical pixel so fractional UI scaling cannot
    // expose a sampling seam along the transparent opening.
    commands
        .entity(skin)
        .entry::<Node>()
        .and_modify(|mut node| {
            node.left = px(-1);
            node.right = px(-1);
            node.top = px(-1);
            node.bottom = px(-1);
        });
}
