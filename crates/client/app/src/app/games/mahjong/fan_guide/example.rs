use super::super::{
    MAHJONG_KONG_STACK_LIFT, MahjongAssets, MahjongTileHighlight, MahjongTileMaterial,
    MahjongTileSize, MahjongTileVisual, add_mahjong_tile_material,
};
use super::state::MahjongFanGuideTile;
use bevy::prelude::*;
use leocard_mahjong::{MahjongDragon, MahjongFlower, MahjongSuit, MahjongTileKind, MahjongWind};
use leocard_ui::{UiComposer, spawn_node};

pub(super) fn render_example(
    commands: &mut UiComposer,
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
                    elevation: 0.0,
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
                    elevation: 0.0,
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
    use super::super::catalog::ENTRIES;
    use super::super::view::TIERS;
    use super::*;
    use leocard_mahjong::MahjongFan;
    use std::collections::HashSet;
    use std::mem;

    #[test]
    fn catalogue_has_all_81_fans_and_valid_examples() {
        assert_eq!(ENTRIES.len(), 81);
        let mut seen = HashSet::new();
        for entry in ENTRIES {
            assert!(seen.insert(mem::discriminant(&entry.fan)));
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
            let expected = if entry.fan == MahjongFan::FlowerTiles {
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
