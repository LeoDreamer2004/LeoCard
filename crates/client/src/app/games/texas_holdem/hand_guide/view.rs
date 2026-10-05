use super::super::{TexasHoldemUiAction, texas_card_face, texas_category_label};
use super::catalog::{HandGuideEntry, ordered_entries};
use super::probabilities::HandGuideProbabilities;
use super::state::TexasHandGuideScroll;
use crate::app::presentation::{ACCENT, MUTED, TEXT, add_text, spawn_node};
use crate::app::runtime::UiAssets;
use crate::app::shell::{
    CozyModalBackdrop, CozyModalKind, CozyModalPanel, UiAction, add_cozy_close_button,
    add_cozy_panel, cozy_backdrop_color, cozy_panel_transform,
};
use bevy::prelude::*;
use bevy::ui::{FocusPolicy, RelativeCursorPosition};
use leocard_texas_holdem::TexasHoldemRuleSet;

pub(super) fn render_hand_guide(
    commands: &mut Commands,
    parent: Entity,
    rules: TexasHoldemRuleSet,
    progress: f32,
    assets: &UiAssets,
) {
    let backdrop = spawn_node(
        commands,
        parent,
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            right: px(0),
            top: px(0),
            bottom: px(0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        Some(cozy_backdrop_color(progress)),
    );
    commands.entity(backdrop).insert((
        GlobalZIndex(2200),
        FocusPolicy::Block,
        CozyModalBackdrop(CozyModalKind::TexasHandGuide),
    ));
    let panel = add_cozy_panel(
        commands,
        backdrop,
        Node {
            width: px(980),
            max_width: percent(92),
            height: percent(88),
            max_height: px(880),
            min_height: px(480),
            padding: UiRect::all(px(24)),
            flex_direction: FlexDirection::Column,
            row_gap: px(12),
            ..default()
        },
        assets,
    );
    commands.entity(panel).insert((
        GlobalZIndex(2201),
        FocusPolicy::Block,
        CozyModalPanel(CozyModalKind::TexasHandGuide),
        cozy_panel_transform(progress),
    ));
    render_header(commands, panel, rules, assets);
    let content = spawn_node(
        commands,
        panel,
        Node {
            width: percent(100),
            height: px(0),
            min_height: px(0),
            flex_grow: 1.0,
            padding: UiRect::right(px(12)),
            flex_direction: FlexDirection::Column,
            overflow: Overflow::scroll_y(),
            ..default()
        },
        None,
    );
    commands.entity(content).insert((
        TexasHandGuideScroll,
        RelativeCursorPosition::default(),
        ScrollPosition(Vec2::ZERO),
    ));
    let probabilities = HandGuideProbabilities::for_rules(&rules);
    for (index, entry) in ordered_entries(&rules).into_iter().enumerate() {
        render_entry(
            commands,
            content,
            index + 1,
            entry,
            probabilities.percentage_label(entry.category),
            assets,
        );
    }
}

fn render_header(
    commands: &mut Commands,
    parent: Entity,
    rules: TexasHoldemRuleSet,
    assets: &UiAssets,
) {
    let header = spawn_node(
        commands,
        parent,
        Node {
            width: percent(100),
            min_height: px(40),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            ..default()
        },
        None,
    );
    let variant = if rules.omaha {
        "奥马哈"
    } else {
        "德州扑克"
    };
    let mode = if rules.short_deck { "短牌" } else { "标准" };
    add_text(
        commands,
        header,
        format!("{mode}{variant}牌型介绍"),
        27.0,
        TEXT,
        assets,
    );
    add_cozy_close_button(
        commands,
        header,
        UiAction::TexasHoldem(TexasHoldemUiAction::CloseHandGuide),
        assets,
    );
    spawn_node(
        commands,
        parent,
        Node {
            width: px(164),
            height: px(2),
            ..default()
        },
        Some(ACCENT),
    );
}

fn render_entry(
    commands: &mut Commands,
    parent: Entity,
    rank: usize,
    entry: &HandGuideEntry,
    probability: String,
    assets: &UiAssets,
) {
    let row = spawn_node(
        commands,
        parent,
        Node {
            width: percent(100),
            min_height: px(114),
            flex_shrink: 0.0,
            padding: UiRect::axes(px(4), px(14)),
            align_items: AlignItems::Center,
            column_gap: px(20),
            border: UiRect::bottom(px(1)),
            ..default()
        },
        None,
    );
    commands
        .entity(row)
        .insert(BorderColor::all(ACCENT.with_alpha(0.28)));
    let position = spawn_node(
        commands,
        row,
        Node {
            width: px(34),
            flex_shrink: 0.0,
            ..default()
        },
        None,
    );
    add_text(
        commands,
        position,
        format!("{rank:02}"),
        20.0,
        ACCENT,
        assets,
    );
    let description = spawn_node(
        commands,
        row,
        Node {
            flex_grow: 1.0,
            flex_basis: px(0),
            min_width: px(0),
            flex_direction: FlexDirection::Column,
            row_gap: px(7),
            ..default()
        },
        None,
    );
    let title = add_text(
        commands,
        description,
        texas_category_label(entry.category),
        21.0,
        TEXT,
        assets,
    );
    commands.entity(title).with_child((
        TextSpan::new(format!(" ({probability})")),
        TextFont::from_font_size(21.0).with_font(assets.font.clone()),
        TextColor(Color::srgb(0.70, 0.70, 0.70)),
    ));
    add_text(
        commands,
        description,
        entry.description,
        14.0,
        MUTED,
        assets,
    );
    let cards = spawn_node(
        commands,
        row,
        Node {
            column_gap: px(6),
            flex_shrink: 0.0,
            align_items: AlignItems::Center,
            ..default()
        },
        None,
    );
    for card in entry.example {
        let image = commands
            .spawn((
                Node {
                    width: px(50),
                    height: px(68),
                    flex_shrink: 0.0,
                    border: UiRect::all(px(1)),
                    border_radius: BorderRadius::all(px(4)),
                    ..default()
                },
                ImageNode::new(texas_card_face(card, assets)),
                BorderColor::all(TEXT.with_alpha(0.42)),
                FocusPolicy::Pass,
            ))
            .id();
        commands.entity(cards).add_child(image);
    }
}
