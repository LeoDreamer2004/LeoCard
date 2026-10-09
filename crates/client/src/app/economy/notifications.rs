use crate::app::presentation::{TransitionVisuals, add_text, ease_out_cubic, fade_panel};
use crate::app::runtime::UiAssets;
use bevy::{
    picking::Pickable,
    prelude::*,
    ui::{BackgroundGradient, ColorStop, FocusPolicy, LinearGradient},
};
use leocard_client::PlayerEconomy;
use std::collections::VecDeque;

const WIDTH: f32 = 280.0;
const HEIGHT: f32 = 34.0;
const ENTRY_DURATION: f32 = 0.28;
const HOLD_DURATION: f32 = 1.5;
const EXIT_DURATION: f32 = 0.36;

#[derive(Resource, Default)]
pub(super) struct CoinNotifications(VecDeque<i64>);

#[derive(Component)]
pub(super) struct CoinNoticeLayer;

#[derive(Component)]
pub(super) struct CoinNotice {
    elapsed: f32,
}

pub(super) fn setup_coin_notifications(mut commands: Commands) {
    // 独立于页面重建，所有页面复用；装饰层不参与鼠标拾取。
    commands.spawn((
        CoinNoticeLayer,
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            height: percent(100),
            overflow: Overflow::clip(),
            ..default()
        },
        GlobalZIndex(2600),
        FocusPolicy::Pass,
        Pickable::IGNORE,
    ));
}

pub(super) fn queue_coin_notifications(
    mut economy: ResMut<PlayerEconomy>,
    mut notices: ResMut<CoinNotifications>,
) {
    notices.0.extend(economy.take_coin_changes());
}

#[expect(
    clippy::too_many_arguments,
    reason = "Bevy injects the independent notice and subtree visual queries"
)]
pub(super) fn animate_coin_notifications(
    mut commands: Commands,
    time: Res<Time>,
    assets: Res<UiAssets>,
    mut notices: ResMut<CoinNotifications>,
    layers: Query<Entity, With<CoinNoticeLayer>>,
    mut active: Query<(Entity, &mut CoinNotice, &mut UiTransform)>,
    children: Query<&Children>,
    mut visuals: TransitionVisuals,
) {
    for (entity, mut notice, mut transform) in &mut active {
        notice.elapsed += time.delta_secs();
        if notice.elapsed >= ENTRY_DURATION + HOLD_DURATION + EXIT_DURATION {
            commands.entity(entity).despawn();
            continue;
        }
        let entry = (notice.elapsed / ENTRY_DURATION).clamp(0.0, 1.0);
        let exit =
            ((notice.elapsed - ENTRY_DURATION - HOLD_DURATION) / EXIT_DURATION).clamp(0.0, 1.0);
        transform.translation = Val2::px(
            WIDTH * (1.0 - ease_out_cubic(entry)),
            -HEIGHT * ease_out_cubic(exit),
        );
        fade_panel(entity, 1.0 - exit, &children, &mut visuals, &mut commands);
    }
    if active.is_empty()
        && let Ok(layer) = layers.single()
        && let Some(delta) = notices.0.pop_front()
    {
        spawn_notice(&mut commands, layer, delta, &assets);
    }
}

fn spawn_notice(commands: &mut Commands, layer: Entity, delta: i64, assets: &UiAssets) {
    let notice = commands
        .spawn((
            CoinNotice { elapsed: 0.0 },
            Node {
                position_type: PositionType::Absolute,
                right: px(0),
                top: px(82),
                width: px(WIDTH),
                height: px(HEIGHT),
                padding: UiRect::axes(px(18), px(4)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::FlexEnd,
                column_gap: px(7),
                ..default()
            },
            BackgroundGradient::from(LinearGradient::to_right(vec![
                ColorStop::percent(Color::NONE, 0.0),
                ColorStop::percent(Color::srgba(0.06, 0.06, 0.06, 0.60), 70.0),
                ColorStop::percent(Color::srgba(0.06, 0.06, 0.06, 0.76), 100.0),
            ])),
            UiTransform::from_translation(Val2::px(WIDTH, 0.0)),
            FocusPolicy::Pass,
            Pickable::IGNORE,
        ))
        .id();
    commands.entity(layer).add_child(notice);
    let icon = commands
        .spawn((
            Node {
                width: px(22),
                height: px(22),
                flex_shrink: 0.0,
                ..default()
            },
            ImageNode::new(assets.shop.coin.clone()),
            FocusPolicy::Pass,
            Pickable::IGNORE,
        ))
        .id();
    commands.entity(notice).add_child(icon);
    let text = add_text(
        commands,
        notice,
        format!("金币 {delta:+}"),
        18.0,
        Color::srgb(0.82, 0.82, 0.82),
        assets,
    );
    commands.entity(text).insert((
        TextLayout::default().with_no_wrap(),
        FocusPolicy::Pass,
        Pickable::IGNORE,
    ));
}
