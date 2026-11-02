//! Printed cards resting on felt, with projective mapping and a contact shadow.

use super::{PlayingCardFlip, TablePieceDeal};
use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::ShaderRef;
use leocard_ui::{UiComposer, spawn_node};

#[derive(AsBindGroup, Asset, TypePath, Debug, Clone, PartialEq)]
pub(crate) struct TableCardMaterial {
    /// Card width/height, far-edge width and corner radius, relative to the padded quad height.
    #[uniform(0)]
    shape: Vec4,
    #[uniform(0)]
    pub tint: LinearRgba,
    #[texture(1)]
    #[sampler(2)]
    pub texture: Handle<Image>,
}

impl UiMaterial for TableCardMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/table_card.wesl".into()
    }
}

pub(crate) struct TableCard {
    pub texture: Handle<Image>,
    pub size: Vec2,
    pub opacity: f32,
}

impl TableCard {
    /// Returns the visible surface; animations act on this node without relaying out its slot.
    #[track_caller]
    pub(crate) fn spawn(
        self,
        commands: &mut UiComposer,
        parent: Entity,
        mut node: Node,
        materials: &mut Assets<TableCardMaterial>,
    ) -> Entity {
        node.width = px(self.size.x);
        node.height = px(self.size.y);
        node.flex_shrink = 0.0;
        let slot = spawn_node(commands, parent, node, None);
        commands.entity(slot).insert(Pickable::IGNORE);
        let padding = self.size.y * 0.1;
        let surface = spawn_node(
            commands,
            slot,
            Node {
                position_type: PositionType::Absolute,
                left: px(-padding),
                top: px(-padding),
                right: px(-padding),
                bottom: px(-padding),
                ..default()
            },
            None,
        );
        let quad_height = self.size.y + padding * 2.0;
        let material = commands.material(
            surface,
            materials,
            TableCardMaterial {
                shape: Vec4::new(
                    self.size.x / quad_height,
                    self.size.y / quad_height,
                    0.96,
                    self.size.x * 0.065 / quad_height,
                ),
                tint: LinearRgba::WHITE.with_alpha(self.opacity),
                texture: self.texture,
            },
        );
        commands.entity(surface).insert((
            MaterialNode(material),
            UiTransform::IDENTITY,
            Pickable::IGNORE,
        ));
        surface
    }
}

pub(crate) fn animate_table_card_flips(
    time: Res<Time>,
    mut materials: ResMut<Assets<TableCardMaterial>>,
    mut cards: Query<(
        &mut PlayingCardFlip,
        Option<&TablePieceDeal>,
        &mut UiTransform,
        &MaterialNode<TableCardMaterial>,
    )>,
) {
    for (mut flip, deal, mut transform, material) in &mut cards {
        if let Some(frame) = flip.advance(time.delta_secs(), deal)
            && let Some(mut material) = materials.get_mut(&material.0)
        {
            material.texture = frame.texture;
            transform.scale = frame.scale;
        }
    }
}
