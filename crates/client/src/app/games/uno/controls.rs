use super::{
    UNO_ACTION_AREA_BOTTOM, UNO_ACTION_AREA_HEIGHT, UnoUiAction, UnoUiState, uno_card_is_playable,
    uno_pair_for_selection, uno_ui_color,
};
use crate::app::presentation::{
    ButtonKind, ButtonTint, GameButtonImageMode, GameButtonSpec, PanelSkin, add_section_title,
    add_text, add_textured_game_button, spawn_node,
};
use crate::app::runtime::UiAssets;
use crate::app::shell::{UiAction, add_cozy_panel_with_skin};
use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use leocard_protocol::{UnoPendingSwapView, UnoPhaseView, UnoSnapshot};
use leocard_uno::{UnoCard, UnoColor, UnoFace, UnoFlipSide, UnoPendingDrawKind};

fn uno_action_width(label: &str) -> f32 {
    match label.chars().count() {
        0..=6 => 164.0,
        7..=9 => 190.0,
        _ => 216.0,
    }
}

fn uno_button_texture(assets: &UiAssets, kind: ButtonKind) -> (&Handle<Image>, &Handle<Image>) {
    match kind {
        ButtonKind::Primary => (
            &assets.controls.game_play_button,
            &assets.controls.game_play_button_hover,
        ),
        ButtonKind::Warning => (
            &assets.controls.game_warning_button,
            &assets.controls.game_warning_button_hover,
        ),
        ButtonKind::Secondary => (
            &assets.controls.game_pass_button,
            &assets.controls.game_pass_button_hover,
        ),
        ButtonKind::Pass => (
            &assets.controls.game_hint_button,
            &assets.controls.game_hint_button_hover,
        ),
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "game buttons share one texture renderer with explicit action and size"
)]
fn add_uno_button(
    commands: &mut Commands,
    parent: Entity,
    label: &str,
    action: Option<UiAction>,
    kind: ButtonKind,
    assets: &UiAssets,
    height: f32,
    min_width: f32,
) -> Entity {
    let (normal, hovered) = uno_button_texture(assets, kind);
    let width = (uno_action_width(label) * height / 52.0).max(min_width);
    let font_size = if label.chars().count() >= 10 {
        15.0
    } else {
        18.0
    };
    add_textured_game_button(
        commands,
        parent,
        assets,
        GameButtonSpec {
            label,
            action,
            normal,
            hovered,
            width,
            height,
            font_size,
            image_mode: GameButtonImageMode::Stretch,
        },
    )
    .0
}

pub(super) fn add_uno_action_button(
    commands: &mut Commands,
    parent: Entity,
    label: &str,
    action: UiAction,
    kind: ButtonKind,
    assets: &UiAssets,
) -> Entity {
    add_uno_button(
        commands,
        parent,
        label,
        Some(action),
        kind,
        assets,
        52.0,
        164.0,
    )
}

pub(super) fn add_uno_disabled_action_button(
    commands: &mut Commands,
    parent: Entity,
    label: &str,
    assets: &UiAssets,
) -> Entity {
    add_uno_button(
        commands,
        parent,
        label,
        None,
        ButtonKind::Secondary,
        assets,
        52.0,
        164.0,
    )
}

pub(super) fn add_uno_actions(
    commands: &mut Commands,
    table: Entity,
    game: &UnoSnapshot,
    ui: &UnoUiState,
    assets: &UiAssets,
) {
    if !matches!(game.phase, UnoPhaseView::Playing)
        || game
            .players
            .iter()
            .find(|player| player.id == game.you)
            .is_some_and(|player| player.eliminated)
    {
        return;
    }
    if let Some(pending) = game.pending_swap {
        if matches!(pending, UnoPendingSwapView::SwapOneGive { player, .. } if player == game.you) {
            let actions = spawn_node(
                commands,
                table,
                Node {
                    position_type: PositionType::Absolute,
                    left: px(390),
                    bottom: px(UNO_ACTION_AREA_BOTTOM),
                    width: px(500),
                    height: px(UNO_ACTION_AREA_HEIGHT),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                None,
            );
            if ui.selected.len() == 1 {
                add_uno_action_button(
                    commands,
                    actions,
                    "交出选中的牌",
                    UiAction::Uno(UnoUiAction::SubmitCard),
                    ButtonKind::Primary,
                    assets,
                );
            } else {
                add_uno_disabled_action_button(commands, actions, "请选择一张要交出的牌", assets);
            }
        }
        return;
    }
    let jump_in = game.your_jump_in_card;
    if game.current_player != Some(game.you) && jump_in.is_none() {
        return;
    }
    let actions = spawn_node(
        commands,
        table,
        Node {
            position_type: PositionType::Absolute,
            left: px(390),
            bottom: px(UNO_ACTION_AREA_BOTTOM),
            width: px(500),
            height: px(UNO_ACTION_AREA_HEIGHT),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            column_gap: px(10),
            ..default()
        },
        None,
    );
    if let Some(card) = jump_in {
        add_uno_action_button(
            commands,
            actions,
            "抢出",
            UiAction::Uno(UnoUiAction::JumpIn(card)),
            ButtonKind::Primary,
            assets,
        );
        return;
    }
    if game.current_color.is_none() {
        return;
    }
    let own_skips = game
        .players
        .iter()
        .find(|player| player.id == game.you)
        .map_or(0, |player| player.skipped_turns);
    let selected = match ui.selected.iter().copied().collect::<Vec<_>>().as_slice() {
        [card] if uno_card_is_playable(game, *card) => Some((*card, 1)),
        [first, second]
            if uno_card_is_playable(game, *first)
                && uno_pair_for_selection(game, *first) == Some(*second) =>
        {
            Some((*first, 2))
        }
        _ => None,
    };
    if game.uno_declared.contains(&game.you) && selected.is_none() {
        return;
    }
    if let Some((card, card_count)) = selected {
        add_uno_action_button(
            commands,
            actions,
            if matches!(
                card.face(),
                UnoFace::Wild
                    | UnoFace::DarkWild
                    | UnoFace::WildDrawTwo
                    | UnoFace::WildDrawFour
                    | UnoFace::WildDrawColor
                    | UnoFace::WildPowerReverse
                    | UnoFace::WildNoU
                    | UnoFace::WildStackThree
                    | UnoFace::WildStackNumber
                    | UnoFace::WildReverseDrawFour
                    | UnoFace::WildDrawSix
                    | UnoFace::WildDrawTen
            ) {
                "出牌并选色"
            } else if card_count == 2 {
                "一次打出 ×2"
            } else {
                "出牌"
            },
            UiAction::Uno(UnoUiAction::SubmitCard),
            ButtonKind::Primary,
            assets,
        );
    } else if game.pending_kind.is_some() {
        add_uno_action_button(
            commands,
            actions,
            &if game.pending_kind == Some(UnoPendingDrawKind::FlipWildDrawColor) {
                "接受指定颜色摸牌".to_owned()
            } else {
                format!("接受 +{}", game.pending_draw)
            },
            UiAction::Uno(UnoUiAction::AcceptDrawPenalty),
            ButtonKind::Warning,
            assets,
        );
        if game.challenge_offender.is_some() {
            add_uno_action_button(
                commands,
                actions,
                if game.pending_kind == Some(UnoPendingDrawKind::FlipWildDrawColor) {
                    "质疑指定颜色摸牌"
                } else if game.pending_kind == Some(UnoPendingDrawKind::FlipWildDrawTwo) {
                    "质疑万能 +2"
                } else {
                    "质疑 +4"
                },
                UiAction::Uno(UnoUiAction::ChallengeDrawFour),
                ButtonKind::Pass,
                assets,
            );
        }
    } else if game.pending_skip > 0 || own_skips > 0 {
        if game.rules.action_stacking_enabled() {
            add_uno_action_button(
                commands,
                actions,
                &format!("接受禁手 ×{}", game.pending_skip + own_skips),
                UiAction::Uno(UnoUiAction::ResolveSkip),
                ButtonKind::Pass,
                assets,
            );
        }
    } else if game.your_drawn_card.is_some() {
        if !game.rules.is_no_mercy() || !game.rules.no_mercy.draw_until_playable {
            add_uno_action_button(
                commands,
                actions,
                "结束回合",
                UiAction::Uno(UnoUiAction::PassAfterDraw),
                ButtonKind::Secondary,
                assets,
            );
        } else {
            add_uno_disabled_action_button(commands, actions, "必须打出摸到的牌", assets);
        }
    } else {
        let has_response = game
            .your_hand
            .iter()
            .copied()
            .any(|card| uno_card_is_playable(game, card));
        if game.sudden_death_free_play {
            add_uno_disabled_action_button(commands, actions, "任选一张出牌", assets);
        } else if game.rules.sudden_death_enabled()
            && game
                .players
                .iter()
                .filter(|player| !player.eliminated)
                .count()
                == 2
            && game.draw_pile_len == 0
        {
            if has_response {
                add_uno_disabled_action_button(commands, actions, "请选择一张出牌", assets);
            } else {
                add_uno_action_button(
                    commands,
                    actions,
                    "无法响应",
                    UiAction::Uno(UnoUiAction::DrawCard),
                    ButtonKind::Secondary,
                    assets,
                );
            }
        } else {
            add_uno_action_button(
                commands,
                actions,
                "摸牌",
                UiAction::Uno(UnoUiAction::DrawCard),
                ButtonKind::Secondary,
                assets,
            );
        }
    }
}

pub(super) fn add_uno_callout_actions(
    commands: &mut Commands,
    table: Entity,
    game: &UnoSnapshot,
    assets: &UiAssets,
) {
    let eliminated = game
        .players
        .iter()
        .find(|player| player.id == game.you)
        .is_some_and(|player| player.eliminated);
    let targets = if eliminated
        || !game.rules.uno_callout()
        || !matches!(game.phase, UnoPhaseView::Playing)
    {
        Vec::new()
    } else {
        game.uno_exposed
            .iter()
            .copied()
            .filter(|target| *target != game.you)
            .collect::<Vec<_>>()
    };
    let can_call = game.can_call_uno;
    let callouts = spawn_node(
        commands,
        table,
        Node {
            position_type: PositionType::Absolute,
            right: px(24),
            bottom: px(16),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::FlexEnd,
            row_gap: px(5),
            ..default()
        },
        None,
    );
    commands.entity(callouts).insert(GlobalZIndex(1700));
    for target in targets {
        let name = game
            .players
            .iter()
            .find(|player| player.id == target)
            .map(|player| player.name.as_str())
            .unwrap_or("玩家");
        add_subtle_uno_button(
            commands,
            callouts,
            &format!("检举 {name}"),
            Some(UiAction::Uno(UnoUiAction::Report(target))),
            ButtonKind::Warning,
            assets,
        );
    }
    add_subtle_uno_button(
        commands,
        callouts,
        "UNO!",
        can_call.then_some(UiAction::Uno(UnoUiAction::Call)),
        ButtonKind::Primary,
        assets,
    );
}

fn add_subtle_uno_button(
    commands: &mut Commands,
    parent: Entity,
    label: &str,
    action: Option<UiAction>,
    kind: ButtonKind,
    assets: &UiAssets,
) {
    add_uno_button(commands, parent, label, action, kind, assets, 42.0, 128.0);
}

pub(super) fn add_initial_color_choice(
    commands: &mut Commands,
    parent: Entity,
    game: &UnoSnapshot,
    assets: &UiAssets,
) {
    let choosing_player = match game.pending_swap {
        Some(
            UnoPendingSwapView::ChooseColor { player }
            | UnoPendingSwapView::ColorRoulette { player },
        ) => Some(player),
        _ => game.current_player,
    };
    let name = choosing_player
        .and_then(|id| game.players.iter().find(|player| player.id == id))
        .map(|player| player.name.as_str())
        .unwrap_or("玩家");
    let own_turn = choosing_player == Some(game.you);
    let after_swap = matches!(
        game.pending_swap,
        Some(UnoPendingSwapView::ChooseColor { .. })
    );
    let color_roulette = matches!(
        game.pending_swap,
        Some(UnoPendingSwapView::ColorRoulette { .. })
    );
    add_color_choice_overlay(
        commands,
        parent,
        if own_turn && color_roulette {
            "选择一种颜色并摸牌，直到翻出该颜色".to_owned()
        } else if own_turn && after_swap {
            "换牌完成，请选择后续颜色".to_owned()
        } else if own_turn {
            "起始牌是万能牌，请选择颜色".to_owned()
        } else if color_roulette {
            format!("等待 {name} 选择颜色轮盘目标色")
        } else if after_swap {
            format!("等待 {name} 选择后续颜色")
        } else {
            format!("等待 {name} 选择起始颜色")
        },
        own_turn.then_some(None),
        game.flip_side,
        assets,
    );
}

pub(super) fn add_play_color_choice(
    commands: &mut Commands,
    parent: Entity,
    flip_side: Option<UnoFlipSide>,
    card: UnoCard,
    assets: &UiAssets,
) {
    add_color_choice_overlay(
        commands,
        parent,
        "选择后续颜色".to_owned(),
        Some(Some(card)),
        flip_side,
        assets,
    );
}

fn add_color_choice_overlay(
    commands: &mut Commands,
    parent: Entity,
    title: String,
    choice: Option<Option<UnoCard>>,
    flip_side: Option<UnoFlipSide>,
    assets: &UiAssets,
) {
    let overlay = spawn_node(
        commands,
        parent,
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            right: px(0),
            top: px(0),
            bottom: px(0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        Some(Color::BLACK.with_alpha(0.58)),
    );
    commands
        .entity(overlay)
        .insert((GlobalZIndex(2050), FocusPolicy::Block));
    let panel = add_cozy_panel_with_skin(
        commands,
        overlay,
        Node {
            width: px(480),
            padding: UiRect::all(px(22)),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: px(18),
            ..default()
        },
        PanelSkin::Popup,
        assets,
    );
    add_section_title(commands, panel, title, assets);
    if let Some(card) = choice {
        let color_row = spawn_node(
            commands,
            panel,
            Node {
                flex_direction: FlexDirection::Row,
                column_gap: px(12),
                ..default()
            },
            None,
        );
        let colors = match flip_side {
            Some(UnoFlipSide::Dark) => UnoColor::DARK,
            Some(UnoFlipSide::Light) | None => UnoColor::LIGHT,
        };
        for color in colors {
            let button = commands
                .spawn((
                    Button,
                    card.map_or(
                        UiAction::Uno(UnoUiAction::ChooseInitialColor(color)),
                        |card| UiAction::Uno(UnoUiAction::PlayCard(card, Some(color))),
                    ),
                    ButtonTint {
                        normal: uno_ui_color(color),
                        hovered: uno_ui_color(color).mix(&Color::WHITE, 0.22),
                        pressed: uno_ui_color(color).mix(&Color::BLACK, 0.22),
                    },
                    Node {
                        width: px(78),
                        height: px(78),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        border: UiRect::all(px(3)),
                        border_radius: BorderRadius::all(percent(50)),
                        ..default()
                    },
                    BackgroundColor(uno_ui_color(color)),
                    BorderColor::all(Color::WHITE.with_alpha(0.78)),
                ))
                .id();
            commands.entity(color_row).add_child(button);
            let label = add_text(
                commands,
                button,
                color.to_string(),
                15.0,
                Color::WHITE,
                assets,
            );
            commands.entity(label).insert(FocusPolicy::Pass);
        }
        if card.is_some() {
            add_uno_action_button(
                commands,
                panel,
                "取消",
                UiAction::Uno(UnoUiAction::CloseColorChoice),
                ButtonKind::Secondary,
                assets,
            );
        }
    }
}
