//! A shared painter's order lets moving tiles settle among static tiles without a layer jump.
use super::{MahjongTableRoot, MahjongTileMaterial, MahjongTileSurface};
use bevy::picking::Pickable;
use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

#[derive(Component)]
pub(super) struct MahjongPaintLayer {
    shadow: bool,
}

#[derive(Component)]
pub(super) struct MahjongTilePaint {
    source: Entity,
    shadow: bool,
}

#[expect(
    clippy::type_complexity,
    clippy::too_many_arguments,
    reason = "source nodes retain layout and animation while painting nodes share depth-sorted layers"
)]
pub(super) fn sync_mahjong_tile_surfaces(
    mut commands: Commands,
    tiles: Query<(Entity, &MahjongTileSurface, Option<&GlobalZIndex>)>,
    mut paints: Query<
        (
            Entity,
            &MahjongTilePaint,
            &mut MaterialNode<MahjongTileMaterial>,
            &mut Visibility,
            &mut ZIndex,
            Option<&mut GlobalZIndex>,
        ),
        Without<MahjongTileSurface>,
    >,
    layers: Query<(Entity, &ChildOf, &MahjongPaintLayer)>,
    parents: Query<&ChildOf>,
    source_visibility: Query<&Visibility, Without<MahjongTilePaint>>,
    tables: Query<(), With<MahjongTableRoot>>,
    mut materials: ResMut<Assets<MahjongTileMaterial>>,
) {
    let mut retained = HashSet::new();
    for (entity, paint, mut node, mut visibility, mut order, global) in &mut paints {
        let Ok((_, surface, source_global)) = tiles.get(paint.source) else {
            commands.entity(entity).despawn();
            continue;
        };
        let Some(mut material) = materials.get(&surface.0).cloned() else {
            continue;
        };
        if !material.table_projection {
            commands.entity(entity).despawn();
            continue;
        }
        retained.insert((paint.source, paint.shadow));
        if paint.shadow {
            material.shadow_only = 1.0;
            if let Some(mut target) = materials.get_mut(&node.0)
                && *target != material
            {
                *target = material;
            }
        } else {
            if node.0 != surface.0 {
                node.0 = surface.0.clone();
            }
            order.set_if_neq(ZIndex(material.paint_order));
            // Win emphasis is temporary. Ordinary flights always obey the same camera depth as resting tiles.
            if let Some(source_global) = source_global.filter(|value| value.0 >= 1000) {
                if let Some(mut global) = global {
                    global.set_if_neq(*source_global);
                } else {
                    commands.entity(entity).insert(*source_global);
                }
            } else if global.is_some() {
                commands.entity(entity).remove::<GlobalZIndex>();
            }
        }
        visibility.set_if_neq(paint_visibility(paint.source, &parents, &source_visibility));
    }
    let mut table_layers: HashMap<_, _> = layers
        .iter()
        .map(|(layer, parent, kind)| ((parent.parent(), kind.shadow), layer))
        .collect();
    for (source, surface, _) in &tiles {
        let Some(material) = materials.get(&surface.0).cloned() else {
            continue;
        };
        if !material.table_projection || material.ray_origin.w <= 0.0 {
            continue;
        }
        let mut ancestor = source;
        let mut table = None;
        if [false, true]
            .into_iter()
            .all(|shadow| retained.contains(&(source, shadow)))
        {
            continue;
        }
        while let Ok(parent) = parents.get(ancestor) {
            ancestor = parent.parent();
            if tables.contains(ancestor) {
                table = Some(ancestor);
                break;
            }
        }
        let Some(table) = table else { continue };
        for shadow in [true, false] {
            if retained.contains(&(source, shadow)) {
                continue;
            }
            let layer = *table_layers.entry((table, shadow)).or_insert_with(|| {
                commands
                    .spawn((
                        overlay_node(),
                        MahjongPaintLayer { shadow },
                        ZIndex(if shadow { -1 } else { 5 }),
                        Pickable::IGNORE,
                        ChildOf(table),
                    ))
                    .id()
            });
            let handle = if shadow {
                let mut copy = material.clone();
                copy.shadow_only = 1.0;
                materials.add(copy)
            } else {
                surface.0.clone()
            };
            commands.spawn((
                overlay_node(),
                MahjongTilePaint { source, shadow },
                MaterialNode(handle),
                paint_visibility(source, &parents, &source_visibility),
                ZIndex(if shadow { 0 } else { material.paint_order }),
                Pickable::IGNORE,
                ChildOf(layer),
            ));
        }
    }
}

fn overlay_node() -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: px(0),
        top: px(0),
        width: percent(100),
        height: percent(100),
        ..default()
    }
}

fn paint_visibility(
    mut source: Entity,
    parents: &Query<&ChildOf>,
    visibility: &Query<&Visibility, Without<MahjongTilePaint>>,
) -> Visibility {
    // A flight's handoff changes Visibility this frame; InheritedVisibility can still contain last frame's value.
    loop {
        match visibility.get(source) {
            Ok(Visibility::Hidden) => return Visibility::Hidden,
            Ok(Visibility::Visible) => return Visibility::Inherited,
            _ => {}
        }
        let Ok(parent) = parents.get(source) else {
            return Visibility::Inherited;
        };
        source = parent.parent();
    }
}
