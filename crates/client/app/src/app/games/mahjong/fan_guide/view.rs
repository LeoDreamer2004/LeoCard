use super::super::actions::MahjongUiAction;
use super::super::{MahjongAssets, MahjongTileMaterial};
use super::catalog::{ENTRIES, FanGuideEntry};
use super::example::render_example;
use super::state::MahjongFanGuideScroll;
use crate::app::presentation::{
    MUTED, PAGE_FRAME_TITLE, TEXT, add_text, add_title, page_frame_skin,
};
use crate::app::runtime::UiAssets;
use crate::app::shell::{
    CozyModalBackdrop, CozyModalKind, CozyModalPanel, UiAction, add_cozy_close_button,
    cozy_backdrop_color, cozy_panel_transform,
};
use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy::ui::{RelativeCursorPosition, VisualBox};
use bevy::ui_widgets::Button;
use leocard_ui::{ButtonHighlight, PageFrame, UiComposer, spawn_node};

pub(super) const TIERS: [u16; 12] = [88, 64, 48, 32, 24, 16, 12, 8, 6, 4, 2, 1];

pub(super) fn render_fan_guide(
    commands: &mut UiComposer,
    parent: Entity,
    selected_tier: u16,
    progress: f32,
    assets: &UiAssets,
    game_assets: &MahjongAssets,
    materials: &mut Assets<MahjongTileMaterial>,
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
        Pickable::default(),
        CozyModalBackdrop(CozyModalKind::MahjongFanGuide),
    ));
    let mut frame = PageFrame::new(page_frame_skin(assets));
    frame.node.width = px(1020);
    frame.node.max_width = percent(92);
    frame.node.height = percent(88);
    frame.node.max_height = px(950);
    frame.node.min_height = px(520);
    frame.heading.height = px(56);
    frame.content.padding = UiRect::all(px(20));
    let frame = frame.mount(commands, backdrop);
    commands.entity(frame.panel).insert((
        GlobalZIndex(2201),
        Pickable::default(),
        CozyModalPanel(CozyModalKind::MahjongFanGuide),
        cozy_panel_transform(progress),
    ));
    render_header(commands, frame.heading, assets);
    let body = spawn_node(
        commands,
        frame.content,
        Node {
            width: percent(100),
            height: px(0),
            min_height: px(0),
            flex_grow: 1.0,
            flex_direction: FlexDirection::Row,
            column_gap: px(16),
            ..default()
        },
        None,
    );
    render_tabs(commands, body, selected_tier, assets);
    spawn_node(
        commands,
        body,
        Node {
            width: px(1),
            height: percent(100),
            ..default()
        },
        Some(Color::srgba(0.68, 0.67, 0.73, 0.34)),
    );
    let main = spawn_node(
        commands,
        body,
        Node {
            width: px(0),
            min_width: px(0),
            height: percent(100),
            flex_grow: 1.0,
            flex_direction: FlexDirection::Column,
            row_gap: px(8),
            ..default()
        },
        None,
    );
    let heading = spawn_node(
        commands,
        main,
        Node {
            width: percent(100),
            min_height: px(34),
            align_items: AlignItems::Center,
            column_gap: px(12),
            ..default()
        },
        None,
    );
    add_title(
        commands,
        heading,
        format!("{selected_tier} 番"),
        22.0,
        fan_color(selected_tier),
        assets,
    );
    let count = ENTRIES
        .iter()
        .filter(|entry| entry.fan.points() == selected_tier)
        .count();
    add_text(
        commands,
        heading,
        format!("共 {count} 种"),
        13.0,
        MUTED,
        assets,
    );
    let content = spawn_node(
        commands,
        main,
        Node {
            width: percent(100),
            height: px(0),
            min_width: px(0),
            min_height: px(0),
            flex_grow: 1.0,
            padding: UiRect::right(px(12)),
            flex_direction: FlexDirection::Column,
            row_gap: px(2),
            overflow: Overflow::scroll_y(),
            ..default()
        },
        None,
    );
    commands.entity(content).insert((
        MahjongFanGuideScroll,
        RelativeCursorPosition::default(),
        ScrollPosition(Vec2::ZERO),
    ));
    for entry in ENTRIES
        .iter()
        .filter(|entry| entry.fan.points() == selected_tier)
    {
        render_fan_entry(commands, content, entry, assets, game_assets, materials);
    }
}

pub(super) fn render_header(commands: &mut UiComposer, parent: Entity, assets: &UiAssets) {
    add_title(
        commands,
        parent,
        "国标麻将番种",
        27.0,
        PAGE_FRAME_TITLE,
        assets,
    );
    add_cozy_close_button(
        commands,
        parent,
        UiAction::Mahjong(MahjongUiAction::CloseFanGuide),
        assets,
    );
}

pub(super) fn render_tabs(
    commands: &mut UiComposer,
    parent: Entity,
    selected_tier: u16,
    assets: &UiAssets,
) {
    let tabs = spawn_node(
        commands,
        parent,
        Node {
            width: px(108),
            min_width: px(108),
            height: percent(100),
            flex_direction: FlexDirection::Column,
            row_gap: px(3),
            ..default()
        },
        None,
    );
    for tier in TIERS {
        let active = selected_tier == tier;
        let mut image = ImageNode::new(if active {
            assets.home.purple_button_compact.clone()
        } else {
            assets.home.button.clone()
        })
        .with_mode(NodeImageMode::Sliced(TextureSlicer {
            border: BorderRect::all(32.0),
            center_scale_mode: SliceScaleMode::Stretch,
            sides_scale_mode: SliceScaleMode::Stretch,
            max_corner_scale: 0.42,
        }));
        image.visual_box = VisualBox::BorderBox;
        let tab = commands.child(
            tabs,
            (
                Button,
                UiAction::Mahjong(MahjongUiAction::SelectFanGuideTier(tier)),
                Node {
                    width: percent(100),
                    min_height: px(30),
                    max_height: px(42),
                    flex_grow: 1.0,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                image,
            ),
        );
        if !active {
            let mut image = ImageNode::new(assets.home.purple_button_compact.clone()).with_mode(
                NodeImageMode::Sliced(TextureSlicer {
                    border: BorderRect::all(32.0),
                    center_scale_mode: SliceScaleMode::Stretch,
                    sides_scale_mode: SliceScaleMode::Stretch,
                    max_corner_scale: 0.42,
                }),
            );
            image.visual_box = VisualBox::BorderBox;
            let overlay = commands.child(
                tab,
                (
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(0),
                        right: px(0),
                        top: px(0),
                        bottom: px(0),
                        ..default()
                    },
                    image,
                    Visibility::Hidden,
                    Pickable::IGNORE,
                ),
            );
            commands.entity(tab).insert(ButtonHighlight {
                overlay,
                arrows: None,
            });
        }
        let label = add_text(
            commands,
            tab,
            format!("{tier} 番"),
            14.0,
            fan_color(tier),
            assets,
        );
        commands.entity(label).insert(Pickable::IGNORE);
    }
}

pub(super) fn render_fan_entry(
    commands: &mut UiComposer,
    parent: Entity,
    entry: &FanGuideEntry,
    assets: &UiAssets,
    game_assets: &MahjongAssets,
    materials: &mut Assets<MahjongTileMaterial>,
) {
    let card = spawn_node(
        commands,
        parent,
        Node {
            width: percent(100),
            flex_shrink: 0.0,
            padding: UiRect {
                left: px(8),
                right: px(12),
                top: px(4),
                bottom: px(4),
            },
            flex_direction: FlexDirection::Column,
            row_gap: px(4),
            ..default()
        },
        None,
    );
    let color = fan_color(entry.fan.points());
    let heading = spawn_node(
        commands,
        card,
        Node {
            width: percent(100),
            min_height: px(26),
            align_items: AlignItems::Center,
            ..default()
        },
        None,
    );
    add_title(commands, heading, entry.fan.name(), 19.0, color, assets);
    add_text(commands, card, entry.requirement, 14.0, TEXT, assets);
    render_example(commands, card, entry.example, game_assets, materials);
    spawn_node(
        commands,
        card,
        Node {
            width: percent(100),
            height: px(1),
            margin: UiRect::top(px(4)),
            ..default()
        },
        Some(Color::srgba(0.68, 0.67, 0.73, 0.28)),
    );
}

pub(super) fn fan_color(points: u16) -> Color {
    if points >= 48 {
        Color::srgb(0.94, 0.73, 0.22)
    } else if points >= 6 {
        Color::srgb(0.37, 0.81, 0.48)
    } else {
        Color::srgb(0.91, 0.95, 0.92)
    }
}
