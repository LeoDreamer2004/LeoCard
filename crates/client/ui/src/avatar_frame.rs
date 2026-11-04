use crate::{UiComposer, UiSceneKey, ui_component};
use bevy::picking::Pickable;
use bevy::prelude::*;

pub struct AvatarFrame {
    pub texture: Handle<Image>,
    pub size: f32,
    pub layout: AvatarFrameLayout,
}

pub struct AvatarFrameLayout {
    /// Texture placement relative to the logical frame, including any protruding ornaments.
    pub artwork: Rect,
    /// Portrait placement relative to the logical frame.
    pub portrait: Rect,
}

impl Default for AvatarFrameLayout {
    fn default() -> Self {
        Self {
            artwork: Rect::new(0.0, 0.0, 1.0, 1.0),
            portrait: Rect::new(0.075, 0.075, 0.925, 0.925),
        }
    }
}

pub struct AvatarFrameEntities {
    pub frame: Entity,
    pub avatar: Entity,
}

impl AvatarFrame {
    pub fn mount_preview(self, commands: &mut UiComposer, parent: Entity) -> Entity {
        let size = self.size;
        let artwork = self.artwork();
        commands.node(
            parent,
            bsn! {
                Node {
                    width: px(size), height: px(size), flex_shrink: 0.0,
                    position_type: PositionType::Relative,
                }
                Pickable::IGNORE
                Children [ @artwork ]
            },
        )
    }

    pub fn mount(
        self,
        commands: &mut UiComposer,
        parent: Entity,
        key: impl Into<String>,
        content: impl Scene,
    ) -> AvatarFrameEntities {
        let size = self.size;
        let mut portrait = placed_node(self.layout.portrait, size);
        let artwork = self.artwork();
        portrait.border_radius = BorderRadius::all(px(size * 0.136));
        portrait.overflow = Overflow::clip();
        portrait.align_items = AlignItems::Center;
        portrait.justify_content = JustifyContent::Center;
        let shadow = BoxShadow(vec![
            ShadowStyle {
                color: Color::BLACK.with_alpha(0.22),
                x_offset: px(0),
                y_offset: px(4),
                spread_radius: px(0),
                blur_radius: px(7),
            },
            ShadowStyle {
                color: Color::BLACK.with_alpha(0.38),
                x_offset: px(0),
                y_offset: px(2),
                spread_radius: px(0),
                blur_radius: px(2.5),
            },
        ]);
        let frame = commands.mount(
            parent,
            key,
            bsn! {
                Node {
                    width: px(size), height: px(size), min_width: px(size), min_height: px(size),
                    flex_shrink: 0.0, position_type: PositionType::Relative,
                    border_radius: BorderRadius::all(percent(20)),
                }
                @ui_component(shadow)
                Pickable::IGNORE
                Children [
                    UiSceneKey("avatar")
                    portrait
                    Pickable::IGNORE
                    @content
                    --
                    @artwork
                ]
            },
        );
        let avatar = commands.entity_for(frame, "avatar");
        AvatarFrameEntities { frame, avatar }
    }

    fn artwork(self) -> impl Scene {
        let artwork = placed_node(self.layout.artwork, self.size);
        let texture = ImageNode::new(self.texture);
        bsn! {
            UiSceneKey("frame")
            artwork
            @ui_component(texture)
            ZIndex(2)
            Pickable::IGNORE
        }
    }
}

fn placed_node(rect: Rect, size: f32) -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: px(rect.min.x * size),
        top: px(rect.min.y * size),
        width: px(rect.width() * size),
        height: px(rect.height() * size),
        ..default()
    }
}
