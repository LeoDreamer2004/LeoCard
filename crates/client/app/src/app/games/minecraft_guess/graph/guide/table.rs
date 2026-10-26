use super::super::super::MinecraftGuessAssets;
use super::super::art::relation_color;
use super::state::GraphGuideScroll;
use crate::app::presentation::{TEXT, add_text};
use crate::app::runtime::UiAssets;
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;
use leocard_minecraft_guess::GraphRelation;
use leocard_ui::{Scrollbar, UiComposer, spawn_node};

struct RelationRow<'a> {
    group: &'a str,
    details: &'a str,
    color: Color,
    heading: bool,
}

pub(super) fn render_table(
    commands: &mut UiComposer,
    parent: Entity,
    assets: &UiAssets,
    game_assets: &MinecraftGuessAssets,
) {
    let body = spawn_node(
        commands,
        parent,
        Node {
            width: percent(100),
            height: px(0),
            min_height: px(0),
            flex_grow: 1.0,
            column_gap: px(8),
            ..default()
        },
        None,
    );
    let table = spawn_node(
        commands,
        body,
        Node {
            flex_grow: 1.0,
            flex_basis: px(0),
            min_width: px(0),
            height: percent(100),
            padding: UiRect {
                left: px(2),
                right: px(2),
                top: px(6),
                bottom: px(2),
            },
            flex_direction: FlexDirection::Column,
            ..default()
        },
        None,
    );
    commands.entity(table).insert(game_assets.inset());
    render_row(
        commands,
        table,
        RelationRow {
            group: "未激活时",
            details: "揭晓后的具体关系",
            color: TEXT,
            heading: true,
        },
        assets,
        game_assets,
    );
    let scroll = spawn_node(
        commands,
        table,
        Node {
            width: percent(100),
            height: px(0),
            min_height: px(0),
            flex_grow: 1.0,
            flex_direction: FlexDirection::Column,
            overflow: Overflow::scroll_y(),
            ..default()
        },
        None,
    );
    commands.entity(scroll).insert((
        GraphGuideScroll,
        RelativeCursorPosition::default(),
        ScrollPosition(Vec2::ZERO),
    ));
    for (group, relations) in relation_groups() {
        let details = relations
            .iter()
            .map(|relation| relation.label())
            .collect::<Vec<_>>()
            .join("、");
        render_row(
            commands,
            scroll,
            RelationRow {
                group: group.label(),
                details: &details,
                color: relation_color(group),
                heading: false,
            },
            assets,
            game_assets,
        );
    }
    Scrollbar::spawn(commands, body, scroll, game_assets.scrollbar());
}

fn relation_groups() -> Vec<(GraphRelation, Vec<GraphRelation>)> {
    let mut groups: Vec<(GraphRelation, Vec<GraphRelation>)> = Vec::new();
    for relation in GraphRelation::DETAILED {
        let group = relation.coarse();
        if let Some((_, details)) = groups.iter_mut().find(|(kind, _)| *kind == group) {
            details.push(relation);
        } else {
            groups.push((group, vec![relation]));
        }
    }
    groups
}

fn render_row(
    commands: &mut UiComposer,
    parent: Entity,
    content: RelationRow<'_>,
    assets: &UiAssets,
    game_assets: &MinecraftGuessAssets,
) {
    let row = spawn_node(
        commands,
        parent,
        Node {
            width: percent(100),
            min_height: px(if content.heading { 46.0 } else { 72.0 }),
            flex_shrink: 0.0,
            align_items: AlignItems::Stretch,
            ..default()
        },
        None,
    );
    for (index, text) in [content.group, content.details].into_iter().enumerate() {
        let cell = spawn_node(
            commands,
            row,
            Node {
                width: percent(if index == 0 { 24.0 } else { 76.0 }),
                min_width: px(0),
                flex_shrink: 0.0,
                padding: UiRect::axes(px(14), px(12)),
                border: UiRect {
                    right: px(if index == 0 { 2.0 } else { 0.0 }),
                    bottom: px(2),
                    ..default()
                },
                align_items: AlignItems::Center,
                justify_content: if index == 0 {
                    JustifyContent::Center
                } else {
                    JustifyContent::FlexStart
                },
                ..default()
            },
            None,
        );
        commands.entity(cell).insert((
            if content.heading {
                game_assets.cell(Color::WHITE)
            } else {
                game_assets.surface()
            },
            BorderColor::all(Color::srgb_u8(83, 85, 101)),
        ));
        add_text(
            commands,
            cell,
            text,
            18.0,
            if index == 0 { content.color } else { TEXT },
            assets,
        );
    }
}
