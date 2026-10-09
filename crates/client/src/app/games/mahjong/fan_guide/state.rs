use self::catalog::{ENTRIES, FanGuideEntry};
use super::super::actions::MahjongUiAction;
use super::super::{
    MAHJONG_KONG_STACK_LIFT, MahjongAssets, MahjongTileHighlight, MahjongTileMaterial,
    MahjongTileSize, MahjongTileVisual, MahjongUiState, add_mahjong_tile_material,
};
use super::*;
use crate::app::presentation::{ButtonHighlight, MUTED, TEXT, add_text, spawn_node};
use crate::app::runtime::UiAssets;
use crate::app::shell::{
    CozyModalBackdrop, CozyModalKind, CozyModalPanel, UiAction, add_cozy_close_button,
    add_cozy_panel, cozy_backdrop_color, cozy_panel_transform,
};
use bevy::prelude::*;
use bevy::ui::{FocusPolicy, RelativeCursorPosition, VisualBox};
use leocard_mahjong::{MahjongDragon, MahjongFlower, MahjongSuit, MahjongTileKind, MahjongWind};

pub(super) const TIERS: [u16; 12] = [88, 64, 48, 32, 24, 16, 12, 8, 6, 4, 2, 1];

#[derive(Component)]
pub(in super::super) struct MahjongFanGuideScroll;

#[derive(Component)]
pub(in super::super) struct MahjongFanGuideRoot(pub(super) u16);

#[derive(Component)]
pub(in super::super) struct MahjongFanGuideTile;

pub(in super::super) fn sync_mahjong_fan_guide(
    mut commands: Commands,
    ui: Res<MahjongUiState>,
    roots: Query<(Entity, &MahjongFanGuideRoot)>,
    assets: Res<UiAssets>,
    game_assets: Res<MahjongAssets>,
    mut materials: ResMut<Assets<MahjongTileMaterial>>,
) {
    let tier = (ui.fan_guide_open || ui.fan_guide_progress > 0.0).then_some(ui.fan_guide_tier);
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
        ui.fan_guide_progress,
        &assets,
        &game_assets,
        &mut materials,
    );
}

pub(in super::super) fn animate_mahjong_fan_guide_tiles(
    ui: Res<MahjongUiState>,
    mut tiles: Query<
        (&MaterialNode<MahjongTileMaterial>, Option<&mut BoxShadow>),
        With<MahjongFanGuideTile>,
    >,
    mut materials: ResMut<Assets<MahjongTileMaterial>>,
) {
    let progress = ui.fan_guide_progress;
    let opacity = progress * progress * (3.0 - 2.0 * progress);
    for (node, shadow) in &mut tiles {
        if let Some(mut material) = materials.get_mut(&node.0) {
            material.params.z = opacity;
        }
        if let Some(mut shadow) = shadow {
            *shadow = BoxShadow::new(
                Color::BLACK.with_alpha(0.24 * opacity),
                px(2),
                px(5),
                px(0),
                px(4),
            );
        }
    }
}

pub(super) fn render_fan_guide(
    commands: &mut Commands,
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
        FocusPolicy::Block,
        CozyModalBackdrop(CozyModalKind::MahjongFanGuide),
    ));
    let window = add_cozy_panel(
        commands,
        backdrop,
        Node {
            width: px(1020),
            max_width: percent(92),
            height: percent(88),
            max_height: px(950),
            min_height: px(520),
            padding: UiRect::all(px(24)),
            flex_direction: FlexDirection::Column,
            row_gap: px(12),
            ..default()
        },
        assets,
    );
    commands.entity(window).insert((
        GlobalZIndex(2201),
        FocusPolicy::Block,
        CozyModalPanel(CozyModalKind::MahjongFanGuide),
        cozy_panel_transform(progress),
    ));
    render_header(commands, window, assets);
    spawn_node(
        commands,
        window,
        Node {
            width: px(164),
            height: px(2),
            ..default()
        },
        Some(Color::srgb(0.64, 0.59, 0.93)),
    );
    let body = spawn_node(
        commands,
        window,
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
    add_text(
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

pub(super) fn render_header(commands: &mut Commands, parent: Entity, assets: &UiAssets) {
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
    add_text(commands, header, "国标麻将番种", 27.0, TEXT, assets);
    add_cozy_close_button(
        commands,
        header,
        UiAction::Mahjong(MahjongUiAction::CloseFanGuide),
        assets,
    );
}

pub(super) fn render_tabs(
    commands: &mut Commands,
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
        let tab = commands
            .spawn((
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
            ))
            .id();
        commands.entity(tabs).add_child(tab);
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
            let overlay = commands
                .spawn((
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
                    FocusPolicy::Pass,
                ))
                .id();
            commands.entity(tab).add_child(overlay);
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
        commands.entity(label).insert(FocusPolicy::Pass);
    }
}

pub(super) fn render_fan_entry(
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
    add_text(commands, heading, entry.fan.name(), 19.0, color, assets);
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

pub(super) fn render_example(
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
            let tile = add_mahjong_tile_material(
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
                    highlight: MahjongTileHighlight::None,
                    deal: None,
                    relative: 0,
                },
                game_assets,
                materials,
            );
            commands.entity(tile).insert(MahjongFanGuideTile);
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
                    highlight: MahjongTileHighlight::None,
                    deal: None,
                    relative: 0,
                },
                game_assets,
                materials,
            );
            commands.entity(stacked_tile).insert((
                MahjongFanGuideTile,
                BoxShadow::new(Color::BLACK.with_alpha(0.24), px(2), px(5), px(0), px(4)),
            ));
        }
    }
}

pub(super) fn guide_tile_size(exposed: bool) -> MahjongTileSize {
    if exposed {
        MahjongTileSize::GuideMeld
    } else {
        MahjongTileSize::GuideHand
    }
}

pub(super) fn guide_kong_size(exposed: bool) -> MahjongTileSize {
    if exposed {
        MahjongTileSize::GuideMeld
    } else {
        MahjongTileSize::GuideConcealedMeld
    }
}

pub(super) fn guide_base_kind(
    kind: MahjongTileKind,
    kong: bool,
    exposed: bool,
) -> Option<MahjongTileKind> {
    if kong && !exposed { None } else { Some(kind) }
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

pub(super) fn parse_group(group: &str) -> Option<(bool, Vec<MahjongTileKind>)> {
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

#[cfg(test)]
mod tests {
    use super::*;
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
}
