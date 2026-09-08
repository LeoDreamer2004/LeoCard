use super::*;
use leocard_protocol::MahjongHandResultView;
use leocard_protocol::MahjongSnapshot;

pub(super) fn render_mahjong_settlement(
    commands: &mut Commands,
    table: Entity,
    game: &MahjongSnapshot,
    result: &MahjongHandResultView,
    assets: &UiAssets,
    avatars: &AvatarImages,
    animation: &GameSummaryAnimation,
) {
    const FAN_INTERVAL: f32 = 0.14;

    let modal_visual = summary_modal_visual(animation.elapsed);
    let modal = spawn_node(
        commands,
        table,
        Node {
            position_type: PositionType::Absolute,
            left: percent(22),
            right: percent(22),
            top: percent(2),
            min_height: px(390),
            padding: UiRect::all(px(24)),
            flex_direction: FlexDirection::Column,
            row_gap: px(6),
            border_radius: BorderRadius::all(px(12)),
            ..default()
        },
        None,
    );
    commands.entity(modal).insert((
        GameSummaryModal,
        UiTransform::from_translation(Val2::px(0.0, modal_visual.offset_y)),
        GlobalZIndex(1200),
        FocusPolicy::Block,
        if animation.elapsed >= 0.0 {
            Visibility::Visible
        } else {
            Visibility::Hidden
        },
    ));
    let texture = decorate_panel_skin(commands, modal, PanelSkin::Window, assets);
    commands.entity(texture).insert(GameSummaryPanelTexture);
    add_animated_summary_text(
        commands,
        modal,
        "本局结算",
        26.0,
        ACCENT,
        0.0,
        animation.elapsed,
        assets,
    );
    if result.exhaustive_draw {
        add_animated_summary_text(
            commands,
            modal,
            "本局荒牌",
            14.0,
            MUTED,
            0.0,
            animation.elapsed,
            assets,
        );
    }

    let mut next_delay = SUMMARY_ROW_START_DELAY;
    for winner in &result.winners {
        let name = game
            .players
            .iter()
            .find(|player| player.id == winner.player)
            .map(|player| player.name.as_str())
            .unwrap_or("玩家");
        let outcome = winner.from.map_or_else(
            || format!("{name} 自摸 {}番", winner.score.total_points),
            |from| {
                let source = game
                    .players
                    .iter()
                    .find(|player| player.id == from)
                    .map(|player| player.name.as_str())
                    .unwrap_or("玩家");
                format!("{source} 放炮给 {name} {}番", winner.score.total_points)
            },
        );
        add_animated_summary_text(
            commands,
            modal,
            outcome,
            17.0,
            READY,
            next_delay,
            animation.elapsed,
            assets,
        );
        next_delay += FAN_INTERVAL;
        let fans = spawn_node(
            commands,
            modal,
            Node {
                width: percent(100),
                min_height: px(30),
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::Wrap,
                align_items: AlignItems::Center,
                column_gap: px(6),
                row_gap: px(6),
                ..default()
            },
            None,
        );
        for fan in &winner.score.fans {
            let delay = next_delay;
            let progress = summary_row_progress(animation.elapsed, delay);
            let color = if fan.points >= 48 {
                Color::srgb(0.98, 0.76, 0.25)
            } else if fan.points >= 6 {
                READY
            } else {
                TEXT
            };
            let badge = spawn_node(
                commands,
                fans,
                Node {
                    min_height: px(28),
                    padding: UiRect::axes(px(9), px(4)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    border: UiRect::all(px(1)),
                    border_radius: BorderRadius::all(px(6)),
                    ..default()
                },
                Some(PANEL_ALT.with_alpha(0.82 * progress)),
            );
            commands.entity(badge).insert((
                GameSummaryRow { delay },
                BorderColor::all(color.with_alpha(0.72)),
                UiTransform::from_translation(Val2::px(0.0, 12.0 * (1.0 - progress))),
                if progress > 0.0 {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                },
            ));
            let fan_name = if fan.count > 1 {
                format!("{}×{}", fan.fan.name(), fan.count)
            } else {
                fan.fan.name().to_owned()
            };
            let text = add_animated_summary_text(
                commands,
                badge,
                format!("{fan_name}  {}番", fan.points),
                if fan.points >= 48 { 14.5 } else { 13.0 },
                color,
                delay,
                animation.elapsed,
                assets,
            );
            if fan.points >= 48 {
                commands.entity(text).insert(TextShadow {
                    offset: Vec2::new(0.8, 0.0),
                    color: color.with_alpha(0.82),
                });
            }
            next_delay += FAN_INTERVAL;
        }
        next_delay += 0.08;
    }

    let list = spawn_node(
        commands,
        modal,
        Node {
            width: percent(100),
            flex_direction: FlexDirection::Column,
            row_gap: px(4),
            ..default()
        },
        None,
    );
    let mut players = game.players.iter().collect::<Vec<_>>();
    players.sort_by(|left, right| {
        result.match_scores[right.id.0 as usize]
            .cmp(&result.match_scores[left.id.0 as usize])
            .then_with(|| left.seat.0.cmp(&right.seat.0))
    });
    for (index, player) in players.iter().enumerate() {
        let delay = next_delay + index as f32 * SUMMARY_ROW_INTERVAL;
        let progress = summary_row_progress(animation.elapsed, delay);
        let row = spawn_node(
            commands,
            list,
            Node {
                width: percent(100),
                min_height: px(38),
                padding: UiRect::axes(px(10), px(4)),
                align_items: AlignItems::Center,
                column_gap: px(10),
                border_radius: BorderRadius::all(px(7)),
                ..default()
            },
            Some(PANEL_ALT.with_alpha(0.82 * progress)),
        );
        commands.entity(row).insert((
            GameSummaryRow { delay },
            UiTransform::from_translation(Val2::px(0.0, 12.0 * (1.0 - progress))),
            if progress > 0.0 {
                Visibility::Visible
            } else {
                Visibility::Hidden
            },
        ));
        let avatar = player.avatar.and_then(|id| avatars.remote.get(&id));
        add_ready_avatar(
            commands,
            row,
            &player.name,
            avatar,
            28.0,
            player.ready,
            assets,
        );
        let name = spawn_node(
            commands,
            row,
            Node {
                flex_grow: 1.0,
                min_width: px(0),
                ..default()
            },
            None,
        );
        add_animated_summary_text(
            commands,
            name,
            &player.name,
            16.0,
            TEXT,
            delay,
            animation.elapsed,
            assets,
        );
        let delta = result.deltas[player.id.0 as usize];
        let total = result.match_scores[player.id.0 as usize];
        let total_text = add_animated_summary_text(
            commands,
            row,
            format!("累计 {total:+}"),
            16.0,
            ACCENT,
            delay,
            animation.elapsed,
            assets,
        );
        commands
            .entity(total_text)
            .insert(AnimatedSignedSummaryScore {
                target: total,
                delay,
            });
        add_animated_summary_text(
            commands,
            row,
            format!("{delta:+}"),
            16.0,
            if delta > 0 {
                READY
            } else if delta < 0 {
                DANGER
            } else {
                TEXT
            },
            delay,
            animation.elapsed,
            assets,
        );
    }
    let actions_delay =
        next_delay + players.len() as f32 * SUMMARY_ROW_INTERVAL + SUMMARY_ACTIONS_EXTRA_DELAY;
    let actions = spawn_node(
        commands,
        modal,
        Node {
            width: percent(100),
            min_height: px(48),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            column_gap: px(10),
            ..default()
        },
        None,
    );
    commands.entity(actions).insert((
        GameSummaryActions {
            delay: actions_delay,
        },
        if animation.elapsed >= actions_delay {
            Visibility::Visible
        } else {
            Visibility::Hidden
        },
    ));
    add_action_button(
        commands,
        actions,
        "退出游戏",
        UiAction::Lobby(LobbyUiAction::LeaveRoom),
        ButtonKind::Pass,
        assets,
    );
    if game.you == game.host {
        add_action_button(
            commands,
            actions,
            "返回房间",
            UiAction::Lobby(LobbyUiAction::ReturnToLobby),
            ButtonKind::Warning,
            assets,
        );
    }
    let ready = game
        .players
        .iter()
        .find(|player| player.id == game.you)
        .is_some_and(|player| player.ready);
    if ready {
        add_disabled_action_button(commands, actions, "已准备", assets);
    } else {
        add_action_button(
            commands,
            actions,
            "再来一局",
            UiAction::Lobby(LobbyUiAction::PlayAgain),
            ButtonKind::Primary,
            assets,
        );
    }
}
