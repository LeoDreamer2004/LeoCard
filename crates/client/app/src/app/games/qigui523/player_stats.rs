use crate::app::presentation::{ACCENT, add_hand_count, add_text};
use crate::app::runtime::UiAssets;
use crate::app::shell::{PlayerGameScoreText, SeatSide};
use bevy::picking::Pickable;
use bevy::prelude::*;
use leocard_protocol::PlayerId;
use leocard_ui::{UiComposer, spawn_node};

pub(super) fn add_qigui_player_stats(
    commands: &mut UiComposer,
    portrait: Entity,
    player: PlayerId,
    side: SeatSide,
    score: u32,
    hand_len: u16,
    assets: &UiAssets,
) {
    let row = spawn_node(
        commands,
        portrait,
        Node {
            position_type: PositionType::Absolute,
            top: px(76.0 * 1.17 + 2.0),
            width: percent(100),
            height: px(27),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            column_gap: px(4),
            ..default()
        },
        None,
    );
    commands.entity(row).insert(Pickable::IGNORE);
    add_hand_count(commands, row, hand_len, assets);
    let icon = spawn_node(
        commands,
        row,
        Node {
            width: px(18),
            height: px(18),
            flex_shrink: 0.0,
            border: UiRect::all(px(1.5)),
            border_radius: BorderRadius::all(percent(50)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        None,
    );
    commands
        .entity(icon)
        .insert((BorderColor::all(ACCENT.with_alpha(0.85)), Pickable::IGNORE));
    let glyph = add_text(commands, icon, "分", 11.0, ACCENT, assets);
    commands.entity(glyph).insert(Pickable::IGNORE);
    let digits = score.to_string();
    let font_size = (20.0 - digits.len().saturating_sub(3) as f32 * 1.5).max(14.0);
    let value = add_text(commands, row, digits, font_size, ACCENT, assets);
    commands.entity(value).insert((
        PlayerGameScoreText::Opponent { player, side },
        TextLayout::default().with_no_wrap(),
        TextShadow {
            offset: Vec2::new(1.5, 2.0),
            color: Color::BLACK.with_alpha(0.82),
        },
        Pickable::IGNORE,
    ));
}
