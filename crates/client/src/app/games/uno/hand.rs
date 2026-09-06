use super::*;

pub(super) fn add_uno_own_area(
    commands: &mut Commands,
    table: Entity,
    game: &UnoSnapshot,
    own: &UnoPlayerState,
    ui: &UiState,
    assets: &UiAssets,
    turn_border_materials: &mut Assets<TurnBorderMaterial>,
) {
    let info = add_panel(
        commands,
        table,
        Node {
            position_type: PositionType::Absolute,
            left: px(22),
            bottom: px(15),
            width: px(175),
            height: px(76),
            padding: UiRect::all(px(10)),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            row_gap: px(4),
            ..default()
        },
        PANEL.with_alpha(0.94),
        PanelSkin::Section,
        assets,
    );
    let selecting_self = matches!(
        game.pending_swap,
        Some(UnoPendingSwapView::ForceTrade { player }) if player == game.you
    );
    let self_selected = selecting_self && ui.uno.swap_targets.contains(&game.you);
    if selecting_self {
        commands.entity(info).insert((
            Button,
            UiAction::ToggleUnoSwapTarget(game.you),
            UnoSwapTargetPanel {
                selected: self_selected,
            },
            BackgroundColor(if self_selected {
                Color::BLACK.with_alpha(0.76)
            } else {
                ACCENT.mix(&PANEL, 0.48).with_alpha(0.97)
            }),
            Outline::new(px(2.0), px(1.0), ACCENT.with_alpha(0.92)),
            BoxShadow::new(ACCENT.with_alpha(0.32), px(0), px(0), px(2), px(9)),
        ));
    }
    commands.entity(info).insert(PlayerAvatarAnchor(game.you));
    if game.current_player == Some(game.you) && matches!(game.phase, UnoPhaseView::Playing) {
        add_turn_border_trace(
            commands,
            info,
            turn_border_materials,
            TurnBorderAnimationKey::new(GameKind::Uno, game.match_id, game.you),
        );
    }
    add_text(
        commands,
        info,
        format!("你 · {}", own.name),
        15.0,
        TEXT,
        assets,
    );
    if self_selected {
        add_uno_swap_selected_label(commands, info, assets);
    }
    add_uno_skip_overlay(commands, info, uno_skip_count(game, own), assets);
    add_text(
        commands,
        info,
        format!("{} 张牌", game.your_hand.len()),
        13.0,
        MUTED,
        assets,
    );

    let hand = spawn_node(
        commands,
        table,
        Node {
            position_type: PositionType::Absolute,
            left: px(205),
            right: px(205),
            bottom: px(4),
            height: px(146),
            align_items: AlignItems::FlexEnd,
            justify_content: JustifyContent::Center,
            overflow: Overflow::visible(),
            ..default()
        },
        None,
    );
    let count = game.your_hand.len();
    let reveal = if count <= 1 {
        78.0
    } else {
        (820.0 / count as f32).clamp(24.0, 72.0)
    };
    for (index, card) in game.your_hand.iter().copied().enumerate() {
        let playing = matches!(game.phase, UnoPhaseView::Playing);
        let playable = uno_card_is_playable(game, card);
        let selectable_for_swap = matches!(
            game.pending_swap,
            Some(UnoPendingSwapView::SwapOneGive { player, .. }) if player == game.you
        );
        let interactive = playable || selectable_for_swap;
        let jump_selected = game.your_jump_in_card == Some(card);
        let selected = playing && ui.uno.selected.contains(&card);
        let animation = if playing {
            ui.uno
                .card_animations
                .get(&card)
                .copied()
                .unwrap_or_default()
        } else {
            CardAnimationState::default()
        };
        let mut entity = commands.spawn((
            Node {
                width: px(if index + 1 == count { 78.0 } else { reveal }),
                height: px(126),
                flex_shrink: 0.0,
                align_items: AlignItems::FlexStart,
                overflow: Overflow::visible(),
                ..default()
            },
            BackgroundColor(Color::NONE),
        ));
        let extension_help = uno_extension_card_help(card.face());
        if interactive {
            entity.insert((
                Button,
                UnoHandCardButton,
                UiAction::ToggleUnoCard(card),
                ButtonTint {
                    normal: Color::WHITE,
                    hovered: Color::srgb(1.0, 0.92, 0.66),
                    pressed: Color::srgb(0.78, 0.84, 0.72),
                },
            ));
        } else if extension_help.is_some() {
            entity.insert(Button);
        }
        let slot = entity.id();
        commands.entity(hand).add_child(slot);
        let face = commands
            .spawn((
                UnoFlipTarget::Own(index),
                UnoHandCardVisual {
                    button: slot,
                    card,
                    selected,
                    hover_amount: animation.face_hover_amount,
                    selected_amount: animation.selected_amount,
                },
                Node {
                    width: px(78),
                    height: px(122),
                    border: UiRect::all(px(2)),
                    border_radius: BorderRadius::all(px(6)),
                    ..default()
                },
                UiTransform::from_translation(Val2::px(
                    0.0,
                    -(animation.face_hover_amount * 10.0 + animation.selected_amount * 22.0),
                )),
                ImageNode::new(uno_card_handle(assets, card)).with_color(
                    if interactive || jump_selected || !playing {
                        Color::WHITE
                    } else {
                        Color::srgba(0.58, 0.58, 0.58, 0.86)
                    },
                ),
                BoxShadow::new(Color::BLACK.with_alpha(0.42), px(2), px(4), px(0), px(5)),
                BorderColor::all(if selected { ACCENT } else { Color::NONE }),
                Outline {
                    width: px(if selected { 2.0 } else { 0.0 }),
                    offset: px(0),
                    color: ACCENT.with_alpha(0.8),
                },
                FocusPolicy::Pass,
            ))
            .id();
        commands.entity(slot).add_child(face);
        if let Some((title, description)) = extension_help {
            commands
                .entity(slot)
                .insert(UnoExtensionCardHelp { title, description });
        }
    }
}

pub const fn uno_extension_card_help(face: UnoFace) -> Option<(&'static str, &'static str)> {
    match face {
        UnoFace::SwapOne => Some((
            "交换一张",
            "随机取得一名玩家的一张牌，再从当前手牌中选择一张交还。",
        )),
        UnoFace::RefreshHand => {
            Some(("刷新手牌", "将剩余手牌放到弃牌堆底部，再摸取相同数量的牌。"))
        }
        UnoFace::WildForceTrade => {
            Some(("指定换手", "选择两名玩家交换全部手牌，完成后选择后续颜色。"))
        }
        UnoFace::WildPassHands => Some((
            "顺序传手",
            "所有玩家按当前方向传递全部手牌，完成后选择后续颜色。",
        )),
        UnoFace::ReverseDrawTwo => Some((
            "反转摸二",
            "立即改变方向，并让新方向的下一名玩家累计摸两张牌。",
        )),
        UnoFace::ReverseSkip => Some((
            "反转禁手",
            "立即改变方向，并让新方向的下一名玩家被禁手一轮。",
        )),
        UnoFace::WildPowerReverse => Some((
            "强力反转",
            "改变方向并选择后续颜色，然后由你立即再行动一次。",
        )),
        UnoFace::WildNoU => Some((
            "罚牌反弹",
            "受到摸牌惩罚时将累计罚牌退给上一名罚牌者；平时作为万能反转牌使用。",
        )),
        UnoFace::StackOne => Some((
            "堆叠 +1",
            "按当前颜色打出，使下家累计摸一张；罚牌链中也必须匹配当前颜色。",
        )),
        UnoFace::StackTwo => Some((
            "堆叠 +2",
            "按当前颜色打出，使下家累计摸两张；罚牌链中也必须匹配当前颜色。",
        )),
        UnoFace::WildStackThree => {
            Some(("万能堆叠 +3", "可随时打出并选择颜色，使下家累计摸三张。"))
        }
        UnoFace::WildStackNumber => Some((
            "万能随机堆叠",
            "选择颜色后从摸牌堆翻牌，直到出现数字牌，并将该数字加入累计罚牌。",
        )),
        UnoFace::DrawFour => Some(("摸四", "使下一名玩家累计摸四张；可压在 +2 或 +4 上。")),
        UnoFace::DrawFive => Some((
            "摸五",
            "使下一名玩家累计摸五张；启用功能牌堆叠时可继续累计。",
        )),
        UnoFace::SkipEveryone => Some(("跳过所有人", "跳过所有其他玩家，由你立即再行动一次。")),
        UnoFace::Flip => Some((
            "翻面",
            "翻转摸牌堆、弃牌堆和所有玩家手牌，并改用另一面的牌面继续游戏。",
        )),
        UnoFace::WildDrawColor => Some((
            "指定颜色摸牌",
            "选择一种颜色；下一名玩家持续摸牌，直到摸到该颜色。",
        )),
        UnoFace::DiscardAll => Some(("全部弃牌", "同时弃掉手中所有与此牌同色的牌。")),
        UnoFace::WildReverseDrawFour => Some((
            "反转摸四",
            "改变方向，选择颜色，并让新方向的下一名玩家累计摸四张。",
        )),
        UnoFace::WildDrawSix => Some(("万能摸六", "选择颜色，使下一名玩家累计摸六张。")),
        UnoFace::WildDrawTen => Some(("万能摸十", "选择颜色，使下一名玩家累计摸十张。")),
        UnoFace::WildColorRoulette => Some((
            "颜色轮盘",
            "下一名玩家选择颜色，持续翻牌并收下所有牌，直到出现该颜色。",
        )),
        UnoFace::Number(_)
        | UnoFace::DrawOne
        | UnoFace::DrawTwo
        | UnoFace::Reverse
        | UnoFace::Skip
        | UnoFace::Wild
        | UnoFace::DarkWild
        | UnoFace::WildDrawTwo
        | UnoFace::WildDrawFour => None,
    }
}

fn spawn_uno_extension_card_tooltip(
    commands: &mut Commands,
    layer: Entity,
    assets: &UiAssets,
) -> Entity {
    let tooltip = spawn_node(
        commands,
        layer,
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            top: px(0),
            width: px(198),
            padding: UiRect::axes(px(11), px(9)),
            flex_direction: FlexDirection::Column,
            row_gap: px(4),
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(8)),
            ..default()
        },
        Some(PANEL.with_alpha(0.78)),
    );
    commands.entity(tooltip).insert((
        Visibility::Hidden,
        BorderColor::all(ACCENT.with_alpha(0.24)),
        BoxShadow::new(Color::BLACK.with_alpha(0.30), px(2), px(4), px(0), px(8)),
        GlobalZIndex(1900),
        FocusPolicy::Pass,
    ));
    let title = add_text(commands, tooltip, "", 13.0, TEXT.with_alpha(0.90), assets);
    let description = add_text(commands, tooltip, "", 11.5, MUTED.with_alpha(0.88), assets);
    commands
        .entity(tooltip)
        .insert(UnoExtensionCardHelpOverlay { title, description });
    tooltip
}

pub fn sync_uno_extension_card_help(
    mut commands: Commands,
    assets: Res<UiAssets>,
    cards: Query<(
        &Interaction,
        &UnoExtensionCardHelp,
        &ComputedNode,
        &UiGlobalTransform,
    )>,
    layers: Query<(Entity, &ComputedNode, &UiGlobalTransform), With<PlayerInteractionLayer>>,
    mut overlays: Query<(&UnoExtensionCardHelpOverlay, &mut Node, &mut Visibility)>,
    mut texts: Query<&mut Text>,
) {
    let Ok((layer, layer_node, layer_transform)) = layers.single() else {
        return;
    };
    let Ok((overlay, mut node, mut visibility)) = overlays.single_mut() else {
        spawn_uno_extension_card_tooltip(&mut commands, layer, &assets);
        return;
    };
    let Some((_, help, card_node, card_transform)) = cards.iter().find(|(interaction, ..)| {
        matches!(interaction, Interaction::Hovered | Interaction::Pressed)
    }) else {
        *visibility = Visibility::Hidden;
        return;
    };
    let Some(center) = uno_anchor_in_layer(card_node, card_transform, layer_node, layer_transform)
    else {
        *visibility = Visibility::Hidden;
        return;
    };
    let size = card_node.size() * card_node.inverse_scale_factor();
    node.left = px(center.x - size.x * 0.5 + 84.0);
    node.top = px(center.y - size.y * 0.5 + 12.0);
    if let Ok(mut title) = texts.get_mut(overlay.title)
        && title.0 != help.title
    {
        title.0 = help.title.to_owned();
    }
    if let Ok(mut description) = texts.get_mut(overlay.description)
        && description.0 != help.description
    {
        description.0 = help.description.to_owned();
    }
    *visibility = Visibility::Visible;
}
