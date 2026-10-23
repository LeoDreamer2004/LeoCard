//! Texture buttons share their hierarchy, highlight binding and disabled state.

use super::text_scene;
use crate::app::presentation::{ButtonHighlight, UiComposer, UiSceneKey, ui_component};
use crate::app::runtime::UiAssets;
use crate::app::shell::UiAction;
use bevy::ecs::template::EntityTemplate;
use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy::ui_widgets::Button;

pub(crate) struct TextureButton<'a> {
    pub node: Node,
    pub normal: ImageNode,
    pub hovered: ImageNode,
    pub action: Option<UiAction>,
    pub label: Option<&'a str>,
    pub font_size: f32,
    pub color: Color,
    pub icon: Option<(Handle<Image>, f32)>,
}

impl TextureButton<'_> {
    #[track_caller]
    pub(crate) fn mount(
        self,
        commands: &mut UiComposer,
        parent: Entity,
        assets: &UiAssets,
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
            .map(|label| text_scene(assets, label, self.font_size, self.color));
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
