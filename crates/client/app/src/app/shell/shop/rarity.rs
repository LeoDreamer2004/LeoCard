use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy::ui::{
    AngularColorStop, BackgroundGradient, ColorStop, ConicGradient, RadialGradient,
    RadialGradientShape,
};
use leocard_client::ShopRarity;
use leocard_ui::UiComposer;
use std::f32::consts::TAU;

pub(super) fn add_rarity_backdrop(commands: &mut UiComposer, preview: Entity, rarity: ShopRarity) {
    let (light, shade, edge) = match rarity {
        ShopRarity::Common => (
            Color::srgba(0.48, 0.68, 0.84, 0.23),
            Color::srgba(0.025, 0.065, 0.12, 0.44),
            Color::srgba(0.57, 0.72, 0.86, 0.24),
        ),
        ShopRarity::Fine => (
            Color::srgba(0.76, 0.49, 1.0, 0.42),
            Color::srgba(0.14, 0.035, 0.29, 0.55),
            Color::srgba(0.76, 0.54, 1.0, 0.67),
        ),
        ShopRarity::Epic => (
            Color::srgba(1.0, 0.78, 0.32, 0.56),
            Color::srgba(0.28, 0.105, 0.018, 0.67),
            Color::srgba(1.0, 0.81, 0.40, 0.89),
        ),
    };
    let backdrop = commands.child(
        preview,
        (
            Node {
                position_type: PositionType::Absolute,
                left: px(5),
                right: px(5),
                top: px(5),
                bottom: px(5),
                border: UiRect::all(px(1)),
                border_radius: BorderRadius::all(px(5)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BorderColor::all(edge),
            BackgroundGradient::from(RadialGradient::new(
                UiPosition::CENTER,
                RadialGradientShape::FarthestCorner,
                vec![
                    ColorStop::percent(light, 0.0),
                    ColorStop::percent(light.with_alpha(0.15), 49.0),
                    ColorStop::percent(shade, 100.0),
                ],
            )),
            Pickable::IGNORE,
        ),
    );
    match rarity {
        ShopRarity::Common => {}
        ShopRarity::Fine => {
            commands.child(
                backdrop,
                (
                    Node {
                        width: px(88),
                        height: px(88),
                        border: UiRect::all(px(1.5)),
                        border_radius: BorderRadius::all(px(6)),
                        ..default()
                    },
                    BorderColor::all(edge.with_alpha(0.53)),
                    UiTransform::from_rotation(Rot2::degrees(45.0)),
                    Pickable::IGNORE,
                ),
            );
        }
        ShopRarity::Epic => add_epic_rays(commands, backdrop, edge),
    }
}

fn add_epic_rays(commands: &mut UiComposer, backdrop: Entity, gold: Color) {
    const RAYS: u32 = 10;
    let step = TAU / RAYS as f32;
    let transparent = gold.with_alpha(0.0);
    let stops = (0..RAYS)
        .flat_map(|index| {
            let angle = index as f32 * step;
            [
                AngularColorStop::new(transparent, angle),
                AngularColorStop::new(gold.with_alpha(0.26), angle + step * 0.32),
                AngularColorStop::new(transparent, angle + step * 0.67),
            ]
        })
        .chain([AngularColorStop::new(transparent, TAU)])
        .collect();
    commands.child(
        backdrop,
        (
            Node {
                position_type: PositionType::Absolute,
                left: px(2),
                right: px(2),
                top: px(2),
                bottom: px(2),
                border: UiRect::all(px(1)),
                border_radius: BorderRadius::all(px(4)),
                ..default()
            },
            BorderColor::all(gold.with_alpha(0.46)),
            BackgroundGradient::from(ConicGradient::new(UiPosition::CENTER, stops)),
            Pickable::IGNORE,
        ),
    );
}
