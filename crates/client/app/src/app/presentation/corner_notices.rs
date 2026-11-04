use crate::app::presentation::{add_text, ease_out_cubic};
use crate::app::runtime::UiAssets;
use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy::ui::{BackgroundGradient, ColorStop, LinearGradient};
use leocard_ui::{TransitionVisuals, UiComposer, fade_panel};
use std::collections::VecDeque;

const WIDTH: f32 = 280.0;
const HEIGHT: f32 = 34.0;
const ENTRY_DURATION: f32 = 0.28;
const EXIT_DURATION: f32 = 0.36;

#[derive(Resource, Default)]
pub(crate) struct CornerNotifications(VecDeque<NoticeContent>);

enum NoticeContent {
    Coin(i64),
    Message(String),
}

impl CornerNotifications {
    pub(crate) fn coin(&mut self, delta: i64) {
        self.0.push_back(NoticeContent::Coin(delta));
    }

    pub(crate) fn message(&mut self, message: String) {
        self.0.push_back(NoticeContent::Message(message));
    }
}

#[derive(Component)]
pub(super) struct CornerNoticeLayer;

#[derive(Component)]
pub(super) struct CornerNotice {
    elapsed: f32,
    width: f32,
    hold: f32,
}

pub(super) fn setup_corner_notifications(mut commands: UiComposer) {
    // 独立于页面重建，所有页面复用；装饰层不参与鼠标拾取。
    commands.spawn((
        CornerNoticeLayer,
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            height: percent(100),
            overflow: Overflow::clip(),
            ..default()
        },
        GlobalZIndex(2600),
        Pickable::IGNORE,
    ));
}

#[expect(
    clippy::too_many_arguments,
    reason = "Bevy injects the independent notice and subtree visual queries"
)]
pub(super) fn animate_corner_notifications(
    mut commands: UiComposer,
    time: Res<Time>,
    assets: Res<UiAssets>,
    mut notices: ResMut<CornerNotifications>,
    layers: Query<Entity, With<CornerNoticeLayer>>,
    mut active: Query<(Entity, &mut CornerNotice, &mut UiTransform)>,
    children: Query<&Children>,
    mut visuals: TransitionVisuals,
) {
    for (entity, mut notice, mut transform) in &mut active {
        notice.elapsed += time.delta_secs();
        if notice.elapsed >= ENTRY_DURATION + notice.hold + EXIT_DURATION {
            commands.entity(entity).despawn();
            continue;
        }
        let entry = (notice.elapsed / ENTRY_DURATION).clamp(0.0, 1.0);
        let exit =
            ((notice.elapsed - ENTRY_DURATION - notice.hold) / EXIT_DURATION).clamp(0.0, 1.0);
        transform.translation = Val2::px(
            notice.width * (1.0 - ease_out_cubic(entry)),
            -HEIGHT * ease_out_cubic(exit),
        );
        fade_panel(entity, 1.0 - exit, &children, &mut visuals, &mut commands);
    }
    if active.is_empty()
        && let Ok(layer) = layers.single()
        && let Some(content) = notices.0.pop_front()
    {
        spawn_notice(&mut commands, layer, content, &assets);
    }
}

fn spawn_notice(
    commands: &mut UiComposer,
    layer: Entity,
    content: NoticeContent,
    assets: &UiAssets,
) {
    let (text, coin, width, hold) = match content {
        NoticeContent::Coin(delta) => (format!("金币 {delta:+}"), true, WIDTH, 1.5),
        NoticeContent::Message(text) => (text, false, 720.0, 4.5),
    };
    let notice = commands.child(
        layer,
        (
            CornerNotice {
                elapsed: 0.0,
                width,
                hold,
            },
            Node {
                position_type: PositionType::Absolute,
                right: px(0),
                top: px(82),
                width: px(width),
                max_width: percent(100),
                min_height: px(HEIGHT),
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
            UiTransform::from_translation(Val2::px(width, 0.0)),
            Pickable::IGNORE,
        ),
    );
    if coin {
        commands.child(
            notice,
            (
                Node {
                    width: px(22),
                    height: px(22),
                    flex_shrink: 0.0,
                    ..default()
                },
                ImageNode::new(assets.shop.coin.clone()),
                Pickable::IGNORE,
            ),
        );
    }
    let text = add_text(
        commands,
        notice,
        text,
        18.0,
        Color::srgb(0.82, 0.82, 0.82),
        assets,
    );
    commands.entity(text).insert(Pickable::IGNORE);
    if coin {
        commands
            .entity(text)
            .insert(TextLayout::default().with_no_wrap());
    } else {
        commands
            .entity(text)
            .insert(TextLayout::justify(Justify::Right));
        commands.entity(text).insert(Node {
            width: percent(100),
            min_width: px(0),
            flex_grow: 1.0,
            ..default()
        });
    }
}
