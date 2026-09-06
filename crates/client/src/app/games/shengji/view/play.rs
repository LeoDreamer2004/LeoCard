use super::*;

pub fn add_shengji_play_area(
    commands: &mut Commands,
    parent: Entity,
    game: &ShengjiSnapshot,
    player: PlayerId,
    side: SeatSide,
    assets: &UiAssets,
    previous_trick: Option<&[ShengjiPublicPlay]>,
) {
    let area = spawn_node(
        commands,
        parent,
        Node {
            width: px(165),
            min_width: px(165),
            min_height: px(104),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: match side {
                SeatSide::Left => JustifyContent::FlexStart,
                SeatSide::Top => JustifyContent::Center,
                SeatSide::Right => JustifyContent::FlexEnd,
            },
            ..default()
        },
        None,
    );
    let throw_failure = previous_trick.is_none().then_some(()).and_then(|_| {
        game.throw_failure
            .as_ref()
            .filter(|failure| failure.player == player)
    });
    let cards = if let Some(previous_trick) = previous_trick {
        previous_trick
            .iter()
            .find(|played| played.player == player)
            .map(|played| played.play.cards.as_slice())
    } else if matches!(
        game.phase,
        ShengjiPhaseView::Dealing { .. }
            | ShengjiPhaseView::BiddingGrace { .. }
            | ShengjiPhaseView::BottomCopyBurying { .. }
    ) {
        game.declaration
            .as_ref()
            .filter(|declaration| declaration.player == player)
            .map(|declaration| declaration.cards.as_slice())
    } else if let Some(failure) = throw_failure {
        Some(failure.attempted.as_slice())
    } else {
        game.trick.as_ref().and_then(|trick| {
            trick
                .plays
                .iter()
                .find(|played| played.player == player)
                .map(|played| played.play.cards.as_slice())
        })
    };
    if let Some(cards) = cards {
        if let Some(failure) = throw_failure {
            let holder = spawn_node(
                commands,
                area,
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    row_gap: px(2),
                    ..default()
                },
                None,
            );
            commands
                .entity(holder)
                .insert((UiTransform::IDENTITY, Visibility::Visible));
            let label = add_text(commands, holder, "甩牌失败", 15.0, DANGER, assets);
            commands.entity(label).insert((
                ShengjiFailedThrowLabel {
                    returning: failure.stage == ShengjiThrowFailureStage::Returning,
                    elapsed: 0.0,
                },
                TextShadow {
                    offset: Vec2::new(1.0, 1.0),
                    color: Color::BLACK.with_alpha(0.88),
                },
            ));
            let direction = if player == game.you {
                Vec2::new(0.0, 92.0)
            } else {
                match side {
                    SeatSide::Left => Vec2::new(-88.0, 0.0),
                    SeatSide::Top => Vec2::new(0.0, -82.0),
                    SeatSide::Right => Vec2::new(88.0, 0.0),
                }
            };
            add_shengji_failed_throw_card_row(
                commands,
                holder,
                cards,
                shengji_display_trump(game),
                failure.stage,
                direction,
                assets,
            );
        } else {
            add_shengji_card_row(
                commands,
                area,
                cards,
                ShengjiCardSize::Seat,
                shengji_display_trump(game),
                assets,
            );
        }
    }
}

pub fn add_shengji_own_play(
    commands: &mut Commands,
    table: Entity,
    game: &ShengjiSnapshot,
    assets: &UiAssets,
    previous_trick: Option<&[ShengjiPublicPlay]>,
) {
    let area = spawn_node(
        commands,
        table,
        Node {
            position_type: PositionType::Absolute,
            left: percent(35),
            right: percent(35),
            bottom: px(8),
            min_height: px(104),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        None,
    );
    add_shengji_play_area(
        commands,
        area,
        game,
        game.you,
        SeatSide::Top,
        assets,
        previous_trick,
    );
}

pub fn add_shengji_collecting_tray(
    commands: &mut Commands,
    table: Entity,
    game: &ShengjiSnapshot,
    cards: &[ShengjiCard],
    score_capture: &ShengjiScoreCaptureEffectState,
    assets: &UiAssets,
) {
    let tray = spawn_node(
        commands,
        table,
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            top: px(0),
            width: px(410),
            min_height: px(62),
            padding: UiRect::new(px(7), px(78), px(6), px(6)),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(8)),
            overflow: Overflow::clip(),
            ..default()
        },
        Some(Color::BLACK.with_alpha(0.30)),
    );
    commands
        .entity(tray)
        .insert(BorderColor::all(ACCENT.with_alpha(0.62)));
    let score = spawn_node(
        commands,
        tray,
        Node {
            position_type: PositionType::Absolute,
            right: px(5),
            top: px(4),
            bottom: px(4),
            width: px(68),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        None,
    );
    add_text(commands, score, "闲家得分", 10.0, MUTED, assets);
    let score_text = add_text(
        commands,
        score,
        displayed_shengji_captured_score(score_capture, shengji_tray_score(game)).to_string(),
        28.0,
        ACCENT,
        assets,
    );
    commands
        .entity(score_text)
        .insert((ShengjiScoreTrayAnchor, ShengjiCollectingScoreText));
    if game.throw_failure.as_ref().is_some_and(|failure| {
        failure.stage == ShengjiThrowFailureStage::Showing && failure.penalty_points > 0
    }) {
        commands
            .entity(score_text)
            .insert(ShengjiThrowPenaltyScorePulse { elapsed: 0.0 });
    }
    if cards.is_empty() {
        add_text(commands, tray, "闲家尚未获得分牌", 12.0, MUTED, assets);
    } else {
        add_shengji_card_row(
            commands,
            tray,
            cards,
            ShengjiCardSize::Score,
            shengji_display_trump(game),
            assets,
        );
    }
}

pub fn add_shengji_throw_penalty_effect(
    commands: &mut Commands,
    table: Entity,
    game: &ShengjiSnapshot,
    assets: &UiAssets,
) {
    let Some(failure) = game.throw_failure.as_ref().filter(|failure| {
        failure.stage == ShengjiThrowFailureStage::Showing && failure.penalty_points > 0
    }) else {
        return;
    };
    let own_seat = game
        .players
        .iter()
        .find(|player| player.id == game.you)
        .map_or(0, |player| player.seat.0);
    let offender_seat = game
        .players
        .iter()
        .find(|player| player.id == failure.player)
        .map_or(own_seat, |player| player.seat.0);
    let offender_anchor = match (offender_seat + 4 - own_seat) % 4 {
        0 => Vec2::new(50.0, 86.0),
        1 => Vec2::new(20.0, 52.0),
        2 => Vec2::new(50.0, 20.0),
        3 => Vec2::new(80.0, 52.0),
        _ => unreachable!(),
    };
    // 左上得分托盘固定宽 410px，数字位于其最右侧；设计宽度下约为 29%。
    let score_anchor = Vec2::new(29.0, 6.0);
    let dealer_side_penalty = game
        .dealer
        .is_some_and(|dealer| dealer.0 % 2 == failure.player.0 % 2);
    let (source, target, sign) = if dealer_side_penalty {
        (offender_anchor, score_anchor, "+")
    } else {
        (score_anchor, Vec2::new(-4.0, 6.0), "−")
    };
    let effect = add_text(
        commands,
        table,
        format!("{sign}{}", failure.penalty_points),
        24.0,
        Color::NONE,
        assets,
    );
    commands.entity(effect).insert((
        Node {
            position_type: PositionType::Absolute,
            left: percent(source.x),
            top: percent(source.y),
            width: px(60),
            height: px(36),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        ShengjiThrowPenaltyFloat {
            source,
            target,
            elapsed: 0.0,
        },
        UiTransform::IDENTITY,
        TextShadow {
            offset: Vec2::new(1.0, 1.0),
            color: Color::BLACK.with_alpha(0.9),
        },
        GlobalZIndex(840),
        FocusPolicy::Pass,
    ));
}

fn shengji_tray_score(game: &ShengjiSnapshot) -> u32 {
    match &game.phase {
        ShengjiPhaseView::Finished { result, .. } => i64::from(result.trick_points)
            .saturating_add(i64::from(result.penalty_adjustment))
            .max(0) as u32,
        _ => game.collecting_score,
    }
}
