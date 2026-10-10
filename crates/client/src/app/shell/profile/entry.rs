//! 个人资料的头像入口。
use crate::app::presentation::{add_text, avatar_color};
use crate::app::runtime::{AvatarImages, ConnectionDraft, UiAssets};
use crate::app::shell::{NavigationUiAction, UiAction};
use bevy::prelude::*;
use bevy::ui_widgets::Button;

pub(crate) struct ProfileEntry<'a> {
    pub form: &'a ConnectionDraft,
    pub assets: &'a UiAssets,
    pub avatars: &'a AvatarImages,
}
impl ProfileEntry<'_> {
    pub(crate) fn render(
        &self,
        commands: &mut Commands,
        parent: Entity,
        size: f32,
        fallback_font_size: f32,
    ) {
        let mut entity = commands.spawn((
            Button,
            UiAction::Navigation(NavigationUiAction::ToggleProfile),
            Node {
                width: px(size),
                height: px(size),
                min_width: px(size),
                border: UiRect::all(px(2)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border_radius: BorderRadius::all(percent(50)),
                overflow: Overflow::clip(),
                ..default()
            },
            UiTransform::IDENTITY,
        ));
        entity.insert(BorderColor::all(Color::srgb(0.68, 0.65, 0.85)));
        if let Some(image) = self.avatars.local.as_ref() {
            entity.insert(ImageNode::new(image.clone()));
        } else {
            entity.insert(BackgroundColor(avatar_color(&self.form.player_name)));
        }
        let button = entity.id();
        commands.entity(parent).add_child(button);
        if self.avatars.local.is_none() {
            add_text(
                commands,
                button,
                self.form
                    .player_name
                    .chars()
                    .next()
                    .unwrap_or('玩')
                    .to_string(),
                fallback_font_size,
                Color::WHITE,
                self.assets,
            );
        }
    }
}
