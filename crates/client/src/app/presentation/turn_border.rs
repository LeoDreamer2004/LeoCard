//! Shader-driven rounded trace around the player whose turn is active.

use bevy::prelude::*;
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::ShaderRef;
use bevy::ui::FocusPolicy;
use leocard_protocol::{GameKind, MatchId, PlayerId};
use std::collections::{HashMap, HashSet};

use crate::app::*;

const TURN_BORDER_GROW_DURATION: f32 = 0.95;
const TURN_BORDER_HOLD_DURATION: f32 = 0.14;
const TURN_BORDER_SHRINK_DURATION: f32 = 0.95;
const TURN_BORDER_GAP_DURATION: f32 = 0.18;
const TURN_BORDER_RADIUS: f32 = 8.0;
pub const TURN_BORDER_THICKNESS: f32 = 2.6;
const TURN_BORDER_OUTSET: f32 = TURN_BORDER_THICKNESS * 0.5 + 0.75;

#[derive(AsBindGroup, Asset, TypePath, Debug, Clone)]
pub struct TurnBorderMaterial {
    /// x: normalized tail, y: normalized head, z: corner radius, w: line thickness.
    #[uniform(0)]
    params: Vec4,
    /// 与人物框大分数使用同一个 ACCENT 色，并由 Bevy 转换到线性色彩空间。
    #[uniform(0)]
    color: LinearRgba,
}

impl UiMaterial for TurnBorderMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/turn_border.wgsl".into()
    }
}

#[derive(Component)]
pub struct TurnBorderTrace {
    key: TurnBorderAnimationKey,
    material: Handle<TurnBorderMaterial>,
}

/// 描边进度不能挂在人物框实体上：手牌选择等操作会重建整棵 UI，实体随之销毁。
/// 使用“游戏 + 对局 + 当前玩家”作为稳定键，可在同一回合的 UI 重建后继续播放；
/// 真正换人或换局时则自然从头开始。
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct TurnBorderAnimationKey {
    game: GameKind,
    match_id: MatchId,
    player: PlayerId,
}

impl TurnBorderAnimationKey {
    pub fn new(game: GameKind, match_id: MatchId, player: PlayerId) -> Self {
        Self {
            game,
            match_id,
            player,
        }
    }
}

#[derive(Default, Resource)]
pub struct TurnBorderAnimationState {
    elapsed: HashMap<TurnBorderAnimationKey, f32>,
}

pub fn add_turn_border_trace(
    commands: &mut Commands,
    panel: Entity,
    materials: &mut Assets<TurnBorderMaterial>,
    key: TurnBorderAnimationKey,
) {
    let material = materials.add(TurnBorderMaterial {
        params: Vec4::new(
            0.0,
            0.0,
            TURN_BORDER_RADIUS + TURN_BORDER_OUTSET,
            TURN_BORDER_THICKNESS,
        ),
        color: ACCENT.to_linear(),
    });
    commands.entity(panel).insert(TurnBorderTrace {
        key,
        material: material.clone(),
    });
    let trace = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(-TURN_BORDER_OUTSET),
                right: px(-TURN_BORDER_OUTSET),
                top: px(-TURN_BORDER_OUTSET),
                bottom: px(-TURN_BORDER_OUTSET),
                ..default()
            },
            MaterialNode(material),
            ZIndex(90),
            FocusPolicy::Pass,
        ))
        .id();
    commands.entity(panel).add_child(trace);
}

pub fn turn_border_visible_interval(elapsed: f32, perimeter: f32) -> (f32, f32) {
    let cycle = TURN_BORDER_GROW_DURATION
        + TURN_BORDER_HOLD_DURATION
        + TURN_BORDER_SHRINK_DURATION
        + TURN_BORDER_GAP_DURATION;
    let phase = elapsed.rem_euclid(cycle);
    if phase < TURN_BORDER_GROW_DURATION {
        let progress = smootherstep(phase / TURN_BORDER_GROW_DURATION);
        (0.0, perimeter * progress)
    } else if phase < TURN_BORDER_GROW_DURATION + TURN_BORDER_HOLD_DURATION {
        (0.0, perimeter)
    } else if phase
        < TURN_BORDER_GROW_DURATION + TURN_BORDER_HOLD_DURATION + TURN_BORDER_SHRINK_DURATION
    {
        let progress = smootherstep(
            (phase - TURN_BORDER_GROW_DURATION - TURN_BORDER_HOLD_DURATION)
                / TURN_BORDER_SHRINK_DURATION,
        );
        (perimeter * progress, perimeter)
    } else {
        (perimeter, perimeter)
    }
}

pub fn animate_turn_border_traces(
    time: Res<Time>,
    mut materials: ResMut<Assets<TurnBorderMaterial>>,
    mut animation: ResMut<TurnBorderAnimationState>,
    traces: Query<(&Visibility, &TurnBorderTrace)>,
) {
    let mut active = HashSet::new();
    for (visibility, trace) in &traces {
        let elapsed = if *visibility == Visibility::Hidden {
            0.0
        } else {
            if active.insert(trace.key) {
                *animation.elapsed.entry(trace.key).or_default() += time.delta_secs();
            }
            animation
                .elapsed
                .get(&trace.key)
                .copied()
                .unwrap_or_default()
        };
        let (tail, head) = turn_border_visible_interval(elapsed, 1.0);
        if let Some(mut material) = materials.get_mut(&trace.material) {
            material.params.x = tail;
            material.params.y = head;
        }
    }
    animation.elapsed.retain(|key, _| active.contains(key));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rebuilding_the_same_active_player_trace_keeps_animation_progress() {
        let mut app = App::new();
        let mut time = Time::<()>::default();
        time.advance_by(std::time::Duration::from_millis(250));
        app.insert_resource(time);
        app.insert_resource(Assets::<TurnBorderMaterial>::default());
        app.insert_resource(TurnBorderAnimationState::default());
        app.add_systems(Update, animate_turn_border_traces);

        let key = TurnBorderAnimationKey::new(GameKind::Shengji, MatchId([9; 16]), PlayerId(2));
        let first = app
            .world_mut()
            .spawn((
                Visibility::Visible,
                TurnBorderTrace {
                    key,
                    material: Handle::default(),
                },
            ))
            .id();
        app.update();
        assert_eq!(
            app.world()
                .resource::<TurnBorderAnimationState>()
                .elapsed
                .get(&key),
            Some(&0.25)
        );

        // 模拟手牌交互触发的 UiRoot 重建：旧人物框实体消失，但同一帧会产生
        // 拥有相同稳定键的新人物框，动画必须续播而不是归零。
        app.world_mut().despawn(first);
        app.world_mut().spawn((
            Visibility::Visible,
            TurnBorderTrace {
                key,
                material: Handle::default(),
            },
        ));
        app.update();
        assert_eq!(
            app.world()
                .resource::<TurnBorderAnimationState>()
                .elapsed
                .get(&key),
            Some(&0.5)
        );
    }
}
