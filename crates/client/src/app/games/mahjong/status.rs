use super::{MahjongAssets, MahjongTurnArrow, wind_label};
use crate::app::presentation::{MUTED, PanelSkin, TEXT, add_panel, add_text};
use crate::app::runtime::UiAssets;
use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use leocard_protocol::{MahjongPhaseView, MahjongSnapshot};

pub(super) fn render_round_status(
    commands: &mut Commands,
    table: Entity,
    game: &MahjongSnapshot,
    own_seat: u8,
    assets: &UiAssets,
    game_assets: &MahjongAssets,
) {
    let status = add_panel(
        commands,
        table,
        Node {
            position_type: PositionType::Absolute,
            left: px(565),
            top: px(288),
            width: px(150),
            height: px(88),
            border: UiRect::all(px(1)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            row_gap: px(4),
            ..default()
        },
        Color::srgba(0.018, 0.075, 0.052, 0.94),
        PanelSkin::Section,
        assets,
    );
    commands.entity(status).insert((
        BorderColor::all(Color::srgba(0.72, 0.58, 0.25, 0.72)),
        BoxShadow::new(Color::BLACK.with_alpha(0.46), px(0), px(4), px(0), px(9)),
        ZIndex(20),
    ));
    add_text(
        commands,
        status,
        format!(
            "{}风  第 {} 局",
            wind_label(game.prevalent_wind),
            game.sequence_index + 1
        ),
        15.0,
        Color::srgb(0.92, 0.79, 0.43),
        assets,
    );
    add_text(
        commands,
        status,
        game.wall_len.to_string(),
        25.0,
        TEXT,
        assets,
    );
    add_text(commands, status, "牌墙余张", 10.0, MUTED, assets);
    if matches!(game.phase, MahjongPhaseView::Playing) {
        let current_seat = game
            .players
            .iter()
            .find(|player| player.id == game.current_player)
            .map(|player| player.seat.0)
            .unwrap_or(own_seat);
        render_turn_arrows(
            commands,
            table,
            (current_seat + 4 - own_seat) % 4,
            game_assets,
        );
    }
}

fn render_turn_arrows(
    commands: &mut Commands,
    table: Entity,
    relative: u8,
    assets: &MahjongAssets,
) {
    let (origin, direction, rotation) = match relative {
        0 => (
            Vec2::new(630.0, 382.0),
            Vec2::new(0.0, 15.0),
            std::f32::consts::FRAC_PI_2,
        ),
        1 => (Vec2::new(720.0, 322.0), Vec2::new(15.0, 0.0), 0.0),
        2 => (
            Vec2::new(630.0, 266.0),
            Vec2::new(0.0, -15.0),
            -std::f32::consts::FRAC_PI_2,
        ),
        _ => (
            Vec2::new(540.0, 322.0),
            Vec2::new(-15.0, 0.0),
            std::f32::consts::PI,
        ),
    };
    for slot in 0..3 {
        let position = origin + direction * slot as f32;
        let arrow = commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: px(position.x),
                    top: px(position.y),
                    width: px(20),
                    height: px(20),
                    ..default()
                },
                ImageNode::new(assets.turn_arrow.clone()).with_color(Color::WHITE.with_alpha(0.0)),
                UiTransform::from_rotation(Rot2::radians(rotation)),
                MahjongTurnArrow { slot: slot as f32 },
                ZIndex(24),
                FocusPolicy::Pass,
            ))
            .id();
        commands.entity(table).add_child(arrow);
    }
}
