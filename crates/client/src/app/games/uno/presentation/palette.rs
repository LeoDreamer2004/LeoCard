use super::{
    UnoPaletteColorRing, UnoPaletteEffect, UnoPaletteMaterial, UnoPaletteParticle,
    UnoPaletteSelectedSector, uno_ui_color,
};
use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use leocard_uno::UnoColor;

pub(super) fn spawn_uno_palette_effect(
    commands: &mut Commands,
    layer: Entity,
    center: Vec2,
    selected: UnoColor,
    materials: &mut Assets<UnoPaletteMaterial>,
) {
    let base_material = materials.add(UnoPaletteMaterial::new(selected, false));
    let palette = commands
        .spawn((
            UnoPaletteEffect { elapsed: 0.0 },
            Node {
                position_type: PositionType::Absolute,
                left: px(center.x - 120.0),
                top: px(center.y - 120.0),
                width: px(240),
                height: px(240),
                border_radius: BorderRadius::all(percent(50)),
                ..default()
            },
            MaterialNode(base_material),
            UiTransform::from_scale(Vec2::splat(0.78)),
            BoxShadow::new(Color::BLACK.with_alpha(0.0), px(3), px(6), px(0), px(9)),
            GlobalZIndex(1500),
            FocusPolicy::Pass,
        ))
        .id();
    commands.entity(layer).add_child(palette);
    let sector_material = materials.add(UnoPaletteMaterial::new(selected, true));
    let selected_sector = commands
        .spawn((
            UnoPaletteSelectedSector { elapsed: 0.0 },
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                top: px(0),
                width: px(240),
                height: px(240),
                ..default()
            },
            MaterialNode(sector_material),
            UiTransform::from_scale(Vec2::ONE),
            FocusPolicy::Pass,
        ))
        .id();
    commands.entity(palette).add_child(selected_sector);

    let selected_color = uno_ui_color(selected);
    for (diameter, thickness, delay, start_scale, end_scale, max_alpha) in [
        // 两圈从调色盘外缘之外发射，避免涟漪起步时压在四色扇形上。
        (224.0, 6.0, 0.0, 1.16, 2.30, 0.82),
        (208.0, 2.5, 0.07, 1.19, 2.25, 0.68),
    ] {
        let ring = commands
            .spawn((
                UnoPaletteColorRing {
                    elapsed: 0.0,
                    delay,
                    color: selected_color,
                    start_scale,
                    end_scale,
                    max_alpha,
                },
                Node {
                    position_type: PositionType::Absolute,
                    left: px((240.0 - diameter) * 0.5),
                    top: px((240.0 - diameter) * 0.5),
                    width: px(diameter),
                    height: px(diameter),
                    border: UiRect::all(px(thickness)),
                    border_radius: BorderRadius::all(percent(50)),
                    ..default()
                },
                BorderColor::all(selected_color.with_alpha(0.0)),
                UiTransform::from_scale(Vec2::splat(start_scale)),
                FocusPolicy::Pass,
                ZIndex(2),
            ))
            .id();
        commands.entity(palette).add_child(ring);
    }

    let sector_angle = match selected {
        UnoColor::Red => std::f32::consts::FRAC_PI_4,
        UnoColor::Yellow => std::f32::consts::FRAC_PI_4 * 3.0,
        UnoColor::Green => std::f32::consts::FRAC_PI_4 * 5.0,
        UnoColor::Blue => std::f32::consts::FRAC_PI_4 * 7.0,
        UnoColor::Pink => std::f32::consts::FRAC_PI_4,
        UnoColor::Teal => std::f32::consts::FRAC_PI_4 * 3.0,
        UnoColor::Orange => std::f32::consts::FRAC_PI_4 * 5.0,
        UnoColor::Purple => std::f32::consts::FRAC_PI_4 * 7.0,
    };
    for index in 0..10 {
        let spread = (index as f32 - 4.5) * 0.18;
        let angle = sector_angle + spread;
        let distance = 76.0 + (index % 4) as f32 * 12.0;
        let direction = Vec2::new(angle.cos(), angle.sin()) * distance;
        let size = 5.0 + (index % 3) as f32 * 1.7;
        let particle = commands
            .spawn((
                UnoPaletteParticle {
                    elapsed: 0.0,
                    delay: 0.72 + index as f32 * 0.018,
                    origin: Vec2::splat(120.0),
                    direction,
                    size: Vec2::splat(size),
                    color: selected_color,
                    rotation: index as f32 * 23.0,
                },
                Node {
                    position_type: PositionType::Absolute,
                    left: px(120.0 - size * 0.5),
                    top: px(120.0 - size * 0.5),
                    width: px(size),
                    height: px(size),
                    border_radius: BorderRadius::all(px(1.5)),
                    ..default()
                },
                BackgroundColor(selected_color.with_alpha(0.0)),
                UiTransform::from_scale(Vec2::splat(0.2)),
                FocusPolicy::Pass,
                ZIndex(3),
            ))
            .id();
        commands.entity(palette).add_child(particle);
    }
}
