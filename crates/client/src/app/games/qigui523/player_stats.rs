use crate::app::presentation::{ACCENT, TEXT, add_text, spawn_node};
use crate::app::runtime::UiAssets;
use crate::app::shell::{PlayerGameScoreText, SeatSide};
use bevy::{prelude::*, ui::FocusPolicy};
use leocard_protocol::PlayerId;

pub(super) fn add_qigui_player_stats(
    commands: &mut Commands,
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
    commands.entity(row).insert(FocusPolicy::Pass);
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
        .insert((BorderColor::all(ACCENT.with_alpha(0.85)), FocusPolicy::Pass));
    let glyph = add_text(commands, icon, "分", 11.0, ACCENT, assets);
    commands.entity(glyph).insert(FocusPolicy::Pass);
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
        FocusPolicy::Pass,
    ));
}

fn add_hand_count(commands: &mut Commands, parent: Entity, count: u16, assets: &UiAssets) {
    let fan = spawn_node(
        commands,
        parent,
        Node {
            width: px(28),
            height: px(24),
            flex_shrink: 0.0,
            position_type: PositionType::Relative,
            ..default()
        },
        None,
    );
    commands.entity(fan).insert(FocusPolicy::Pass);
    for (left, top, rotation, layer) in [
        (2.0, 4.0, -0.27, 0),
        (8.0, 1.0, 0.0, 1),
        (14.0, 4.0, 0.27, 2),
    ] {
        let card = commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: px(left),
                    top: px(top),
                    width: px(12),
                    height: px(18),
                    ..default()
                },
                ImageNode::new(assets.playing_cards.card_back.clone()),
                UiTransform::from_rotation(Rot2::radians(rotation)),
                ZIndex(layer),
                FocusPolicy::Pass,
            ))
            .id();
        commands.entity(fan).add_child(card);
    }
    let number = add_text(commands, parent, count.to_string(), 14.0, TEXT, assets);
    commands.entity(number).insert((
        TextLayout::default().with_no_wrap(),
        TextShadow {
            offset: Vec2::new(1.0, 1.0),
            color: Color::BLACK.with_alpha(0.8),
        },
        FocusPolicy::Pass,
    ));
}
