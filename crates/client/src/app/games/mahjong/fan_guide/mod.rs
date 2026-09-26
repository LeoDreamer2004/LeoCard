mod catalog;

use self::catalog::{ENTRIES, FanGuideEntry};
use super::actions::MahjongUiAction;
use super::{
    MAHJONG_KONG_STACK_LIFT, MahjongAssets, MahjongTileMaterial, MahjongTileSize,
    MahjongTileVisual, MahjongUiState, add_mahjong_tile_material,
};
use crate::app::presentation::{
    ACCENT, BORDER, BackgroundButtonTint, ButtonTint, HEADER_BG, PANEL, TEXT, add_text, spawn_node,
};
use crate::app::runtime::UiAssets;
use crate::app::shell::UiAction;
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use bevy::ui::{FocusPolicy, RelativeCursorPosition};
use leocard_mahjong::{MahjongDragon, MahjongFlower, MahjongSuit, MahjongTileKind, MahjongWind};

const TIERS: [u16; 12] = [88, 64, 48, 32, 24, 16, 12, 8, 6, 4, 2, 1];

#[derive(Component)]
pub(super) struct MahjongFanGuideScroll;

#[derive(Component)]
pub(super) struct MahjongFanGuideRoot(u16);

pub(super) fn sync_mahjong_fan_guide(
    mut commands: Commands,
    ui: Res<MahjongUiState>,
    roots: Query<(Entity, &MahjongFanGuideRoot)>,
    assets: Res<UiAssets>,
    game_assets: Res<MahjongAssets>,
    mut materials: ResMut<Assets<MahjongTileMaterial>>,
) {
    let tier = ui.fan_guide_open.then_some(ui.fan_guide_tier);
    let mut current = None;
    for (entity, root) in &roots {
        if Some(root.0) == tier {
            current = Some(entity);
        } else {
            commands.entity(entity).despawn();
        }
    }
    let Some(tier) = tier else {
        return;
    };
    if current.is_some() {
        return;
    }
    let root = commands
        .spawn((
            MahjongFanGuideRoot(tier),
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                right: px(0),
                top: px(0),
                bottom: px(0),
                ..default()
            },
            GlobalZIndex(2200),
            FocusPolicy::Pass,
        ))
        .id();
    render_fan_guide(
        &mut commands,
        root,
        tier,
        &assets,
        &game_assets,
        &mut materials,
    );
}

pub(super) fn add_fan_guide_button(commands: &mut Commands, chat_panel: Entity, assets: &UiAssets) {
    let normal = Color::srgb(0.13, 0.39, 0.29);
    let button = commands
        .spawn((
            Button,
            UiAction::Mahjong(MahjongUiAction::ToggleFanGuide),
            ButtonTint {
                normal,
                hovered: Color::srgb(0.20, 0.52, 0.38),
                pressed: Color::srgb(0.10, 0.28, 0.21),
            },
            Node {
                position_type: PositionType::Absolute,
                left: px(-32),
                top: px(234),
                width: px(32),
                height: px(32),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border: UiRect::all(px(1)),
                border_radius: BorderRadius::all(px(7)),
                ..default()
            },
            BorderColor::all(BORDER),
            GlobalZIndex(2000),
            ImageNode::new(assets.controls.secondary_button.clone())
                .with_mode(NodeImageMode::Stretch)
                .with_color(normal),
        ))
        .id();
    commands.entity(chat_panel).add_child(button);
    let label = add_text(commands, button, "?", 23.0, TEXT, assets);
    commands.entity(label).insert(FocusPolicy::Pass);
}

fn render_fan_guide(
    commands: &mut Commands,
    parent: Entity,
    selected_tier: u16,
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
            ..default()
        },
        Some(Color::BLACK.with_alpha(0.45)),
    );
    commands
        .entity(backdrop)
        .insert((GlobalZIndex(2200), FocusPolicy::Block));
    let window = spawn_node(
        commands,
        parent,
        Node {
            position_type: PositionType::Absolute,
            left: percent(13),
            top: percent(7),
            width: percent(74),
            height: percent(86),
            padding: UiRect::all(px(16)),
            flex_direction: FlexDirection::Column,
            row_gap: px(12),
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(12)),
            ..default()
        },
        Some(HEADER_BG.with_alpha(0.86)),
    );
    commands.entity(window).insert((
        GlobalZIndex(2201),
        BorderColor::all(ACCENT.with_alpha(0.68)),
        FocusPolicy::Block,
        BoxShadow::new(Color::BLACK.with_alpha(0.52), px(0), px(12), px(0), px(22)),
    ));
    render_header(commands, window, assets);
    let body = spawn_node(
        commands,
        window,
        Node {
            width: percent(100),
            height: px(0),
            min_height: px(0),
            flex_grow: 1.0,
            flex_direction: FlexDirection::Row,
            column_gap: px(12),
            ..default()
        },
        None,
    );
    render_tabs(commands, body, selected_tier, assets);
    let content = spawn_node(
        commands,
        body,
        Node {
            width: px(0),
            height: percent(100),
            min_width: px(0),
            min_height: px(0),
            flex_grow: 1.0,
            padding: UiRect::right(px(8)),
            flex_direction: FlexDirection::Column,
            row_gap: px(10),
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

fn render_header(commands: &mut Commands, parent: Entity, assets: &UiAssets) {
    let header = spawn_node(
        commands,
        parent,
        Node {
            width: percent(100),
            height: px(34),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            ..default()
        },
        None,
    );
    add_text(commands, header, "国标麻将番种介绍", 21.0, ACCENT, assets);
    let close = commands
        .spawn((
            Button,
            UiAction::Mahjong(MahjongUiAction::CloseFanGuide),
            BackgroundButtonTint,
            ButtonTint {
                normal: Color::srgb(0.14, 0.29, 0.24),
                hovered: Color::srgb(0.24, 0.44, 0.35),
                pressed: Color::srgb(0.10, 0.21, 0.18),
            },
            Node {
                width: px(34),
                height: px(30),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border_radius: BorderRadius::all(px(6)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.14, 0.29, 0.24)),
        ))
        .id();
    commands.entity(header).add_child(close);
    let label = add_text(commands, close, "×", 21.0, TEXT, assets);
    commands.entity(label).insert(FocusPolicy::Pass);
}

fn render_tabs(commands: &mut Commands, parent: Entity, selected_tier: u16, assets: &UiAssets) {
    let tabs = spawn_node(
        commands,
        parent,
        Node {
            width: px(94),
            height: percent(100),
            flex_direction: FlexDirection::Column,
            row_gap: px(3),
            ..default()
        },
        None,
    );
    for tier in TIERS {
        let active = selected_tier == tier;
        let color = fan_color(tier);
        let normal = if active {
            color.with_alpha(0.25)
        } else {
            PANEL
        };
        let tab = commands
            .spawn((
                Button,
                UiAction::Mahjong(MahjongUiAction::SelectFanGuideTier(tier)),
                BackgroundButtonTint,
                ButtonTint {
                    normal,
                    hovered: color.with_alpha(0.35),
                    pressed: color.with_alpha(0.46),
                },
                Node {
                    width: percent(100),
                    min_height: px(32),
                    flex_grow: 1.0,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    border: UiRect::left(px(if active { 3 } else { 0 })),
                    border_radius: BorderRadius::all(px(5)),
                    ..default()
                },
                BackgroundColor(normal),
                BorderColor::all(color),
            ))
            .id();
        commands.entity(tabs).add_child(tab);
        let label = add_text(commands, tab, format!("{tier} 番"), 15.0, color, assets);
        commands.entity(label).insert(FocusPolicy::Pass);
    }
}

fn render_fan_entry(
    commands: &mut Commands,
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
            padding: UiRect::all(px(8)),
            flex_direction: FlexDirection::Column,
            row_gap: px(7),
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(6)),
            ..default()
        },
        Some(PANEL.with_alpha(0.78)),
    );
    commands.entity(card).insert(BorderColor::all(BORDER));
    let color = fan_color(entry.fan.points());
    let bar = spawn_node(
        commands,
        card,
        Node {
            width: percent(100),
            min_height: px(32),
            padding: UiRect::horizontal(px(10)),
            align_items: AlignItems::Center,
            ..default()
        },
        Some(color.with_alpha(0.28)),
    );
    add_text(commands, bar, entry.fan.name(), 17.0, color, assets);
    add_text(commands, card, entry.requirement, 14.0, TEXT, assets);
    render_example(commands, card, entry.example, game_assets, materials);
}

fn render_example(
    commands: &mut Commands,
    parent: Entity,
    notation: &str,
    game_assets: &MahjongAssets,
    materials: &mut Assets<MahjongTileMaterial>,
) {
    let row = spawn_node(
        commands,
        parent,
        Node {
            width: percent(100),
            min_height: px(54),
            align_items: AlignItems::FlexEnd,
            flex_wrap: FlexWrap::Wrap,
            column_gap: px(8),
            row_gap: px(8),
            ..default()
        },
        None,
    );
    for token in notation.split_ascii_whitespace() {
        let (exposed, kinds) = parse_group(token).expect("番种牌型示例必须有效");
        let kong = kinds.len() == 4;
        let group = spawn_node(
            commands,
            row,
            Node {
                width: px(if kong {
                    95.0
                } else {
                    kinds.len() as f32 * 30.0 + 5.0
                }),
                height: px(if kong { 65.0 } else { 54.0 }),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::FlexEnd,
                ..default()
            },
            None,
        );
        let tiles = spawn_node(
            commands,
            group,
            Node {
                height: px(45),
                flex_direction: FlexDirection::Row,
                ..default()
            },
            None,
        );
        for (index, kind) in kinds
            .iter()
            .copied()
            .take(if kong { 3 } else { kinds.len() })
            .enumerate()
        {
            add_mahjong_tile_material(
                commands,
                tiles,
                MahjongTileVisual {
                    kind: guide_base_kind(kind, kong, exposed),
                    size: if kong {
                        guide_kong_size(exposed)
                    } else {
                        guide_tile_size(exposed)
                    },
                    index,
                    highlighted: false,
                    deal: None,
                    relative: 0,
                },
                game_assets,
                materials,
            );
        }
        if kong {
            let top = spawn_node(
                commands,
                group,
                Node {
                    position_type: PositionType::Absolute,
                    left: px(30),
                    top: px(20.0 - MAHJONG_KONG_STACK_LIFT),
                    width: px(33),
                    height: px(45),
                    ..default()
                },
                None,
            );
            commands.entity(top).insert(ZIndex(100));
            let stacked_tile = add_mahjong_tile_material(
                commands,
                top,
                MahjongTileVisual {
                    kind: Some(kinds[3]),
                    size: guide_kong_size(exposed),
                    index: 3,
                    highlighted: false,
                    deal: None,
                    relative: 0,
                },
                game_assets,
                materials,
            );
            commands.entity(stacked_tile).insert(BoxShadow::new(
                Color::BLACK.with_alpha(0.24),
                px(2),
                px(5),
                px(0),
                px(4),
            ));
        }
    }
}

fn guide_tile_size(exposed: bool) -> MahjongTileSize {
    if exposed {
        MahjongTileSize::GuideMeld
    } else {
        MahjongTileSize::GuideHand
    }
}

fn guide_kong_size(exposed: bool) -> MahjongTileSize {
    if exposed {
        MahjongTileSize::GuideMeld
    } else {
        MahjongTileSize::GuideConcealedMeld
    }
}

fn guide_base_kind(kind: MahjongTileKind, kong: bool, exposed: bool) -> Option<MahjongTileKind> {
    if kong && !exposed { None } else { Some(kind) }
}

fn fan_color(points: u16) -> Color {
    if points >= 48 {
        Color::srgb(0.94, 0.73, 0.22)
    } else if points >= 6 {
        Color::srgb(0.37, 0.81, 0.48)
    } else {
        Color::srgb(0.91, 0.95, 0.92)
    }
}

fn parse_group(group: &str) -> Option<(bool, Vec<MahjongTileKind>)> {
    let (exposed, group) = if let Some(group) = group.strip_prefix('!') {
        (true, group)
    } else {
        (false, group)
    };
    let suit = match group.as_bytes().last().copied() {
        Some(b'm') => Some(MahjongSuit::Characters),
        Some(b's') => Some(MahjongSuit::Bamboo),
        Some(b'p') => Some(MahjongSuit::Dots),
        _ => None,
    };
    let mut tiles = Vec::new();
    for byte in group
        .bytes()
        .take(group.len() - usize::from(suit.is_some()))
    {
        let kind = if let Some(suit) = suit {
            if !(b'1'..=b'9').contains(&byte) {
                return None;
            }
            MahjongTileKind::suited(suit, byte - b'0')
        } else {
            match byte {
                b'E' => MahjongTileKind::Wind(MahjongWind::East),
                b'S' => MahjongTileKind::Wind(MahjongWind::South),
                b'W' => MahjongTileKind::Wind(MahjongWind::West),
                b'N' => MahjongTileKind::Wind(MahjongWind::North),
                b'R' => MahjongTileKind::Dragon(MahjongDragon::Red),
                b'G' => MahjongTileKind::Dragon(MahjongDragon::Green),
                b'H' => MahjongTileKind::Dragon(MahjongDragon::White),
                b'a'..=b'h' => MahjongTileKind::Flower(MahjongFlower::ALL[(byte - b'a') as usize]),
                _ => return None,
            }
        };
        tiles.push(kind);
    }
    (!tiles.is_empty()).then_some((exposed, tiles))
}

pub(super) fn scroll_mahjong_fan_guide(
    mut wheels: MessageReader<MouseWheel>,
    mut scrolls: Query<
        (&RelativeCursorPosition, &mut ScrollPosition, &ComputedNode),
        With<MahjongFanGuideScroll>,
    >,
) {
    let delta = wheels
        .read()
        .map(|wheel| match wheel.unit {
            MouseScrollUnit::Line => wheel.y * 40.0,
            MouseScrollUnit::Pixel => wheel.y,
        })
        .sum::<f32>();
    if delta == 0.0 {
        return;
    }
    for (cursor, mut position, node) in &mut scrolls {
        if cursor.cursor_over() {
            let maximum =
                ((node.content_size().y - node.size().y) * node.inverse_scale_factor()).max(0.0);
            position.y = (position.y - delta).clamp(0.0, maximum);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::shell::DomainUiAction;
    use std::collections::HashSet;

    #[test]
    fn catalogue_has_all_81_fans_and_valid_examples() {
        assert_eq!(ENTRIES.len(), 81);
        let mut seen = HashSet::new();
        for entry in ENTRIES {
            assert!(seen.insert(std::mem::discriminant(&entry.fan)));
            assert!(TIERS.contains(&entry.fan.points()));
            let mut counts = [0u8; 34];
            let mut total = 0;
            let mut kongs = 0;
            for token in entry.example.split_ascii_whitespace() {
                let (_, tiles) = parse_group(token).expect("invalid fan example");
                total += tiles.len();
                kongs += usize::from(tiles.len() == 4);
                for tile in tiles {
                    if let Some(index) = tile.index34() {
                        counts[index] += 1;
                    }
                }
            }
            let expected = if entry.fan == leocard_mahjong::Fan::FlowerTiles {
                8
            } else {
                14 + kongs
            };
            assert_eq!(total, expected, "{}", entry.fan.name());
            assert!(
                counts.iter().all(|count| *count <= 4),
                "{}",
                entry.fan.name()
            );
        }
    }

    #[test]
    fn guide_keeps_hand_tiles_upright_and_all_melds_laid_down() {
        assert!(matches!(guide_tile_size(false), MahjongTileSize::GuideHand));
        assert!(matches!(guide_tile_size(true), MahjongTileSize::GuideMeld));
        assert!(matches!(
            guide_kong_size(false),
            MahjongTileSize::GuideConcealedMeld
        ));
        assert!(matches!(guide_kong_size(true), MahjongTileSize::GuideMeld));
        let tile = MahjongTileKind::suited(MahjongSuit::Characters, 1);
        assert_eq!(guide_base_kind(tile, true, false), None);
        assert_eq!(guide_base_kind(tile, true, true), Some(tile));
        assert!(!MahjongUiAction::SelectFanGuideTier(48).rebuilds_ui());
    }
}
