use super::{
    TEXAS_SHOWDOWN_REVEAL_DURATION, TexasShowdownBackdrop, TexasShowdownBestCard,
    TexasShowdownRevealRoot, TexasShowdownTitle, TexasShowdownTitleText, TexasShowdownUnderline,
    add_texas_card, texas_card_face, texas_category_label, texas_player_chip_zone,
};
use crate::app::presentation::add_animated_summary_text;
use crate::app::presentation::{
    ACCENT, ButtonKind, DANGER, GameSummaryActions, GameSummaryAnimation, GameSummaryDivider,
    GameSummaryModal, GameSummaryPanelTexture, GameSummaryRow, MUTED, PANEL_ALT, PanelSkin, READY,
    SUMMARY_ACTIONS_EXTRA_DELAY, SUMMARY_ROW_INTERVAL, SUMMARY_ROW_START_DELAY, SummaryDescriptor,
    TEXAS_UNCONTESTED_REVEAL_DURATION, TEXT, add_action_button, add_avatar,
    add_disabled_action_button, add_ready_avatar, add_text, decorate_panel_skin, spawn_node,
    summary_modal_visual, summary_row_progress,
};
use crate::app::runtime::{AvatarImages, UiAssets};
use crate::app::shell::{LobbyUiAction, UiAction};
use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use leocard_protocol::{
    SeatId, TABLE_SEAT_COUNT, TexasHoldemPhaseView, TexasHoldemPlayerState, TexasHoldemSnapshot,
};

pub(crate) fn texas_holdem_summary_descriptor(
    game: &TexasHoldemSnapshot,
) -> Option<SummaryDescriptor> {
    let TexasHoldemPhaseView::HandComplete {
        showdown,
        tournament_complete,
        reference_changes,
        ..
    } = &game.phase
    else {
        return None;
    };
    let own = game.players.iter().find(|player| player.id == game.you)?;
    let nonnegative_outcome = if *tournament_complete {
        reference_changes
            .iter()
            .find(|change| change.player == game.you)
            .is_none_or(|change| change.delta >= 0)
    } else {
        own.stack >= own.hand_start_stack
    };
    Some(SummaryDescriptor {
        match_id: game.match_id,
        texas_hand_number: Some(game.hand_number),
        settlement_index: None,
        entry_count: game.players.len() * usize::from(*tournament_complete) + game.players.len(),
        nonnegative_outcome,
        reveal_duration: if *showdown {
            TEXAS_SHOWDOWN_REVEAL_DURATION
        } else {
            TEXAS_UNCONTESTED_REVEAL_DURATION
        },
    })
}

pub(super) fn add_texas_showdown_reveal(
    commands: &mut Commands,
    table: Entity,
    game: &TexasHoldemSnapshot,
    own_seat: SeatId,
    assets: &UiAssets,
    animation: &GameSummaryAnimation,
) {
    let TexasHoldemPhaseView::HandComplete {
        showdown: true,
        awards,
        ..
    } = &game.phase
    else {
        return;
    };
    if animation.elapsed >= 0.0 {
        return;
    }
    let Some(main_award) = awards.first() else {
        return;
    };
    let Some(winner_id) = main_award.winners.first().copied() else {
        return;
    };
    let Some(winner) = game.players.iter().find(|player| player.id == winner_id) else {
        return;
    };
    let Some(revealed) = game
        .revealed_hands
        .iter()
        .find(|hand| hand.player == winner_id)
    else {
        return;
    };
    let Some(best) = revealed.best else {
        return;
    };

    let root = spawn_node(
        commands,
        table,
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            right: px(0),
            top: px(0),
            bottom: px(0),
            ..default()
        },
        None,
    );
    commands.entity(root).insert((
        TexasShowdownRevealRoot,
        GlobalZIndex(1080),
        FocusPolicy::Pass,
    ));
    let backdrop = spawn_node(
        commands,
        root,
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            right: px(0),
            top: px(0),
            bottom: px(0),
            ..default()
        },
        Some(Color::BLACK.with_alpha(0.0)),
    );
    commands
        .entity(backdrop)
        .insert((TexasShowdownBackdrop, FocusPolicy::Pass));

    let relative = (winner.seat.0 + TABLE_SEAT_COUNT - own_seat.0) % TABLE_SEAT_COUNT;
    let winner_zone = texas_player_chip_zone(relative);
    let hole_spacing = if revealed.cards.len() > 2 { 18.0 } else { 31.0 };
    let first_hole_offset = -hole_spacing * (revealed.cards.len().saturating_sub(1) as f32) / 2.0;
    for (index, card) in best.cards().into_iter().enumerate() {
        let hole_index = revealed.cards.iter().position(|hole| *hole == card);
        let (source, delay, start_scale) = if let Some(hole_index) = hole_index {
            (
                Vec2::new(
                    winner_zone.left
                        + winner_zone.width * 0.5
                        + first_hole_offset
                        + hole_index as f32 * hole_spacing,
                    winner_zone.top + 34.0,
                ),
                0.12 + hole_index as f32 * 0.10,
                0.55,
            )
        } else {
            let board_index = game
                .community
                .iter()
                .position(|community| *community == card)
                .unwrap_or(index);
            (
                Vec2::new(520.0 + board_index as f32 * 63.0, 213.0),
                0.68 + board_index as f32 * 0.055,
                0.80,
            )
        };
        let target = Vec2::new(444.0 + index as f32 * 78.0, 238.0);
        let entity = commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: px(target.x),
                    top: px(target.y),
                    width: px(70),
                    height: px(96),
                    border: UiRect::all(px(1)),
                    border_radius: BorderRadius::all(px(6)),
                    ..default()
                },
                ImageNode::new(texas_card_face(card, assets))
                    .with_color(Color::WHITE.with_alpha(0.0)),
                BorderColor::all(Color::srgb(0.88, 0.75, 0.35).with_alpha(0.0)),
                UiTransform {
                    translation: Val2::px(source.x - target.x, source.y - target.y),
                    scale: Vec2::splat(start_scale),
                    ..UiTransform::IDENTITY
                },
                TexasShowdownBestCard {
                    source,
                    target,
                    delay,
                    start_scale,
                },
                ZIndex(index as i32 + 5),
                FocusPolicy::Pass,
            ))
            .id();
        commands.entity(root).add_child(entity);
    }

    let title_area = spawn_node(
        commands,
        root,
        Node {
            position_type: PositionType::Absolute,
            left: px(360),
            top: px(350),
            width: px(560),
            height: px(42),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        None,
    );
    commands.entity(title_area).insert((
        TexasShowdownTitle,
        UiTransform::from_translation(Val2::px(0.0, 16.0)),
        FocusPolicy::Pass,
    ));
    let winner_names = main_award
        .winners
        .iter()
        .filter_map(|id| game.players.iter().find(|player| player.id == *id))
        .map(|player| player.name.as_str())
        .collect::<Vec<_>>()
        .join("、");
    let title = add_text(
        commands,
        title_area,
        format!("{winner_names} · {}", texas_category_label(best.category())),
        30.0,
        Color::NONE,
        assets,
    );
    commands.entity(title).insert(TextShadow {
        offset: Vec2::new(1.2, 1.8),
        color: Color::BLACK.with_alpha(0.72),
    });
    commands.entity(title).insert(TexasShowdownTitleText);
    let underline = spawn_node(
        commands,
        root,
        Node {
            position_type: PositionType::Absolute,
            left: px(470),
            top: px(394),
            width: px(340),
            height: px(2),
            border_radius: BorderRadius::all(px(2)),
            ..default()
        },
        Some(Color::srgb(0.90, 0.72, 0.27).with_alpha(0.0)),
    );
    commands.entity(underline).insert((
        TexasShowdownUnderline,
        UiTransform {
            scale: Vec2::new(0.0, 1.0),
            ..UiTransform::IDENTITY
        },
        FocusPolicy::Pass,
    ));
}

pub(super) fn add_texas_hand_result(
    commands: &mut Commands,
    table: Entity,
    game: &TexasHoldemSnapshot,
    assets: &UiAssets,
    avatars: &AvatarImages,
    animation: &GameSummaryAnimation,
) {
    let TexasHoldemPhaseView::HandComplete {
        showdown,
        awards,
        tournament_complete,
        reference_changes,
        ..
    } = &game.phase
    else {
        return;
    };
    let modal_visual = summary_modal_visual(animation.elapsed);
    let panel = spawn_node(
        commands,
        table,
        Node {
            position_type: PositionType::Absolute,
            left: percent(12),
            right: percent(12),
            top: percent(2),
            padding: UiRect::all(px(24)),
            flex_direction: FlexDirection::Column,
            row_gap: px(4),
            border_radius: BorderRadius::all(px(12)),
            ..default()
        },
        None,
    );
    commands.entity(panel).insert((
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
    let texture = decorate_panel_skin(commands, panel, PanelSkin::Window, assets);
    commands.entity(texture).insert(GameSummaryPanelTexture);
    add_animated_summary_text(
        commands,
        panel,
        "本手结算",
        25.0,
        ACCENT,
        0.0,
        animation.elapsed,
        assets,
    );

    let mut hand_players = game.players.iter().collect::<Vec<_>>();
    hand_players.sort_by(|left, right| {
        right
            .stack
            .cmp(&left.stack)
            .then_with(|| left.seat.0.cmp(&right.seat.0))
    });
    let hand_list = spawn_node(
        commands,
        panel,
        Node {
            width: percent(100),
            flex_direction: FlexDirection::Column,
            row_gap: px(3),
            ..default()
        },
        None,
    );
    for (index, player) in hand_players.iter().enumerate() {
        let delay = SUMMARY_ROW_START_DELAY + index as f32 * SUMMARY_ROW_INTERVAL;
        let row_progress = summary_row_progress(animation.elapsed, delay);
        let row = spawn_node(
            commands,
            hand_list,
            Node {
                width: percent(100),
                height: px(47),
                padding: UiRect::axes(px(9), px(3)),
                align_items: AlignItems::Center,
                column_gap: px(8),
                border_radius: BorderRadius::all(px(7)),
                ..default()
            },
            Some(PANEL_ALT.with_alpha(0.82 * row_progress)),
        );
        commands.entity(row).insert((
            GameSummaryRow { delay },
            UiTransform::from_translation(Val2::px(0.0, 12.0 * (1.0 - row_progress))),
            if row_progress > 0.0 {
                Visibility::Visible
            } else {
                Visibility::Hidden
            },
        ));
        add_texas_summary_avatar(commands, row, player, avatars, assets);
        let name = add_animated_summary_text(
            commands,
            row,
            &player.name,
            15.5,
            TEXT,
            delay,
            animation.elapsed,
            assets,
        );
        commands.entity(name).insert((
            Node {
                width: px(120),
                min_width: px(120),
                max_width: px(120),
                flex_shrink: 0.0,
                overflow: Overflow::clip(),
                ..default()
            },
            TextLayout::no_wrap(),
        ));
        let result = spawn_node(
            commands,
            row,
            Node {
                flex_grow: 1.0,
                min_width: px(0),
                align_items: AlignItems::Center,
                column_gap: px(7),
                ..default()
            },
            None,
        );
        let won_early = !*showdown
            && awards
                .iter()
                .any(|award| award.winners.contains(&player.id));
        if !*showdown {
            add_animated_summary_text(
                commands,
                result,
                if won_early { "胜利" } else { "弃牌" },
                18.0,
                if won_early { ACCENT } else { MUTED },
                delay,
                animation.elapsed,
                assets,
            );
        } else if player.folded {
            add_animated_summary_text(
                commands,
                result,
                "弃牌",
                18.0,
                MUTED,
                delay,
                animation.elapsed,
                assets,
            );
        } else if let Some(best) = game
            .revealed_hands
            .iter()
            .find(|revealed| revealed.player == player.id)
            .and_then(|revealed| revealed.best)
        {
            add_animated_summary_text(
                commands,
                result,
                texas_category_label(best.category()),
                14.0,
                READY,
                delay,
                animation.elapsed,
                assets,
            );
            for card in best.cards() {
                add_texas_card(commands, result, card, (28.0, 40.0), None, assets);
            }
        }
        let delta = i64::from(player.stack) - i64::from(player.hand_start_stack);
        add_animated_summary_text(
            commands,
            row,
            format!("{delta:+}"),
            19.0,
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

    let mut row_count = hand_players.len();
    if *tournament_complete {
        let divider_delay = SUMMARY_ROW_START_DELAY + row_count as f32 * SUMMARY_ROW_INTERVAL;
        let divider_progress = summary_row_progress(animation.elapsed, divider_delay);
        let divider = spawn_node(
            commands,
            panel,
            Node {
                width: percent(96),
                height: px(1),
                margin: UiRect::vertical(px(1)),
                align_self: AlignSelf::Center,
                ..default()
            },
            Some(Color::srgb(0.52, 0.55, 0.54).with_alpha(0.28 * divider_progress)),
        );
        commands.entity(divider).insert((
            GameSummaryDivider {
                delay: divider_delay,
            },
            if divider_progress > 0.0 {
                Visibility::Visible
            } else {
                Visibility::Hidden
            },
        ));
        let mut standings = hand_players.clone();
        standings.sort_by(|left, right| {
            right
                .stack
                .cmp(&left.stack)
                .then_with(|| left.seat.0.cmp(&right.seat.0))
        });
        for (index, player) in standings.iter().enumerate() {
            let delay = divider_delay + index as f32 * SUMMARY_ROW_INTERVAL;
            let progress = summary_row_progress(animation.elapsed, delay);
            let row = spawn_node(
                commands,
                panel,
                Node {
                    width: percent(100),
                    height: px(35),
                    padding: UiRect::axes(px(10), px(3)),
                    align_items: AlignItems::Center,
                    column_gap: px(8),
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
            add_avatar(commands, row, &player.name, avatar, 26.0, assets);
            add_animated_summary_text(
                commands,
                row,
                format!("{}. {}", index + 1, player.name),
                14.5,
                TEXT,
                delay,
                animation.elapsed,
                assets,
            );
            let spacer = spawn_node(
                commands,
                row,
                Node {
                    flex_grow: 1.0,
                    ..default()
                },
                None,
            );
            commands.entity(spacer).insert(FocusPolicy::Pass);
            add_animated_summary_text(
                commands,
                row,
                format!("筹码 {}", player.stack),
                16.0,
                ACCENT,
                delay,
                animation.elapsed,
                assets,
            );
            if let Some(change) = reference_changes
                .iter()
                .find(|change| change.player == player.id)
            {
                add_animated_summary_text(
                    commands,
                    row,
                    format!("{:+}", change.delta),
                    15.0,
                    if change.delta >= 0 { READY } else { DANGER },
                    delay,
                    animation.elapsed,
                    assets,
                );
            }
        }
        row_count += standings.len();
    }

    let actions_delay = SUMMARY_ROW_START_DELAY
        + row_count as f32 * SUMMARY_ROW_INTERVAL
        + SUMMARY_ACTIONS_EXTRA_DELAY;
    let controls = spawn_node(
        commands,
        panel,
        Node {
            width: percent(100),
            height: px(46),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            column_gap: px(10),
            ..default()
        },
        None,
    );
    commands.entity(controls).insert((
        GameSummaryActions {
            delay: actions_delay,
        },
        if animation.elapsed >= actions_delay {
            Visibility::Visible
        } else {
            Visibility::Hidden
        },
    ));
    if *tournament_complete {
        add_action_button(
            commands,
            controls,
            "返回大厅",
            UiAction::Lobby(LobbyUiAction::ReturnToLobby),
            ButtonKind::Primary,
            assets,
        );
    } else {
        let ready = game
            .players
            .iter()
            .find(|player| player.id == game.you)
            .is_some_and(|player| player.ready);
        if ready {
            add_disabled_action_button(commands, controls, "已准备", assets);
        } else {
            add_action_button(
                commands,
                controls,
                "准备下一手",
                UiAction::Lobby(LobbyUiAction::PlayAgain),
                ButtonKind::Primary,
                assets,
            );
        }
    }
}

fn add_texas_summary_avatar(
    commands: &mut Commands,
    parent: Entity,
    player: &TexasHoldemPlayerState,
    avatars: &AvatarImages,
    assets: &UiAssets,
) {
    let avatar = player.avatar.and_then(|id| avatars.remote.get(&id));
    add_ready_avatar(
        commands,
        parent,
        &player.name,
        avatar,
        32.0,
        player.ready,
        assets,
    );
}
