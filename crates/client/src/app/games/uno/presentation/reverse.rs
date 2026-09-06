use super::*;

pub(super) fn spawn_uno_reverse_effect(
    commands: &mut Commands,
    layer: Entity,
    layer_node: &ComputedNode,
    layer_transform: &UiGlobalTransform,
    game: &UnoSnapshot,
    anchors: &Query<(&PlayerAvatarAnchor, &ComputedNode, &UiGlobalTransform)>,
    effect_color: Color,
) {
    let layer_size = layer_node.size() * layer_node.inverse_scale_factor();
    let own_effect_anchor = uno_reverse_own_anchor(layer_size);
    let mut ring = game
        .players
        .iter()
        .filter(|player| !player.eliminated)
        .filter_map(|player| {
            let position = if player.id == game.you {
                own_effect_anchor
            } else {
                uno_player_anchor_in_layer(player.id, layer_node, layer_transform, anchors)?
            };
            Some((player.id, position))
        })
        .collect::<Vec<_>>();
    let center = layer_size * 0.5;
    ring.sort_by(|(_, left), (_, right)| {
        let left_angle = (left.y - center.y).atan2(left.x - center.x);
        let right_angle = (right.y - center.y).atan2(right.x - center.x);
        left_angle.total_cmp(&right_angle)
    });
    if matches!(game.direction, UnoDirection::CounterClockwise) {
        ring.reverse();
    }
    if ring.len() < 2 {
        return;
    }
    let pair_count = if ring.len() == 2 { 1 } else { ring.len() };
    for index in 0..pair_count {
        let start = ring[index].1;
        let end = ring[(index + 1) % ring.len()].1;
        let midpoint = (start + end) * 0.5;
        let control = center + (midpoint - center) * 1.28;
        const SEGMENTS: usize = 18;
        for step in 0..SEGMENTS {
            let t0 = 0.13 + step as f32 / SEGMENTS as f32 * 0.64;
            let t1 = 0.13 + (step + 1) as f32 / SEGMENTS as f32 * 0.64;
            let start_point = quadratic_bezier(start, control, end, t0);
            let end_point = quadratic_bezier(start, control, end, t1);
            let direction = end_point - start_point;
            let t = (t0 + t1) * 0.5;
            let thickness = 7.0 + ((t - 0.13) / 0.64).clamp(0.0, 1.0) * 5.0;
            spawn_uno_reverse_arrow_part(
                commands,
                layer,
                (start_point + end_point) * 0.5,
                direction.y.atan2(direction.x).to_degrees(),
                index as f32 * 0.035 + step as f32 * 0.012,
                Vec2::new(direction.length() + 7.0, thickness),
                effect_color,
            );
        }
        let t = 0.84;
        let tip = quadratic_bezier(start, control, end, t);
        let tangent = (control - start) * (2.0 * (1.0 - t)) + (end - control) * (2.0 * t);
        let direction = tangent.normalize_or(Vec2::X);
        let side = Vec2::new(-direction.y, direction.x);
        let angle = direction.y.atan2(direction.x).to_degrees();
        let back = tip - direction * 16.0;
        let head_delay = index as f32 * 0.035 + 0.24;
        spawn_uno_reverse_arrow_part(
            commands,
            layer,
            back - side * 9.0,
            angle + 34.0,
            head_delay,
            Vec2::new(40.0, 13.0),
            effect_color,
        );
        spawn_uno_reverse_arrow_part(
            commands,
            layer,
            back + side * 9.0,
            angle - 34.0,
            head_delay,
            Vec2::new(40.0, 13.0),
            effect_color,
        );
    }
}

/// 反转特效把自己视作下方居中的操作区座位，而不是左下角的信息框。
fn uno_reverse_own_anchor(layer_size: Vec2) -> Vec2 {
    Vec2::new(
        layer_size.x * 0.5,
        layer_size.y - UNO_ACTION_AREA_BOTTOM - UNO_ACTION_AREA_HEIGHT * 0.5,
    )
}

fn spawn_uno_reverse_arrow_part(
    commands: &mut Commands,
    layer: Entity,
    position: Vec2,
    angle: f32,
    delay: f32,
    size: Vec2,
    effect_color: Color,
) {
    let radians = angle.to_radians();
    let normal = Vec2::new(-radians.sin(), radians.cos());
    for (offset, layer_size, color, max_alpha, shadow_alpha, z_index, extra_delay) in [
        (
            Vec2::new(2.5, 4.0),
            Vec2::new(size.x + 5.0, size.y + 6.0),
            Color::BLACK,
            0.38,
            0.24,
            1489,
            0.0,
        ),
        (
            Vec2::new(1.2, 2.6),
            Vec2::new(size.x + 1.0, size.y + 1.5),
            effect_color.mix(&Color::BLACK, 0.46),
            0.94,
            0.18,
            1490,
            0.010,
        ),
        (Vec2::ZERO, size, effect_color, 0.98, 0.20, 1491, 0.022),
        (
            -normal * (size.y * 0.30),
            Vec2::new((size.x - 5.0).max(4.0), (size.y * 0.18).max(1.8)),
            effect_color.mix(&Color::WHITE, 0.74),
            0.82,
            0.06,
            1492,
            0.060,
        ),
    ] {
        spawn_uno_reverse_arrow_layer(
            commands,
            layer,
            position + offset,
            angle,
            delay + extra_delay,
            layer_size,
            color,
            max_alpha,
            shadow_alpha,
            z_index,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn spawn_uno_reverse_arrow_layer(
    commands: &mut Commands,
    layer: Entity,
    position: Vec2,
    angle: f32,
    delay: f32,
    size: Vec2,
    color: Color,
    max_alpha: f32,
    shadow_alpha: f32,
    z_index: i32,
) {
    let arrow = commands
        .spawn((
            UnoReverseArrow {
                elapsed: 0.0,
                delay,
                color,
                max_alpha,
                shadow_alpha,
            },
            Node {
                position_type: PositionType::Absolute,
                left: px(position.x - size.x * 0.5),
                top: px(position.y - size.y * 0.5),
                width: px(size.x),
                height: px(size.y),
                border_radius: BorderRadius::all(percent(50)),
                ..default()
            },
            BackgroundColor(color.with_alpha(0.0)),
            UiTransform::from_rotation(Rot2::degrees(angle)),
            BoxShadow::new(Color::BLACK.with_alpha(0.0), px(1), px(2), px(0), px(4)),
            GlobalZIndex(z_index),
            FocusPolicy::Pass,
        ))
        .id();
    commands.entity(layer).add_child(arrow);
}

pub(super) fn quadratic_bezier(start: Vec2, control: Vec2, end: Vec2, t: f32) -> Vec2 {
    start * (1.0 - t).powi(2) + control * (2.0 * (1.0 - t) * t) + end * t.powi(2)
}
