//! Nonblocking achievement presentation. Awards arrive after persistence.

use super::{AchievementRecipient, AchievementUnlocked};
use crate::app::presentation::{
    TEXT, TransitionVisuals, add_text, ease_out_cubic, fade_panel, spawn_node,
};
use crate::app::runtime::UiAssets;
use bevy::audio::Volume;
use bevy::prelude::*;
use bevy::ui::{FocusPolicy, VisualBox};
use leocard_achievements::{AchievementDefinition, AchievementTier};
use std::collections::VecDeque;

const ENTRY_DURATION: f32 = 0.32;
const HOLD_DURATION: f32 = 4.2;
const EXIT_DURATION: f32 = 0.30;
const STAGGER: f32 = 0.26;
const TOP: f32 = 80.0;
const GAP: f32 = 10.0;
const MIN_HEIGHT: f32 = 80.0;

struct AchievementNotice {
    definition: &'static AchievementDefinition,
    text: String,
}

#[derive(Resource, Default)]
pub(super) struct AchievementNotifications {
    pending: VecDeque<AchievementNotice>,
    since_spawn: f32,
    serial: u64,
}

#[derive(Component)]
pub(super) struct AchievementToastLayer;

#[derive(Component)]
pub(super) struct AchievementToast {
    serial: u64,
    elapsed: f32,
    visual: Entity,
}

pub(super) fn setup_notifications(mut commands: Commands) {
    // This root survives screen rebuilds and does not capture pointer input.
    commands.spawn((
        AchievementToastLayer,
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            top: px(0),
            width: percent(100),
            height: percent(100),
            ..default()
        },
        GlobalZIndex(2500),
        FocusPolicy::Pass,
    ));
}

pub(super) fn queue_achievement_notifications(
    mut unlocked: MessageReader<AchievementUnlocked>,
    mut notifications: ResMut<AchievementNotifications>,
) {
    for event in unlocked.read() {
        let subject = match &event.recipient {
            AchievementRecipient::Local => "你",
            AchievementRecipient::TablePlayer { name } => name.as_str(),
        };
        notifications.pending.push_back(AchievementNotice {
            definition: event.definition,
            text: format!("{subject}获得了「{}」成就！", event.definition.title),
        });
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "Bevy injects independent toast, layout, sound and visual resources"
)]
pub(super) fn animate_achievement_notifications(
    time: Res<Time>,
    assets: Res<UiAssets>,
    mut notifications: ResMut<AchievementNotifications>,
    layers: Query<(Entity, &ComputedNode), With<AchievementToastLayer>>,
    mut toasts: Query<(Entity, &mut AchievementToast, &mut Node, &ComputedNode)>,
    mut transforms: Query<&mut UiTransform>,
    children: Query<&Children>,
    mut visuals: TransitionVisuals,
    mut commands: Commands,
) {
    let Ok((layer, viewport)) = layers.single() else {
        return;
    };
    let mut order = toasts
        .iter()
        .map(|(entity, toast, _, node)| {
            (
                toast.serial,
                entity,
                (node.size().y * node.inverse_scale_factor()).max(MIN_HEIGHT),
            )
        })
        .collect::<Vec<_>>();
    order.sort_by_key(|(serial, _, _)| *serial);
    let mut target_top = TOP;
    for (_, entity, height) in &order {
        let Ok((_, mut toast, mut node, _)) = toasts.get_mut(*entity) else {
            continue;
        };
        toast.elapsed += time.delta_secs();
        if toast.elapsed >= ENTRY_DURATION + HOLD_DURATION + EXIT_DURATION {
            commands.entity(*entity).despawn();
            continue;
        }
        let current_top = if let Val::Px(top) = node.top {
            top
        } else {
            target_top
        };
        let position =
            current_top + (target_top - current_top) * (1.0 - (-time.delta_secs() * 18.0).exp());
        node.top = px(position);
        let entering = (toast.elapsed / ENTRY_DURATION).clamp(0.0, 1.0);
        let exiting =
            ((toast.elapsed - ENTRY_DURATION - HOLD_DURATION) / EXIT_DURATION).clamp(0.0, 1.0);
        if let Ok(mut transform) = transforms.get_mut(toast.visual) {
            *transform = UiTransform::from_translation(Val2::px(
                0.0,
                -(position + height) * (1.0 - ease_out_cubic(entering))
                    - height * ease_out_cubic(exiting),
            ));
        }
        fade_panel(
            *entity,
            entering.min(1.0 - exiting),
            &children,
            &mut visuals,
            &mut commands,
        );
        target_top += height + GAP;
    }
    notifications.since_spawn += time.delta_secs();
    let view_height = viewport.size().y * viewport.inverse_scale_factor();
    let capacity = ((view_height * 0.7 - TOP) / (MIN_HEIGHT + GAP))
        .floor()
        .clamp(1.0, 4.0) as usize;
    if order.len() < capacity
        && notifications.since_spawn >= STAGGER
        && let Some(notice) = notifications.pending.pop_front()
    {
        notifications.since_spawn = 0.0;
        let serial = notifications.serial;
        notifications.serial += 1;
        spawn_notification(&mut commands, layer, serial, target_top, &notice, &assets);
        let sound = if notice.definition.tier == AchievementTier::Gold {
            &assets.achievements.gold_sound
        } else {
            &assets.achievements.sound
        };
        commands.spawn((
            AudioPlayer::new(sound.clone()),
            PlaybackSettings {
                volume: Volume::Linear(0.55),
                ..PlaybackSettings::DESPAWN
            },
        ));
    }
}

fn spawn_notification(
    commands: &mut Commands,
    layer: Entity,
    serial: u64,
    top: f32,
    notice: &AchievementNotice,
    assets: &UiAssets,
) {
    let anchor = spawn_node(
        commands,
        layer,
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            left: px(0),
            right: px(0),
            top: px(top),
            justify_content: JustifyContent::Center,
            // Moving notices reveal within their own slot, never across another.
            overflow: Overflow::clip_y(),
            ..default()
        },
        None,
    );
    let texture = if notice.definition.tier == AchievementTier::Gold {
        &assets.achievements.toast_gold
    } else {
        &assets.achievements.toast
    };
    let popup = spawn_node(
        commands,
        anchor,
        Node {
            width: px(620),
            min_width: px(480),
            max_width: percent(84),
            min_height: px(MIN_HEIGHT),
            padding: UiRect::axes(px(22), px(10)),
            align_items: AlignItems::Center,
            column_gap: px(14),
            ..default()
        },
        None,
    );
    commands.entity(anchor).insert((
        AchievementToast {
            serial,
            elapsed: 0.0,
            visual: popup,
        },
        FocusPolicy::Pass,
    ));
    commands
        .entity(popup)
        .insert(UiTransform::from_translation(Val2::px(0.0, -1000.0)));
    let mut background =
        ImageNode::new(texture.clone()).with_mode(NodeImageMode::Sliced(TextureSlicer {
            border: BorderRect::all(20.0),
            center_scale_mode: SliceScaleMode::Stretch,
            sides_scale_mode: SliceScaleMode::Stretch,
            max_corner_scale: 0.7,
        }));
    background.visual_box = VisualBox::BorderBox;
    commands
        .entity(popup)
        .insert((background, FocusPolicy::Pass));
    let trophy = commands
        .spawn((
            Node {
                width: px(54),
                height: px(54),
                flex_shrink: 0.0,
                ..default()
            },
            ImageNode::new(
                assets.achievements.medals[notice.definition.tier.medal_index()].clone(),
            ),
            FocusPolicy::Pass,
        ))
        .id();
    commands.entity(popup).add_child(trophy);
    let text = add_text(commands, popup, &notice.text, 21.0, TEXT, assets);
    commands.entity(text).insert((
        Node {
            min_width: px(0),
            flex_grow: 1.0,
            ..default()
        },
        FocusPolicy::Pass,
    ));
}
