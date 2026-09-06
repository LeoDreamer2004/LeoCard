use super::*;
use leocard_protocol::QiGui523Snapshot;

pub fn turn_clock_visible(game: &QiGui523Snapshot, player: PlayerId) -> bool {
    matches!(&game.phase, GamePhaseView::Playing)
        && game
            .trick
            .as_ref()
            .is_some_and(|trick| trick.current_player == player)
        && game
            .players
            .iter()
            .find(|state| state.id == player)
            .is_some_and(|state| !state.auto_play)
}

pub(super) fn add_turn_clock(
    commands: &mut Commands,
    parent: Entity,
    timer: Option<TurnTimerView>,
    assets: &UiAssets,
) {
    let clock = commands
        .spawn((
            TurnClock,
            Node {
                width: px(40),
                height: px(40),
                margin: UiRect::right(px(12)),
                border: UiRect::all(px(2)),
                border_radius: BorderRadius::all(percent(50)),
                position_type: PositionType::Relative,
                ..default()
            },
            BackgroundColor(HEADER_BG),
            BorderColor::all(ACCENT),
            UiTransform::IDENTITY,
        ))
        .id();
    commands.entity(parent).add_child(clock);

    for (left, rotation) in [(3.0, -0.25), (27.0, 0.25)] {
        let bell = commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: px(left),
                    top: px(-4),
                    width: px(10),
                    height: px(5),
                    border_radius: BorderRadius::all(px(3)),
                    ..default()
                },
                BackgroundColor(ACCENT),
                UiTransform::from_rotation(Rot2::radians(rotation)),
            ))
            .id();
        commands.entity(clock).add_child(bell);
    }

    let hand = commands
        .spawn((
            TurnClockHand,
            Node {
                position_type: PositionType::Absolute,
                left: px(18),
                top: px(8),
                width: px(2),
                height: px(22),
                border_radius: BorderRadius::all(px(1)),
                ..default()
            },
            BackgroundColor(ACCENT),
            UiTransform::IDENTITY,
        ))
        .id();
    commands.entity(clock).add_child(hand);
    let center = spawn_node(
        commands,
        clock,
        Node {
            position_type: PositionType::Absolute,
            left: px(16),
            top: px(16),
            width: px(6),
            height: px(6),
            border_radius: BorderRadius::all(percent(50)),
            ..default()
        },
        Some(TEXT),
    );
    commands.entity(center).insert(ZIndex(2));
    let label = turn_timer_label(timer);
    let label = add_text(commands, parent, label, 22.0, ACCENT, assets);
    commands.entity(label).insert(TurnClockLabel);
}

pub fn turn_timer_label(timer: Option<TurnTimerView>) -> String {
    match timer {
        Some(timer) if timer.base_seconds > 0 => timer.base_seconds.to_string(),
        Some(timer) => format!("烧条中... {}", timer.reserve_seconds),
        None => String::new(),
    }
}
