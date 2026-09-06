use super::*;

pub(super) fn add_texas_own_area(
    commands: &mut Commands,
    table: Entity,
    game: &TexasHoldemSnapshot,
    own: &TexasHoldemPlayerState,
    deal_delays: Option<&[f32]>,
    ui: &mut UiState,
    assets: &UiAssets,
    avatars: &AvatarImages,
    turn_border_materials: &mut Assets<TurnBorderMaterial>,
    chip_state: &TexasChipTableState,
    start_transition_active: bool,
) {
    let own_panel = spawn_node(
        commands,
        table,
        Node {
            position_type: PositionType::Absolute,
            left: px(10),
            bottom: px(8),
            width: px(118),
            min_width: px(118),
            max_width: px(118),
            height: px(48),
            min_height: px(48),
            max_height: px(48),
            flex_shrink: 0.0,
            padding: UiRect::axes(px(3), px(2)),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            column_gap: px(2),
            border: UiRect::all(px(TURN_BORDER_THICKNESS)),
            border_radius: BorderRadius::all(px(8)),
            ..default()
        },
        Some(HEADER_BG.with_alpha(0.94)),
    );
    let base_border = texas_player_border_color(own, game.current_player == Some(own.id));
    commands.entity(own_panel).insert((
        BorderColor::all(base_border),
        BoxShadow::new(Color::NONE, px(0), px(0), px(0), px(0)),
        TexasPlayerPanel {
            player: own.id,
            base_border,
        },
    ));
    attach_start_game_seat_transition(commands, own_panel, own.id, start_transition_active);
    decorate_player_panel(commands, own_panel, assets, 0.72);
    if !own.folded && game.current_player == Some(own.id) {
        add_turn_border_trace(
            commands,
            own_panel,
            turn_border_materials,
            TurnBorderAnimationKey::new(GameKind::TexasHoldem, game.match_id, own.id),
        );
    }
    let avatar_handle = own.avatar.and_then(|id| avatars.remote.get(&id));
    let avatar = add_avatar(commands, own_panel, &own.name, avatar_handle, 24.0, assets);
    commands.entity(avatar).insert(PlayerAvatarAnchor(own.id));
    add_role_tokens(commands, avatar, own.id, game, assets);
    let info = spawn_node(
        commands,
        own_panel,
        Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        None,
    );
    add_text(commands, info, &own.name, 11.0, TEXT, assets);
    add_text(commands, info, texas_player_status(own), 7.5, MUTED, assets);
    add_texas_chip_popup(
        commands,
        table,
        &own.name,
        own.stack,
        None,
        assets,
        &chip_state.stack_counts(own.id),
    );

    // 一手结束后，自己的底牌也和其他玩家一样改放到面前的筹码区。
    if !matches!(game.phase, TexasHoldemPhaseView::HandComplete { .. }) {
        let omaha = game.your_hole_cards.len() == 4;
        let (area_width, card_width, card_height, card_gap) = if omaha {
            (276.0, 62.0, 84.0, 4.0)
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
                let animation = delay.map(|delay| (delay, Vec2::new(-280.0, -245.0)));
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
        } else {
            let folded_label = spawn_node(
                commands,
                hole_cards,
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
                None,
            );
            commands
                .entity(folded_label)
                .insert((ZIndex(20), FocusPolicy::Pass));
            let label = add_text(commands, folded_label, "您已弃牌", 18.0, MUTED, assets);
            commands.entity(label).insert(TextShadow {
                offset: Vec2::new(1.0, 1.5),
                color: Color::BLACK.with_alpha(0.82),
            });
        }
    }
    add_texas_actions(commands, table, game, own, ui, assets);
}

fn add_texas_actions(
    commands: &mut Commands,
    table: Entity,
    game: &TexasHoldemSnapshot,
    own: &TexasHoldemPlayerState,
    ui: &mut UiState,
    assets: &UiAssets,
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
            );
        } else {
            let player = game
                .players
                .iter()
                .find(|player| player.id == blind.player)
                .map_or("玩家", |player| player.name.as_str());
            add_text(
                commands,
                actions,
                format!("等待 {player} 下盲注…"),
                14.0,
                MUTED,
                assets,
            );
        }
        return;
    }
    if game.current_player != Some(game.you) {
        add_text(commands, actions, "等待其他玩家行动…", 14.0, MUTED, assets);
        return;
    }
    let maximum_target = own.committed_street.saturating_add(own.stack);
    let minimum_target = game.minimum_raise_to.min(maximum_target);
    if ui.texas_holdem.raise_to < minimum_target || ui.texas_holdem.raise_to > maximum_target {
        ui.texas_holdem.raise_to = minimum_target;
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
    if game.raise_allowed && maximum_target >= game.minimum_raise_to {
        let step = game
            .minimum_raise_to
            .saturating_sub(game.current_bet)
            .max(1);
        let lower = ui
            .texas_holdem
            .raise_to
            .saturating_sub(step)
            .max(minimum_target);
        let higher = ui
            .texas_holdem
            .raise_to
            .saturating_add(step)
            .min(maximum_target);
        add_raise_adjust_button(
            commands,
            row,
            "−",
            lower,
            lower < ui.texas_holdem.raise_to,
            TexasRaiseAdjustButton {
                direction: -1,
                step,
                minimum: minimum_target,
                maximum: maximum_target,
            },
            assets,
        );
        add_texas_action_button(
            commands,
            row,
            &format!("加注到 {}", ui.texas_holdem.raise_to),
            TexasHoldemAction::RaiseTo(ui.texas_holdem.raise_to),
            ButtonKind::Primary,
            assets,
        );
        add_raise_adjust_button(
            commands,
            row,
            "+",
            higher,
            higher > ui.texas_holdem.raise_to,
            TexasRaiseAdjustButton {
                direction: 1,
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
        "全下",
        TexasHoldemAction::AllIn,
        ButtonKind::Warning,
        assets,
    );
}

fn add_texas_action_button(
    commands: &mut Commands,
    parent: Entity,
    label: &str,
    action: TexasHoldemAction,
    kind: ButtonKind,
    assets: &UiAssets,
) {
    add_texas_sized_button(
        commands,
        parent,
        label,
        UiAction::TexasAct(action),
        kind,
        105.0,
        assets,
    );
}

fn add_raise_adjust_button(
    commands: &mut Commands,
    parent: Entity,
    label: &str,
    target: u32,
    enabled: bool,
    repeat: TexasRaiseAdjustButton,
    assets: &UiAssets,
) {
    if enabled {
        let button = add_texas_sized_button(
            commands,
            parent,
            label,
            UiAction::SetTexasRaiseTo(target),
            ButtonKind::Primary,
            34.0,
            assets,
        );
        commands.entity(button).insert(repeat);
    } else {
        add_disabled_texas_sized_button(commands, parent, label, 34.0, assets);
    }
}

fn add_disabled_texas_sized_button(
    commands: &mut Commands,
    parent: Entity,
    label: &str,
    width: f32,
    assets: &UiAssets,
) {
    let button = commands
        .spawn((
            Node {
                width: px(width),
                min_width: px(width),
                height: px(42),
                min_height: px(42),
                padding: UiRect::axes(px(7), px(4)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border_radius: BorderRadius::all(px(6)),
                ..default()
            },
            ImageNode::new(assets.controls.secondary_button.clone())
                .with_mode(NodeImageMode::Stretch)
                .with_color(Color::srgb(0.25, 0.29, 0.28)),
            FocusPolicy::Pass,
        ))
        .id();
    commands.entity(parent).add_child(button);
    add_text(
        commands,
        button,
        label,
        15.0,
        MUTED.with_alpha(0.66),
        assets,
    );
}

fn add_texas_sized_button(
    commands: &mut Commands,
    parent: Entity,
    label: &str,
    action: UiAction,
    kind: ButtonKind,
    width: f32,
    assets: &UiAssets,
) -> Entity {
    let (image, normal, hovered, pressed) = match kind {
        ButtonKind::Primary => (
            assets.controls.primary_button.clone(),
            Color::WHITE,
            Color::srgb(1.0, 1.0, 0.82),
            Color::srgb(0.78, 0.90, 0.78),
        ),
        ButtonKind::Secondary => (
            assets.controls.secondary_button.clone(),
            Color::srgb(0.48, 0.62, 0.76),
            Color::srgb(0.64, 0.76, 0.88),
            Color::srgb(0.32, 0.46, 0.60),
        ),
        ButtonKind::Warning => (
            assets.controls.warning_button.clone(),
            Color::srgb(0.88, 0.68, 0.24),
            Color::srgb(0.98, 0.82, 0.48),
            Color::srgb(0.72, 0.54, 0.18),
        ),
        ButtonKind::Pass => (
            assets.controls.danger_button.clone(),
            Color::srgb(0.58, 0.42, 0.42),
            Color::srgb(0.76, 0.58, 0.56),
            Color::srgb(0.42, 0.28, 0.27),
        ),
    };
    let button = commands
        .spawn((
            Button,
            action,
            ButtonTint {
                normal,
                hovered,
                pressed,
            },
            Node {
                width: px(width),
                min_width: px(width),
                height: px(42),
                min_height: px(42),
                padding: UiRect::axes(px(7), px(4)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border_radius: BorderRadius::all(px(6)),
                ..default()
            },
            ImageNode::new(image)
                .with_mode(NodeImageMode::Stretch)
                .with_color(normal),
        ))
        .id();
    commands.entity(parent).add_child(button);
    let text = add_text(commands, button, label, 15.0, Color::WHITE, assets);
    commands.entity(text).insert((
        Node {
            margin: UiRect::ZERO,
            align_self: AlignSelf::Center,
            ..default()
        },
        FocusPolicy::Pass,
    ));
    button
}
