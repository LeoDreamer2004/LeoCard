//! Texture buttons share their hierarchy, highlight binding and disabled state.

use super::ButtonHighlight;
use crate::{UiComposer, UiSceneKey, label_scene, ui_component};
use bevy::ecs::template::EntityTemplate;
use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy::ui_widgets::Button;

pub struct TextureButton<'a, A> {
    pub node: Node,
    pub normal: ImageNode,
    pub hovered: ImageNode,
    pub action: Option<A>,
    pub label: Option<&'a str>,
    pub font_size: f32,
    pub color: Color,
    pub icon: Option<(Handle<Image>, f32)>,
}

impl<A: Component + Clone> TextureButton<'_, A> {
    #[track_caller]
    pub fn mount(
        self,
        commands: &mut UiComposer,
        parent: Entity,
        font: &Handle<Font>,
    ) -> (Entity, Entity) {
        let enabled = self.action.is_some();
        let action = self
            .action
            .map(|action| bsn! { Button @ui_component(action) });
        let binding = move |overlay: EntityTemplate| {
            enabled.then(|| bsn! { ButtonHighlight { overlay: overlay, arrows: None } })
        };
        let hover = enabled.then(|| {
            bsn! {
                Node {
                    position_type: PositionType::Absolute,
                    left: px(0), right: px(0), top: px(0), bottom: px(0),
                }
                @ui_component(self.hovered)
                Visibility::Hidden
                Pickable::IGNORE
            }
        });
        let icon = self.icon.map(|(image, size)| {
            bsn! {
                Node { width: px(size), height: px(size) }
                @ui_component(ImageNode::new(image))
                Pickable::IGNORE
            }
        });
        let label = self
            .label
            .map(|label| label_scene(font.clone(), label, self.font_size, self.color));
        let pickable = if enabled {
            Pickable::default()
        } else {
            Pickable::IGNORE
        };
        let button = commands.node(
            parent,
            bsn! {
                @ui_component(self.node)
                @ui_component(self.normal)
                @ui_component(pickable)
                @action
                @binding(#Hover)
                Children [
                    #Hover UiSceneKey("hover") @hover
                    --
                    UiSceneKey("icon") @icon
                    --
                    UiSceneKey("label") @label Pickable::IGNORE
                ]
            },
        );
        (button, commands.entity_for(button, "label"))
    }
}
