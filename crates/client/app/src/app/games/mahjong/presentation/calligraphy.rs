use bevy::image::{ImageLoaderSettings, ImageSampler};
use bevy::picking::Pickable;
use bevy::prelude::*;
use leocard_ui::UiComposer;

pub(crate) struct MahjongCalligraphyAssets {
    glyphs: [Handle<Image>; 5],
    ink: Handle<Image>,
}

#[derive(Clone, Copy)]
pub(crate) enum MahjongCalligraphyKind {
    Chow,
    Pung,
    Kong,
    Win,
    SelfDraw,
}

impl MahjongCalligraphyKind {
    const fn index(self) -> usize {
        match self {
            Self::Chow => 0,
            Self::Pung => 1,
            Self::Kong => 2,
            Self::Win => 3,
            Self::SelfDraw => 4,
        }
    }

    const fn glyph_size(self) -> Vec2 {
        match self {
            Self::Chow | Self::Pung | Self::Kong => Vec2::splat(76.0),
            Self::Win => Vec2::splat(88.0),
            Self::SelfDraw => Vec2::new(162.8, 88.0),
        }
    }

    const fn ink_size(self) -> Option<Vec2> {
        match self {
            Self::Chow | Self::Pung | Self::Kong => None,
            Self::Win => Some(Vec2::new(152.0, 94.0)),
            Self::SelfDraw => Some(Vec2::new(214.0, 94.0)),
        }
    }
}

#[derive(Component)]
pub(crate) struct MahjongCalligraphy {
    glyph: Entity,
    ink: Option<Entity>,
}

impl MahjongCalligraphy {
    pub(super) fn set_alpha(&self, alpha: f32, images: &mut Query<&mut ImageNode>) {
        for entity in [Some(self.glyph), self.ink].into_iter().flatten() {
            if let Ok(mut image) = images.get_mut(entity) {
                image.color = Color::WHITE.with_alpha(alpha);
            }
        }
    }
}

impl MahjongCalligraphyAssets {
    pub(crate) fn load(server: &AssetServer) -> Self {
        let load = |name| {
            server
                .load_builder()
                .with_settings(|settings: &mut ImageLoaderSettings| {
                    settings.sampler = ImageSampler::linear();
                })
                .load(format!("ui/mahjong/calligraphy/{name}.png"))
        };
        Self {
            glyphs: ["chow", "pung", "kong", "win", "self-draw"].map(load),
            ink: load("ink"),
        }
    }

    pub(super) fn spawn(
        &self,
        commands: &mut UiComposer,
        parent: Entity,
        kind: MahjongCalligraphyKind,
        center: Vec2,
        alpha: f32,
    ) -> Entity {
        let size = kind.glyph_size();
        let holder = commands.child(
            parent,
            (
                Node {
                    position_type: PositionType::Absolute,
                    left: px(center.x - size.x * 0.5),
                    top: px(center.y - size.y * 0.5),
                    width: px(size.x),
                    height: px(size.y),
                    overflow: Overflow::visible(),
                    ..default()
                },
                Pickable::IGNORE,
            ),
        );
        let ink = kind.ink_size().map(|ink_size| {
            spawn_layer(commands, holder, self.ink.clone(), size, ink_size, alpha, 0)
        });
        let glyph = spawn_layer(
            commands,
            holder,
            self.glyphs[kind.index()].clone(),
            size,
            size,
            alpha,
            1,
        );
        commands
            .entity(holder)
            .insert(MahjongCalligraphy { glyph, ink });
        holder
    }
}

fn spawn_layer(
    commands: &mut UiComposer,
    parent: Entity,
    image: Handle<Image>,
    holder_size: Vec2,
    size: Vec2,
    alpha: f32,
    z_index: i32,
) -> Entity {
    commands.child(
        parent,
        (
            Node {
                position_type: PositionType::Absolute,
                left: px((holder_size.x - size.x) * 0.5),
                top: px((holder_size.y - size.y) * 0.5),
                width: px(size.x),
                height: px(size.y),
                ..default()
            },
            ImageNode {
                color: Color::WHITE.with_alpha(alpha),
                ..ImageNode::new(image)
            },
            ZIndex(z_index),
            Pickable::IGNORE,
        ),
    )
}
