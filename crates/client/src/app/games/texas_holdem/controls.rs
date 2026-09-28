use super::{
    TEXAS_PORTRAIT_WIDTH, TexasChipTableState, TexasHoldemAssets, TexasHoldemUiAction,
    TexasHoldemUiState, TexasPlayerPanel, TexasPlayerShake, TexasRaiseAdjustButton,
    add_role_tokens, add_texas_card, add_texas_chip_popup, texas_player_border_color,
};
use crate::app::presentation::{
    ButtonKind, GameButtonImageMode, GameButtonSpec, MUTED, PendingDealSound, PlayerMenuProfile,
    PlayerPortraitSpec, TEXT, TurnBorderAnimationKey, TurnBorderMaterial, add_player_portrait,
    add_textured_game_button, add_turn_border_trace_with_radius, attach_start_game_seat_transition,
    spawn_node,
};
use crate::app::runtime::{AvatarImages, UiAssets};
use crate::app::shell::{HomeHighlightKind, SeatSide, UiAction};
use bevy::prelude::*;
use bevy::ui::{FocusPolicy, VisualBox};
use leocard_protocol::{
    GameKind, PlayerId, TexasHoldemPhaseView, TexasHoldemPlayerState, TexasHoldemSnapshot,
};
use leocard_texas_holdem::{TexasHoldemAction, TexasHoldemBlindKind};

#[expect(
    clippy::too_many_arguments,
    reason = "the own-area builder combines game controls with explicit visual resources"
)]
pub(super) fn add_texas_own_area(
    commands: &mut Commands,
    table: Entity,
    game: &TexasHoldemSnapshot,
    own: &TexasHoldemPlayerState,
    deal_delays: Option<&[f32]>,
    ui: &mut TexasHoldemUiState,
    interaction_menu_open: Option<PlayerId>,
    assets: &UiAssets,
    game_assets: &TexasHoldemAssets,
    avatars: &AvatarImages,
    turn_border_materials: &mut Assets<TurnBorderMaterial>,
    chip_state: &TexasChipTableState,
    start_transition_active: bool,
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
            side: SeatSide::Left,
            avatar_size: 52.0 * 1.17,
            auto_play: own.auto_play,
            menu_open: interaction_menu_open == Some(own.id),
            menu_above: true,
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
    add_texas_chip_popup(
        commands,
        own_seat,
        &own.name,
        own.stack,
        None,
        assets,
        game_assets,
        &chip_state.stack_counts(own.id),
    );

    // 一手结束后，自己的底牌也和其他玩家一样改放到面前的筹码区。
    if !matches!(game.phase, TexasHoldemPhaseView::HandComplete { .. }) {
        let omaha = game.your_hole_cards.len() == 4;
        let (area_width, card_width, card_height, card_gap) = if omaha {
            // 牌顶保持在操作按钮下缘（bottom 124）以下，留出 10 px 间距。
            (324.0, 78.0, 106.0, 4.0)
        } else {
            (180.0, 84.0, 114.0, 9.0)
        };
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
                let delay = deal_delays
                    .and_then(|delays| delays.get(index).or_else(|| delays.last()))
                    .copied();
                let animation = delay.map(|delay| (delay, Vec2::new(-280.0, -257.0)));
                add_texas_card(
                    commands,
                    hole_cards,
                    card,
                    (card_width, card_height),
                    animation,
                    assets,
                );
                if let Some(delay) = delay {
                    commands.spawn(PendingDealSound {
                        remaining: delay,
                        variant: index % assets.audio.deal_sounds.len().max(1),
                    });
                }
            }
        }
    }
    add_texas_actions(commands, table, game, own, ui, assets, game_assets);
}

fn add_texas_actions(
    commands: &mut Commands,
    table: Entity,
    game: &TexasHoldemSnapshot,
    own: &TexasHoldemPlayerState,
    ui: &mut TexasHoldemUiState,
    assets: &UiAssets,
    game_assets: &TexasHoldemAssets,
) {
    let actions = spawn_node(
        commands,
        table,
        Node {
            position_type: PositionType::Absolute,
            left: percent(50),
            bottom: px(124),
            width: px(620),
            min_height: px(42),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: px(7),
            ..default()
        },
        None,
    );
    commands
        .entity(actions)
        .insert(UiTransform::from_translation(Val2::px(-310.0, 0.0)));
    if !matches!(game.phase, TexasHoldemPhaseView::Betting { .. }) {
        return;
    }
    if let Some(blind) = game.blind_to_post {
        if blind.player == game.you {
            let label = match blind.kind {
                TexasHoldemBlindKind::Small => format!("下小盲 {}", blind.amount),
                TexasHoldemBlindKind::Big => format!("下大盲 {}", blind.amount),
            };
            let row = spawn_node(
                commands,
                actions,
                Node {
                    width: percent(100),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                None,
            );
            add_texas_action_button(
                commands,
                row,
                &label,
                TexasHoldemAction::PostBlind,
                ButtonKind::Primary,
                assets,
                game_assets,
            );
        }
        return;
    }
    if game.current_player != Some(game.you) {
        return;
    }
    let maximum_target = own.committed_street.saturating_add(own.stack);
    let minimum_target = game.minimum_raise_to.min(maximum_target);
    if ui.raise_to < minimum_target || ui.raise_to > maximum_target {
        ui.raise_to = minimum_target;
    }
    let row = spawn_node(
        commands,
        actions,
        Node {
            width: percent(100),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            column_gap: px(8),
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
        game_assets,
    );
    if game.amount_to_call == 0 {
        add_texas_action_button(
            commands,
            row,
            "过牌",
            TexasHoldemAction::Check,
            ButtonKind::Secondary,
            assets,
            game_assets,
        );
    } else {
        add_texas_action_button(
            commands,
            row,
            &format!("跟注 {}", game.amount_to_call.min(own.stack)),
            TexasHoldemAction::Call,
            ButtonKind::Secondary,
            assets,
            game_assets,
        );
    }
    if game.raise_allowed && maximum_target >= game.minimum_raise_to {
        let step = game
            .minimum_raise_to
            .saturating_sub(game.current_bet)
            .max(1);
        let lower = ui.raise_to.saturating_sub(step).max(minimum_target);
        let higher = ui.raise_to.saturating_add(step).min(maximum_target);
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
            game_assets,
        );
        add_texas_action_button(
            commands,
            row,
            &format!("加注到 {}", ui.raise_to),
            TexasHoldemAction::RaiseTo(ui.raise_to),
            ButtonKind::Primary,
            assets,
            game_assets,
        );
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
            game_assets,
        );
    }
    add_texas_action_button(
        commands,
        row,
        "全下",
        TexasHoldemAction::AllIn,
        ButtonKind::Warning,
        assets,
        game_assets,
    );
}

fn add_texas_action_button(
    commands: &mut Commands,
    parent: Entity,
    label: &str,
    action: TexasHoldemAction,
    kind: ButtonKind,
    assets: &UiAssets,
    game_assets: &TexasHoldemAssets,
) {
    add_texas_sized_button(
        commands,
        parent,
        label,
        UiAction::TexasHoldem(TexasHoldemUiAction::Act(action)),
        kind,
        assets,
        game_assets,
    );
}

fn add_raise_adjust_button(
    commands: &mut Commands,
    parent: Entity,
    target: u32,
    enabled: bool,
    repeat: TexasRaiseAdjustButton,
    game_assets: &TexasHoldemAssets,
) {
    let (normal, hovered) = if repeat.direction < 0 {
        (&game_assets.adjust_left, &game_assets.adjust_left_hover)
    } else {
        (&game_assets.adjust_right, &game_assets.adjust_right_hover)
    };
    if enabled {
        let button = commands
            .spawn((
                Button,
                UiAction::TexasHoldem(TexasHoldemUiAction::SetRaiseTo(target)),
                Node {
                    width: px(48),
                    height: px(50),
                    ..default()
                },
                texas_adjust_button_image(normal.clone()),
                repeat,
            ))
            .id();
        commands.entity(parent).add_child(button);
        let overlay = commands
            .spawn((
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
                FocusPolicy::Pass,
            ))
            .id();
        commands.entity(button).add_child(overlay);
        commands.entity(button).insert(HomeHighlightKind::Button {
            overlay,
            arrows: None,
        });
    } else {
        let button = commands
            .spawn((
                Node {
                    width: px(48),
                    height: px(50),
                    ..default()
                },
                texas_adjust_button_image(normal.clone()).with_color(Color::WHITE.with_alpha(0.48)),
                FocusPolicy::Pass,
            ))
            .id();
        commands.entity(parent).add_child(button);
    }
}

fn add_texas_sized_button(
    commands: &mut Commands,
    parent: Entity,
    label: &str,
    action: UiAction,
    kind: ButtonKind,
    assets: &UiAssets,
    game_assets: &TexasHoldemAssets,
) -> Entity {
    let (normal, hovered) = match kind {
        ButtonKind::Primary => (
            &game_assets.action_primary,
            &game_assets.action_primary_hover,
        ),
        ButtonKind::Secondary => (
            &game_assets.action_secondary,
            &game_assets.action_secondary_hover,
        ),
        ButtonKind::Warning => (
            &game_assets.action_warning,
            &game_assets.action_warning_hover,
        ),
        ButtonKind::Pass => (&game_assets.action_pass, &game_assets.action_pass_hover),
    };
    let (button, label_entity) = add_textured_game_button(
        commands,
        parent,
        assets,
        GameButtonSpec {
            label,
            action: Some(action),
            normal,
            hovered,
            width: 118.0,
            height: 50.0,
            font_size: 17.0,
            image_mode: GameButtonImageMode::Sliced {
                border: 30.0,
                corner_scale: 0.42,
            },
        },
    );
    commands.entity(button).insert(Node {
        width: px(118),
        min_width: px(118),
        height: px(50),
        min_height: px(50),
        padding: UiRect::axes(px(9), px(4)),
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
