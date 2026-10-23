use super::state::{GraphLogMessage, LogMessage};
use crate::app::presentation::{ACCENT, UiComposer, add_avatar, add_text, spawn_node};
use crate::app::runtime::UiAssets;
use bevy::picking::Pickable;
use bevy::prelude::*;

pub(super) fn render_message(
    commands: &mut UiComposer,
    parent: Entity,
    message: &LogMessage,
    assets: &UiAssets,
) {
    let row = spawn_node(
        commands,
        parent,
        Node {
            width: percent(100),
            min_width: px(0),
            flex_shrink: 0.0,
            padding: UiRect::vertical(px(3)),
            column_gap: px(3),
            align_items: AlignItems::Start,
            ..default()
        },
        None,
    );
    commands
        .entity(row)
        .insert((Pickable::IGNORE, GraphLogMessage));
    let opening = add_text(commands, row, "<", 14.0, Color::WHITE, assets);
    commands.entity(opening).insert(Pickable::IGNORE);
    let avatar = add_avatar(
        commands,
        row,
        &message.name,
        message.avatar.as_ref(),
        18.0,
        assets,
    );
    commands.entity(avatar).insert((
        Pickable::IGNORE,
        Node {
            width: px(18),
            height: px(18),
            min_width: px(18),
            flex_shrink: 0.0,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
    ));
    let text = add_text(
        commands,
        row,
        format!("{}>", message.name),
        14.0,
        Color::WHITE,
        assets,
    );
    commands.entity(text).insert((
        Pickable::IGNORE,
        Node {
            flex_grow: 1.0,
            flex_basis: px(0),
            min_width: px(0),
            ..default()
        },
        TextShadow {
            offset: Vec2::splat(0.5),
            color: Color::BLACK.with_alpha(0.8),
        },
    ));
    let muted = Color::srgb(0.78, 0.78, 0.78);
    let mut spans = vec![
        (" 猜测 ".to_owned(), muted),
        (message.node.to_string(), ACCENT),
        (" 号节点是".to_owned(), muted),
        (format!("「{}」", message.guess), ACCENT),
    ];
    if message.correct {
        spans.extend([
            ("，猜测正确！".to_owned(), Color::srgb(0.5, 0.95, 0.58)),
            (format!("连击 x{}！", message.combo), ACCENT),
        ]);
    } else {
        spans.extend([
            ("，结果".to_owned(), muted),
            ("错误".to_owned(), Color::srgb(1.0, 0.48, 0.45)),
            ("。".to_owned(), muted),
        ]);
    }
    for (value, color) in spans {
        commands.child(
            text,
            (
                TextSpan::new(value),
                TextFont::from_font_size(14.0).with_font(assets.font.clone()),
                TextColor(color),
            ),
        );
    }
}
