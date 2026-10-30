use super::{
    TEXAS_PORTRAIT_WIDTH, TexasChipTableState, TexasHoldemUiAction, TexasHoldemUiState,
    TexasInitialDeal, TexasPlayerPanel, TexasPlayerShake, TexasRaiseAdjustButton, TexasTableLayout,
    add_role_tokens, add_texas_card, add_texas_chip_popup, add_texas_win_rate,
    texas_player_border_color,
};
use crate::app::presentation::{
    ButtonKind, GameButtonImageMode, GameButtonSpec, MUTED, PendingDealSound, PlayerMenuProfile,
    PlayerPortraitSpec, SeatSide, TEXT, TablePieceDeal, TurnBorderAnimationKey, TurnBorderMaterial,
    add_player_portrait, add_textured_game_button, add_turn_border_trace_with_radius,
    attach_start_game_seat_transition,
};
use crate::app::runtime::{AvatarImages, UiAssets};
use crate::app::shell::UiAction;

use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy::ui::VisualBox;
use bevy::ui_widgets::Button;
use leocard_protocol::{
    GameKind, PlayerId, TexasHoldemPhaseView, TexasHoldemPlayerState, TexasHoldemSnapshot,
};
use leocard_texas_holdem::{RaiseSizing, TexasHoldemAction};
use leocard_ui::{ButtonHighlight, UiComposer, spawn_node};

#[expect(
    clippy::too_many_arguments,
    reason = "the own-area builder combines game controls with explicit visual resources"
)]
pub(super) fn add_texas_own_area(
    commands: &mut UiComposer,
    table: Entity,
    game: &TexasHoldemSnapshot,
    own: &TexasHoldemPlayerState,
    initial_deal: Option<&TexasInitialDeal>,
    ui: &mut TexasHoldemUiState,
    interaction_menu_open: Option<PlayerId>,
    assets: &UiAssets,
    avatars: &AvatarImages,
    turn_border_materials: &mut Assets<TurnBorderMaterial>,
    chip_state: &TexasChipTableState,
    start_transition_active: bool,
    intro_only: bool,
) {
    let portrait_height = 76.0 * 1.17;
    let own_seat = spawn_node(
        commands,
        table,
        Node {
            position_type: PositionType::Absolute,
            left: px(10),
            bottom: px(8),
            width: px(TEXAS_PORTRAIT_WIDTH),
            height: px(portrait_height),
            ..default()
        },
        None,
    );
    commands.entity(own_seat).insert(TexasPlayerShake(own.id));
    let portrait = add_player_portrait(
        commands,
        own_seat,
        Node {
            width: px(TEXAS_PORTRAIT_WIDTH),
            height: px(portrait_height),
            ..default()
        },
        PlayerPortraitSpec {
            player: own.id,
            profile: PlayerMenuProfile {
                name: &own.name,
                avatar: own.avatar.and_then(|id| avatars.remote.get(&id)),
                reference_points: own.reference_points,
                completed_games: own.completed_games,
                game_profiles: &own.game_profiles,
            },
            avatar_size: 52.0 * 1.17,
            auto_play: own.auto_play,
            menu_open: interaction_menu_open == Some(own.id),
            name_color: if own.folded { MUTED } else { TEXT },
        },
        assets,
    );
    attach_start_game_seat_transition(commands, portrait.portrait, own.id, start_transition_active);
    let base_border = texas_player_border_color(own, game.current_player == Some(own.id));
    commands
        .entity(portrait.avatar_ring)
        .entry::<Node>()
        .and_modify(|mut node| node.border_radius = BorderRadius::all(px(52.0 * 1.17 * 0.2)));
    commands.entity(portrait.avatar_ring).insert((
        TexasPlayerPanel {
            player: own.id,
            base_border,
        },
        Outline::new(px(3.0), px(0), base_border),
        BoxShadow::new(Color::NONE, px(0), px(0), px(0), px(0)),
    ));
    if !own.folded && game.current_player == Some(own.id) {
        add_turn_border_trace_with_radius(
            commands,
            portrait.avatar_ring,
            turn_border_materials,
            TurnBorderAnimationKey::new(GameKind::TexasHoldem, game.match_id, own.id),
            52.0 * 1.17 * 0.2,
            52.0 * 1.17,
        );
    }
    add_role_tokens(commands, portrait.avatar_ring, own.id, game, assets);
    if !intro_only {
        add_texas_win_rate(
            commands,
            portrait.avatar_ring,
            own.id,
            SeatSide::Top,
            game,
            &ui.spectator,
            assets,
        );
    }
    if intro_only {
        return;
    }
    add_texas_chip_popup(
        commands,
        own_seat,
        own.stack,
        None,
        assets,
        &chip_state.stack_counts(own.id),
    );

    // 一手结束后，自己的底牌也和其他玩家一样改放到面前的筹码区。
    if !matches!(game.phase, TexasHoldemPhaseView::HandComplete { .. }) {
        let omaha = game.your_hole_cards.len() == 4;
        let (area_width, card_width, card_height, card_gap) = if omaha {
            // 四张底牌使用较窄牌面，与操作行保留间隔。
            (324.0, 78.0, 106.0, 4.0)
        } else {
            (180.0, 84.0, 114.0, 9.0)
        };
        let scale = TexasTableLayout::HAND_SCALE;
        let (area_width, card_width, card_height, card_gap) = (
            area_width * scale,
            card_width * scale,
            card_height * scale,
            card_gap * scale,
        );
        let hole_cards = spawn_node(
            commands,
            table,
            Node {
                position_type: PositionType::Absolute,
                left: percent(50),
                bottom: px(8),
                width: px(area_width),
                height: px(card_height),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::FlexEnd,
                justify_content: JustifyContent::Center,
                column_gap: px(card_gap),
                ..default()
            },
            None,
        );
        commands
            .entity(hole_cards)
            .insert(UiTransform::from_translation(Val2::px(
                -area_width / 2.0,
                0.0,
            )));
        if !own.folded {
            for (index, card) in game.your_hole_cards.iter().copied().enumerate() {
                let delay = initial_deal
                    .and_then(|deal| {
                        deal.own_delays
                            .get(index)
                            .or_else(|| deal.own_delays.last())
                    })
                    .copied();
                let animation = initial_deal.zip(delay).map(|(deal, delay)| {
                    if cfg!(target_os = "android") {
                        TablePieceDeal::from_anchor(deal.source, delay)
                    } else {
                        TablePieceDeal::new(delay, Vec2::new(-280.0, -257.0))
                    }
                });
                add_texas_card(
                    commands,
                    hole_cards,
                    card,
                    (card_width, card_height),
                    animation,
                    assets,
                );
                if let Some(delay) = delay {
                    commands.effect(PendingDealSound {
                        remaining: delay,
                        variant: index % assets.audio.deal_sounds.len().max(1),
                    });
                }
            }
        }
    }
    add_texas_actions(commands, table, game, own, ui, assets);
}

fn add_texas_actions(
    commands: &mut UiComposer,
    table: Entity,
    game: &TexasHoldemSnapshot,
    own: &TexasHoldemPlayerState,
    ui: &mut TexasHoldemUiState,
    assets: &UiAssets,
) {
    if !matches!(game.phase, TexasHoldemPhaseView::Betting { .. })
        || game.blind_to_post.is_some()
        || game.current_player != Some(game.you)
    {
        return;
    }
    let actions = spawn_node(
        commands,
        table,
        Node {
            position_type: PositionType::Absolute,
            left: percent(50),
            bottom: px(TexasTableLayout::ACTION_BOTTOM),
            width: px(620.0 * TexasTableLayout::ACTION_SCALE),
            min_height: px(42.0 * TexasTableLayout::ACTION_SCALE),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: px(7.0 * TexasTableLayout::ACTION_SCALE),
            ..default()
        },
        None,
    );
    commands
        .entity(actions)
        .insert(UiTransform::from_translation(Val2::px(
            -310.0 * TexasTableLayout::ACTION_SCALE,
            0.0,
        )));
    let maximum_target = game.maximum_raise_to.unwrap_or(game.current_bet);
    let step = game
        .minimum_raise_to
        .saturating_sub(game.current_bet)
        .max(1);
    let sizing = RaiseSizing::new(game.current_bet, step, maximum_target);
    let row = spawn_node(
        commands,
        actions,
        Node {
            width: percent(100),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            column_gap: px(8.0 * TexasTableLayout::ACTION_SCALE),
            ..default()
        },
        None,
    );
    add_texas_action_button(
        commands,
        row,
        "弃牌",
        TexasHoldemAction::Fold,
        ButtonKind::Pass,
        assets,
    );
    if game.amount_to_call == 0 {
        add_texas_action_button(
            commands,
            row,
            "过牌",
            TexasHoldemAction::Check,
            ButtonKind::Secondary,
            assets,
        );
    } else {
        add_texas_action_button(
            commands,
            row,
            &format!("跟注 {}", game.amount_to_call.min(own.stack)),
            TexasHoldemAction::Call,
            ButtonKind::Secondary,
            assets,
        );
    }
    if game.raise_allowed
        && let Some(maximum_target) = sizing.maximum()
    {
        let minimum_target = sizing.minimum();
        ui.raise_to = sizing.normalize(ui.raise_to).expect("affordable raise");
        let lower = ui.raise_to.saturating_sub(step).max(minimum_target);
        let higher = ui.raise_to.saturating_add(step).min(maximum_target);
        if maximum_target > minimum_target {
            add_raise_adjust_button(
                commands,
                row,
                lower,
                lower < ui.raise_to,
                TexasRaiseAdjustButton {
                    direction: -1,
                    step,
                    minimum: minimum_target,
                    maximum: maximum_target,
                },
                assets,
            );
        }
        add_texas_action_button(
            commands,
            row,
            &format!("加注到 {}", ui.raise_to),
            TexasHoldemAction::RaiseTo(ui.raise_to),
            ButtonKind::Primary,
            assets,
        );
        if maximum_target > minimum_target {
            add_raise_adjust_button(
                commands,
                row,
                higher,
                higher > ui.raise_to,
                TexasRaiseAdjustButton {
                    direction: 1,
                    step,
                    minimum: minimum_target,
                    maximum: maximum_target,
                },
                assets,
            );
        }
    }
    if game.all_in_allowed {
        add_texas_action_button(
            commands,
            row,
            "全下",
            TexasHoldemAction::AllIn,
            ButtonKind::Warning,
            assets,
        );
    }
}

fn add_raise_adjust_button(
    commands: &mut UiComposer,
    parent: Entity,
    target: u32,
    enabled: bool,
    repeat: TexasRaiseAdjustButton,
    assets: &UiAssets,
) {
    let (normal, hovered) = if repeat.direction < 0 {
        (
            &assets.controls.adjust_left,
            &assets.controls.adjust_left_hover,
        )
    } else {
        (
            &assets.controls.adjust_right,
            &assets.controls.adjust_right_hover,
        )
    };
    if enabled {
        let button = commands.child(
            parent,
            (
                Button,
                UiAction::TexasHoldem(TexasHoldemUiAction::SetRaiseTo(target)),
                Node {
                    width: px(48.0 * TexasTableLayout::ACTION_SCALE),
                    height: px(50.0 * TexasTableLayout::ACTION_SCALE),
                    flex_shrink: 0.0,
                    ..default()
                },
                texas_adjust_button_image(normal.clone()),
                repeat,
            ),
        );
        let overlay = commands.child(
            button,
            (
                Node {
                    position_type: PositionType::Absolute,
                    left: px(0),
                    right: px(0),
                    top: px(0),
                    bottom: px(0),
                    ..default()
                },
                texas_adjust_button_image(hovered.clone()),
                Visibility::Hidden,
                Pickable::IGNORE,
            ),
        );
        commands.entity(button).insert(ButtonHighlight {
            overlay,
            arrows: None,
        });
    } else {
        commands.child(
            parent,
            (
                Node {
                    width: px(48.0 * TexasTableLayout::ACTION_SCALE),
                    height: px(50.0 * TexasTableLayout::ACTION_SCALE),
                    flex_shrink: 0.0,
                    ..default()
                },
                texas_adjust_button_image(normal.clone()).with_color(Color::WHITE.with_alpha(0.48)),
                Pickable::IGNORE,
            ),
        );
    }
}

fn add_texas_action_button(
    commands: &mut UiComposer,
    parent: Entity,
    label: &str,
    action: TexasHoldemAction,
    kind: ButtonKind,
    assets: &UiAssets,
) -> Entity {
    let (normal, hovered) = match kind {
        ButtonKind::Primary => (
            &assets.controls.game_play_button,
            &assets.controls.game_play_button_hover,
        ),
        ButtonKind::Secondary => (
            &assets.controls.game_hint_button,
            &assets.controls.game_hint_button_hover,
        ),
        ButtonKind::Warning => (
            &assets.controls.game_warning_button,
            &assets.controls.game_warning_button_hover,
        ),
        ButtonKind::Pass => (
            &assets.controls.game_pass_button,
            &assets.controls.game_pass_button_hover,
        ),
    };
    let (button, label_entity) = add_textured_game_button(
        commands,
        parent,
        assets,
        GameButtonSpec {
            label,
            action: Some(UiAction::TexasHoldem(TexasHoldemUiAction::Act(action))),
            normal,
            hovered,
            width: 118.0 * TexasTableLayout::ACTION_SCALE,
            height: 50.0 * TexasTableLayout::ACTION_SCALE,
            font_size: 17.0 * TexasTableLayout::ACTION_SCALE,
            image_mode: GameButtonImageMode::Sliced {
                border: 30.0,
                corner_scale: 0.42 * TexasTableLayout::ACTION_SCALE,
            },
        },
    );
    commands.entity(button).insert(Node {
        width: px(118.0 * TexasTableLayout::ACTION_SCALE),
        min_width: px(118.0 * TexasTableLayout::ACTION_SCALE),
        height: px(50.0 * TexasTableLayout::ACTION_SCALE),
        min_height: px(50.0 * TexasTableLayout::ACTION_SCALE),
        flex_shrink: 0.0,
        padding: UiRect::axes(
            px(9.0 * TexasTableLayout::ACTION_SCALE),
            px(4.0 * TexasTableLayout::ACTION_SCALE),
        ),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        ..default()
    });
    commands.entity(label_entity).insert((
        Node {
            margin: UiRect::ZERO,
            align_self: AlignSelf::Center,
            ..default()
        },
        TextColor(Color::WHITE),
    ));
    button
}

fn texas_adjust_button_image(texture: Handle<Image>) -> ImageNode {
    let mut image = ImageNode::new(texture).with_mode(NodeImageMode::Stretch);
    image.visual_box = VisualBox::BorderBox;
    image
}
