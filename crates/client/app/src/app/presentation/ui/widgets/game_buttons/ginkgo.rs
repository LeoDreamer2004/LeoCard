use super::entrance::mount_game_button;
use crate::app::runtime::UiAssets;
use crate::app::shell::UiAction;
use bevy::prelude::*;
use bevy::text::{FontSource, FontWeight};
use leocard_ui::{TextOutline, TextureButton, UiComposer, sliced_image};

#[derive(Clone, Copy)]
pub(crate) enum GinkgoButtonTone {
    Moss,
    Blue,
    Plum,
    Cinnabar,
    Amber,
    Ash,
}

impl GinkgoButtonTone {
    fn colors(self) -> (Color, Color) {
        let (base, text) = match self {
            Self::Moss => ([87, 113, 72], [214, 230, 165]),
            Self::Blue => ([68, 101, 134], [191, 223, 245]),
            Self::Plum => ([106, 79, 113], [226, 199, 237]),
            Self::Cinnabar => ([157, 61, 48], [255, 219, 154]),
            Self::Amber => ([146, 103, 42], [255, 230, 166]),
            Self::Ash => ([84, 84, 79], [215, 207, 192]),
        };
        (
            Color::srgb_u8(base[0], base[1], base[2]),
            Color::srgb_u8(text[0], text[1], text[2]),
        )
    }
}

pub(crate) struct GinkgoButton<'a> {
    pub label: &'a str,
    pub action: UiAction,
    pub tone: GinkgoButtonTone,
    pub size: Vec2,
    pub font_size: f32,
}

impl GinkgoButton<'_> {
    #[track_caller]
    pub(crate) fn mount(
        self,
        commands: &mut UiComposer,
        parent: Entity,
        assets: &UiAssets,
    ) -> Entity {
        let (base, text) = self.tone.colors();
        let image = sliced_image(
            assets.controls.ginkgo_button.clone(),
            BorderRect {
                min_inset: Vec2::new(92.0, 44.0),
                max_inset: Vec2::new(24.0, 20.0),
            },
            self.size.y / 122.0,
        );
        let prominence = match self.tone {
            GinkgoButtonTone::Cinnabar => 1.12,
            _ => 1.0,
        };
        let font_size = self.font_size * prominence;
        let skin = TextureButton {
            node: Node {
                min_width: px(self.size.x),
                height: px(self.size.y),
                padding: UiRect::horizontal(px(self.size.y * 0.25)),
                flex_shrink: 0.0,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            normal: image.clone().with_color(base),
            hovered: image.with_color(base.mix(&Color::WHITE, 0.14)),
            action: Some(self.action),
            label: None,
            font_size,
            color: text,
            icon: None,
        };
        let (button, label) =
            mount_game_button(skin, commands, parent, &assets.controls.ginkgo_font);
        commands.entity(label).insert((
            Node {
                flex_shrink: 0.0,
                ..default()
            },
            UiTransform::from_translation(Val2::px(0.0, -0.10 * self.size.y)),
        ));
        let lettering = TextOutline {
            color: base.mix(&Color::BLACK, 0.72).with_alpha(0.95),
            width: font_size / 34.0,
        }
        .mount(
            commands,
            label,
            self.label,
            TextFont {
                font: FontSource::list([assets.controls.ginkgo_font.clone(), assets.font.clone()]),
                weight: FontWeight::BOLD,
                ..TextFont::from_font_size(font_size)
            },
            text,
        );
        commands.entity(lettering.fill).insert(TextShadow {
            offset: Vec2::new(0.8, 1.5),
            color: Color::BLACK.with_alpha(0.72),
        });
        button
    }
}
