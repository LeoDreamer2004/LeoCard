use super::geometry::DirectionLoopGeometry;
use super::material::UnoDirectionMaterial;
use super::view::UnoDirectionLoop;
use crate::app::presentation::GAME_SIDE_CONTROL_SCALE;
use crate::app::runtime::ClientResource;
use bevy::ecs::query::QueryData;
use bevy::prelude::*;
use leocard_protocol::{GameSnapshot, MatchId, UnoPhaseView};
use leocard_uno::UnoDirection;

#[derive(Resource, Default)]
pub(crate) struct UnoDirectionState {
    match_id: Option<MatchId>,
    phase: f32,
    speed: f32,
    elapsed: f32,
    geometry: Option<DirectionLoopGeometry>,
    material: Option<Handle<UnoDirectionMaterial>>,
}

#[derive(QueryData)]
pub(crate) struct DirectionLoopLayer {
    entity: Entity,
    node: &'static ComputedNode,
    material: Option<&'static MaterialNode<UnoDirectionMaterial>>,
}

pub(crate) fn animate_uno_direction_loop(
    mut commands: Commands,
    client: Option<Res<ClientResource>>,
    time: Res<Time>,
    mut state: ResMut<UnoDirectionState>,
    mut materials: ResMut<Assets<UnoDirectionMaterial>>,
    layers: Query<DirectionLoopLayer, With<UnoDirectionLoop>>,
) {
    let Some(game) = client
        .as_deref()
        .and_then(|client| client.0.model().game_snapshot().and_then(GameSnapshot::uno))
        .filter(|game| matches!(game.phase, UnoPhaseView::Playing))
    else {
        if state.match_id.is_some() {
            *state = UnoDirectionState::default();
        }
        return;
    };
    let target = match game.direction {
        UnoDirection::Clockwise => 1.0,
        UnoDirection::CounterClockwise => -1.0,
    };
    if state.match_id != Some(game.match_id) {
        *state = UnoDirectionState {
            match_id: Some(game.match_id),
            speed: target,
            ..default()
        };
    }
    let Ok(layer) = layers.single() else {
        return;
    };
    let node = layer.node;
    if node.size().min_element() <= 1.0 {
        return;
    }
    let size = node.size() * node.inverse_scale_factor();
    if state
        .geometry
        .as_ref()
        .is_none_or(|geometry| !geometry.matches(size))
    {
        state.geometry = Some(DirectionLoopGeometry::new(
            size,
            42.0 * GAME_SIDE_CONTROL_SCALE,
        ));
    }
    let delta = time.delta_secs().min(0.1);
    state.speed += (target - state.speed).clamp(-6.0 * delta, 6.0 * delta);
    state.phase = state
        .geometry
        .as_ref()
        .expect("direction loop has a canvas")
        .advance(state.phase, state.speed, delta);
    state.elapsed += delta;
    let direction = if state.speed < 0.0 { -1.0 } else { 1.0 };
    let entry = (state.elapsed / 0.35).min(1.0);
    let opacity = entry * entry * (3.0 - 2.0 * entry);
    let geometry = state
        .geometry
        .as_ref()
        .expect("direction loop has a canvas");
    let material = UnoDirectionMaterial {
        canvas: Vec4::new(size.x, size.y, geometry.center.x, geometry.center.y),
        shape: Vec4::new(
            geometry.radii.x,
            geometry.radii.y,
            geometry.width,
            geometry.perimeter,
        ),
        motion: Vec4::new(state.phase, direction, state.speed.abs(), opacity),
        color: Color::srgba(0.86, 0.84, 0.74, 0.045).to_linear(),
    };
    let handle = if let Some(handle) = state.material.as_ref() {
        *materials
            .get_mut(handle)
            .expect("direction loop retains its material") = material;
        handle.clone()
    } else {
        let handle = materials.add(material);
        state.material = Some(handle.clone());
        handle
    };
    if layer.material.is_none_or(|node| node.0 != handle) {
        commands.entity(layer.entity).insert(MaterialNode(handle));
    }
}
