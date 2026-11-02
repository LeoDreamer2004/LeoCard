use crate::{UiComposer, UiSceneKey, ui_component};
use bevy::picking::Pickable;
use bevy::prelude::*;

pub struct AvatarFrame {
    pub texture: Handle<Image>,
    pub size: f32,
}

pub struct AvatarFrameEntities {
    pub frame: Entity,
    pub avatar: Entity,
}

impl AvatarFrame {
    pub fn mount(
        self,
        commands: &mut UiComposer,
        parent: Entity,
        key: impl Into<String>,
        content: impl Scene,
    ) -> AvatarFrameEntities {
        let size = self.size;
        let texture = ImageNode::new(self.texture);
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
                    Node {
                        position_type: PositionType::Absolute,
                        left: percent(7.5), top: percent(7.5),
                        width: percent(85), height: percent(85),
                        border_radius: BorderRadius::all(percent(16)),
                        overflow: Overflow::clip(), align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                    }
                    Pickable::IGNORE
                    @content
                    --
                    UiSceneKey("frame")
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(0), right: px(0), top: px(0), bottom: px(0),
                    }
                    @ui_component(texture)
                    ZIndex(2)
                    Pickable::IGNORE
                ]
            },
        );
        let avatar = commands.entity_for(frame, "avatar");
        AvatarFrameEntities { frame, avatar }
    }
}
