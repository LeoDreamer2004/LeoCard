use super::*;
use leocard_protocol::ShengjiHandResultView;
use leocard_protocol::ShengjiSnapshot;
use leocard_shengji::ShengjiCard;

const SHENGJI_KITTY_SCORE_DELAY: f32 = 0.72;
const SHENGJI_TOTAL_LABEL_DELAY: f32 = 1.92;

pub fn add_shengji_result(
    commands: &mut Commands,
    table: Entity,
    game: &ShengjiSnapshot,
    result: &ShengjiHandResultView,
    buried: &[ShengjiCard],
    assets: &UiAssets,
    avatars: &AvatarImages,
    animation: &ShengjiSettlementAnimation,
) {
    let kitty_award = u32::from(result.kitty_points).saturating_mul(result.kitty_multiplier);
    let stage = spawn_node(
        commands,
        table,
        Node {
            position_type: PositionType::Absolute,
            // 居中且保持足够紧凑，左边缘不会压住左上角的闲家得分框。
            left: percent(30),
            right: percent(30),
            top: percent(3),
            height: percent(32),
            min_height: px(205),
            padding: UiRect::all(px(20)),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: px(5),
            border_radius: BorderRadius::all(px(12)),
            ..default()
        },
        Some(HEADER_BG.with_alpha(0.94)),
    );
    commands.entity(stage).insert((
        BorderColor::all(Color::NONE),
        BoxShadow::new(Color::BLACK.with_alpha(0.48), px(2), px(7), px(0), px(12)),
        GlobalZIndex(1120),
        FocusPolicy::Pass,
    ));
    decorate_panel_skin(commands, stage, PanelSkin::Section, assets);

    let kitty_title = add_text(commands, stage, "底牌", 17.0, ACCENT, assets);
    commands
        .entity(kitty_title)
        .insert(ShengjiTimedReveal { delay: 0.0 });
    let card_row = spawn_node(
        commands,
        stage,
        Node {
            height: px(70),
            min_width: px(390),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            column_gap: px(5),
            ..default()
        },
        None,
    );
    let trump = shengji_display_trump(game);
    for (index, card) in buried.iter().copied().enumerate() {
        let entity = commands
            .spawn((
                Node {
                    width: px(45),
                    height: px(62),
                    border_radius: BorderRadius::all(px(4)),
                    ..default()
                },
                ImageNode::new(shengji_card_face(card, assets)).with_color(Color::NONE),
                UiTransform {
                    translation: Val2::px(0.0, 16.0),
                    scale: Vec2::splat(0.78),
                    ..default()
                },
                Visibility::Hidden,
                ShengjiKittyRevealCard { index },
            ))
            .id();
        commands.entity(card_row).add_child(entity);
        add_shengji_trump_stars(
            commands,
            entity,
            card,
            trump,
            ShengjiCardSize::Score,
            assets,
        );
    }

    let score_summary = spawn_node(
        commands,
        stage,
        Node {
            width: percent(100),
            height: px(44),
            padding: UiRect::horizontal(px(16)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            column_gap: px(20),
            ..default()
        },
        None,
    );
    let kitty_score_line = spawn_node(
        commands,
        score_summary,
        Node {
            height: px(40),
            flex_grow: 1.0,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::FlexStart,
            column_gap: px(8),
            ..default()
        },
        None,
    );
    commands.entity(kitty_score_line).insert((
        ShengjiTimedReveal {
            delay: SHENGJI_KITTY_SCORE_DELAY,
        },
        Visibility::Hidden,
    ));
    add_text(commands, kitty_score_line, "底牌分数", 14.0, MUTED, assets);
    let kitty_score = add_text(
        commands,
        kitty_score_line,
        result.kitty_points.to_string(),
        28.0,
        ACCENT,
        assets,
    );
    commands.entity(kitty_score).insert((
        ShengjiKittyScoreAnchor,
        ShengjiKittyScoreText {
            base: u32::from(result.kitty_points),
            awarded: kitty_award,
        },
        UiTransform::IDENTITY,
    ));
    if result.kitty_multiplier > 0 {
        let multiplier = add_text(
            commands,
            kitty_score_line,
            format!("×{}", result.kitty_multiplier),
            25.0,
            READY,
            assets,
        );
        commands.entity(multiplier).insert((
            ShengjiKittyMultiplier,
            UiTransform {
                translation: Val2::px(42.0, -24.0),
                scale: Vec2::splat(0.82),
                ..default()
            },
            Visibility::Hidden,
        ));
    } else {
        add_text(commands, kitty_score_line, "庄家守底", 13.0, MUTED, assets);
    }

    let total_box = spawn_node(
        commands,
        score_summary,
        Node {
            min_width: px(170),
            height: px(40),
            flex_grow: 1.0,
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::FlexEnd,
            column_gap: px(8),
            ..default()
        },
        None,
    );
    commands.entity(total_box).insert((
        ShengjiTimedReveal {
            delay: SHENGJI_TOTAL_LABEL_DELAY,
        },
        Visibility::Hidden,
    ));
    add_text(commands, total_box, "闲家总得分", 13.0, MUTED, assets);
    let total_text = add_text(commands, total_box, "0", 28.0, ACCENT, assets);
    commands.entity(total_text).insert((
        ShengjiSettlementTotalAnchor,
        ShengjiSettlementTotalText {
            target: result.collecting_score,
        },
        UiTransform::IDENTITY,
    ));

    add_shengji_settlement_modal(commands, table, game, result, assets, avatars, animation);
}

fn add_shengji_settlement_modal(
    commands: &mut Commands,
    table: Entity,
    game: &ShengjiSnapshot,
    result: &ShengjiHandResultView,
    assets: &UiAssets,
    avatars: &AvatarImages,
    animation: &ShengjiSettlementAnimation,
) {
    let modal = spawn_node(
        commands,
        table,
        Node {
            position_type: PositionType::Absolute,
            left: percent(25),
            right: percent(25),
            top: percent(37),
            height: percent(50),
            min_height: px(300),
            padding: UiRect::all(px(24)),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: px(5),
            border_radius: BorderRadius::all(px(12)),
            ..default()
        },
        Some(Color::NONE),
    );
    commands.entity(modal).insert((
        ShengjiSettlementModal,
        UiTransform {
            translation: Val2::px(0.0, 72.0),
            ..default()
        },
        GlobalZIndex(1220),
        FocusPolicy::Block,
        Visibility::Hidden,
    ));
    let texture = decorate_panel_skin(commands, modal, PanelSkin::Window, assets);
    commands
        .entity(texture)
        .insert(ShengjiSettlementPanelTexture);
    add_text(commands, modal, "本局结算", 24.0, ACCENT, assets);
    let outcome = add_text(
        commands,
        modal,
        shengji_settlement_outcome(result, game.rules.deck_count),
        20.0,
        TEXT,
        assets,
    );
    commands
        .entity(outcome)
        .insert(ShengjiSettlementOutcomeText);

    let list = spawn_node(
        commands,
        modal,
        Node {
            width: percent(100),
            flex_grow: 1.0,
            flex_direction: FlexDirection::Column,
            row_gap: px(4),
            ..default()
        },
        None,
    );
    let mut players = game.players.iter().collect::<Vec<_>>();
    players.sort_by_key(|player| player.seat.0);
    for (index, player) in players.iter().enumerate() {
        let delay =
            SHENGJI_SETTLEMENT_MODAL_DELAY + 0.46 + index as f32 * SHENGJI_SETTLEMENT_ROW_INTERVAL;
        let row = spawn_node(
            commands,
            list,
            Node {
                width: percent(100),
                min_height: px(38),
                padding: UiRect::axes(px(9), px(4)),
                align_items: AlignItems::Center,
                column_gap: px(9),
                border_radius: BorderRadius::all(px(7)),
                ..default()
            },
            Some(Color::NONE),
        );
        commands.entity(row).insert((
            ShengjiSettlementRow { delay },
            UiTransform {
                translation: Val2::px(0.0, 16.0),
                ..default()
            },
            Visibility::Hidden,
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
                flex_direction: FlexDirection::Column,
                ..default()
            },
            None,
        );
        add_text(commands, name, &player.name, 15.0, TEXT, assets);
        add_text(
            commands,
            name,
            format!("平台积分 {}", player.reference_points),
            10.0,
            MUTED,
            assets,
        );
        let delta = result
            .reference_changes
            .iter()
            .find(|change| change.player == player.id)
            .map_or(0, |change| change.delta);
        add_text(
            commands,
            row,
            format!("{delta:+}"),
            20.0,
            if delta >= 0 { READY } else { DANGER },
            assets,
        );
    }

    let actions_delay = SHENGJI_SETTLEMENT_MODAL_DELAY
        + 0.46
        + players.len() as f32 * SHENGJI_SETTLEMENT_ROW_INTERVAL
        + 0.26;
    let actions = spawn_node(
        commands,
        modal,
        Node {
            width: percent(100),
            min_height: px(47),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            column_gap: px(8),
            ..default()
        },
        None,
    );
    commands.entity(actions).insert((
        ShengjiSettlementActions {
            delay: actions_delay,
        },
        Visibility::Hidden,
    ));
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
            "准备下一局",
            UiAction::Lobby(LobbyUiAction::PlayAgain),
            ButtonKind::Primary,
            assets,
        );
    }
    if game.you == game.host {
        add_action_button(
            commands,
            actions,
            "结束并返回大厅",
            UiAction::Lobby(LobbyUiAction::ReturnToLobby),
            ButtonKind::Secondary,
            assets,
        );
    }
    let _ = animation;
}

fn shengji_settlement_outcome(result: &ShengjiHandResultView, deck_count: u8) -> String {
    shengji_settlement_outcome_for_score(result.collecting_score, result.promoted_steps, deck_count)
}

pub fn shengji_settlement_outcome_for_score(
    collecting_score: u32,
    promoted_steps: u8,
    deck_count: u8,
) -> String {
    let (small_light, takeover) = match deck_count {
        4 => (80, 160),
        3 => (60, 120),
        _ => (40, 80),
    };
    if collecting_score == 0 {
        "闲家大光".to_owned()
    } else if collecting_score < small_light {
        "闲家小光".to_owned()
    } else if collecting_score < takeover {
        "闲家脱贫".to_owned()
    } else if promoted_steps == 0 {
        "闲家上台".to_owned()
    } else {
        format!("闲家升{promoted_steps}级")
    }
}
