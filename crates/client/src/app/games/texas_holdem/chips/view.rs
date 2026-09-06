use super::*;
use leocard_protocol::{SeatId, TABLE_SEAT_COUNT, TexasHoldemPhaseView, TexasHoldemSnapshot};
use leocard_texas_holdem::TexasHoldemCard;

pub fn add_texas_chip_areas(
    commands: &mut Commands,
    table: Entity,
    game: &TexasHoldemSnapshot,
    hole_card_count: usize,
    state: &TexasChipTableState,
    assets: &UiAssets,
) {
    let own_seat = game
        .players
        .iter()
        .find(|player| player.id == game.you)
        .map_or(SeatId(0), |player| player.seat);
    for relative in 0..TABLE_SEAT_COUNT {
        let physical_seat = SeatId((own_seat.0 + relative) % TABLE_SEAT_COUNT);
        let player = game
            .players
            .iter()
            .find(|player| player.seat == physical_seat);
        let layout = texas_player_chip_zone(relative);
        let zone = add_chip_zone_panel(commands, table, layout);
        if let Some(player) = player {
            if let Some(label) = state.actions.get(&player.id) {
                add_chip_zone_title(commands, zone, label, assets);
                if label.kind == ActionFeedbackKind::Fold {
                    let own_cards =
                        (player.id == game.you).then_some(game.your_hole_cards.as_slice());
                    add_fold_card_feedback(
                        commands,
                        table,
                        zone,
                        label.elapsed,
                        own_cards,
                        hole_card_count,
                        assets,
                    );
                }
            }
            if matches!(game.phase, TexasHoldemPhaseView::HandComplete { .. })
                && let Some(hand) = game
                    .revealed_hands
                    .iter()
                    .find(|hand| hand.player == player.id)
            {
                add_revealed_hole_cards(commands, zone, &hand.cards, assets);
            }
        }
    }

    // One continuous dark filter frames the complete centre: draw pile, five
    // community cards, and the chips below them. There is no separate pot title.
    add_chip_zone_panel(commands, table, texas_center_zone_panel());
    add_pot_divisions(commands, table, state);
    add_pot_hover_regions(commands, table, state);

    let layer = spawn_node(
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
    commands
        .entity(layer)
        .insert((ZIndex(35), FocusPolicy::Pass));
    for chip in state.chips.iter().filter(|chip| {
        matches!(
            chip.zone,
            ChipZone::Bet(_) | ChipZone::Pot(_) | ChipZone::Retired
        ) || chip.motion.is_some()
    }) {
        add_chip_sprite(commands, layer, chip, assets);
    }
}

fn add_pot_divisions(commands: &mut Commands, table: Entity, state: &TexasChipTableState) {
    if let Some(transition) = state.division {
        spawn_pot_divider_set(
            commands,
            table,
            transition.old_count,
            true,
            transition.elapsed,
        );
        spawn_pot_divider_set(
            commands,
            table,
            transition.new_count,
            false,
            transition.elapsed,
        );
    } else {
        spawn_pot_divider_set(commands, table, state.pots.len().max(1), false, 1.0);
    }
}

fn spawn_pot_divider_set(
    commands: &mut Commands,
    table: Entity,
    count: usize,
    old_layout: bool,
    elapsed: f32,
) {
    if count <= 1 {
        return;
    }
    let whole = texas_pot_chip_zone();
    let (alpha, scale) = pot_divider_visual(old_layout, elapsed);
    for index in 1..count {
        let divider = commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: px(whole.left + whole.width * index as f32 / count as f32 - 0.5),
                    top: px(whole.top + 8.0),
                    width: px(1),
                    height: px(whole.height - 16.0),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.68, 0.72, 0.70).with_alpha(alpha * 0.42)),
                UiTransform {
                    scale: Vec2::new(1.0, scale),
                    ..UiTransform::IDENTITY
                },
                TexasPotDivider {
                    old_layout,
                    elapsed,
                },
                ZIndex(28),
                FocusPolicy::Pass,
            ))
            .id();
        commands.entity(table).add_child(divider);
    }
}

fn add_pot_hover_regions(commands: &mut Commands, table: Entity, state: &TexasChipTableState) {
    if state.pots.len() <= 1 {
        return;
    }
    for (index, pot) in state.pots.iter().enumerate() {
        let layout = texas_pot_partition_zone(index, state.pots.len());
        let region = commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: px(layout.left),
                    top: px(layout.top),
                    width: px(layout.width),
                    height: px(layout.height),
                    ..default()
                },
                RelativeCursorPosition::default(),
                TexasPotHover {
                    eligible: pot.eligible.clone(),
                },
                ZIndex(34),
                FocusPolicy::Pass,
            ))
            .id();
        commands.entity(table).add_child(region);
    }
}

pub fn pot_divider_visual(old_layout: bool, elapsed: f32) -> (f32, f32) {
    if old_layout {
        (1.0 - (elapsed / 0.16).clamp(0.0, 1.0), 1.0)
    } else {
        let progress = ((elapsed - 0.18) / 0.30).clamp(0.0, 1.0);
        let eased = 1.0 - (1.0 - progress).powi(3);
        (eased, eased)
    }
}

pub fn animate_texas_pot_dividers(
    time: Res<Time>,
    mut dividers: Query<(&mut TexasPotDivider, &mut UiTransform, &mut BackgroundColor)>,
) {
    for (mut divider, mut transform, mut background) in &mut dividers {
        divider.elapsed += time.delta_secs();
        let (alpha, scale) = pot_divider_visual(divider.old_layout, divider.elapsed);
        transform.scale.y = scale;
        background.0 = Color::srgb(0.68, 0.72, 0.70).with_alpha(alpha * 0.42);
    }
}

pub fn highlight_texas_pot_eligible_players(
    time: Res<Time>,
    hovers: Query<(&RelativeCursorPosition, &TexasPotHover)>,
    mut panels: Query<(&TexasPlayerPanel, &mut BorderColor, &mut BoxShadow)>,
) {
    let hovered = hovers
        .iter()
        .find(|(cursor, _)| cursor.cursor_over())
        .map(|(_, pot)| pot);
    let breath = pot_eligibility_breath(time.elapsed_secs());
    for (panel, mut border, mut shadow) in &mut panels {
        if hovered.is_some_and(|pot| pot.eligible.contains(&panel.player)) {
            let green = Color::srgb(
                0.28 + breath * 0.12,
                0.82 + breath * 0.18,
                0.48 + breath * 0.16,
            );
            border.set_all(green.with_alpha(0.76 + breath * 0.24));
            *shadow = BoxShadow::new(
                green.with_alpha(0.20 + breath * 0.34),
                px(0),
                px(0),
                px(1.0 + breath * 2.0),
                px(6.0 + breath * 10.0),
            );
        } else {
            border.set_all(panel.base_border);
            *shadow = BoxShadow::new(Color::NONE, px(0), px(0), px(0), px(0));
        }
    }
}

pub fn pot_eligibility_breath(elapsed: f32) -> f32 {
    let phase = elapsed.rem_euclid(1.7) / 1.7;
    let wave = 0.5 - 0.5 * (phase * std::f32::consts::TAU).cos();
    wave * wave * (3.0 - 2.0 * wave)
}

pub fn add_chip_zone_panel(
    commands: &mut Commands,
    table: Entity,
    layout: ChipZoneLayout,
) -> Entity {
    let zone = spawn_node(
        commands,
        table,
        Node {
            position_type: PositionType::Absolute,
            left: px(layout.left),
            top: px(layout.top),
            width: px(layout.width),
            height: px(layout.height),
            padding: UiRect::new(px(9), px(42), px(6), px(6)),
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(11)),
            overflow: Overflow::clip(),
            ..default()
        },
        Some(TEXAS_CHIP_ZONE_FILTER),
    );
    commands.entity(zone).insert((
        TexasChipZonePanel,
        BackgroundGradient::from(LinearGradient::to_bottom_right(vec![
            ColorStop::auto(Color::srgba(0.10, 0.16, 0.13, 0.12)),
            ColorStop::auto(Color::srgba(0.04, 0.08, 0.06, 0.08)),
            ColorStop::auto(Color::BLACK.with_alpha(0.18)),
        ])),
        BorderGradient::from(LinearGradient::to_bottom_right(vec![
            ColorStop::auto(Color::srgba(0.24, 0.34, 0.29, 0.20)),
            ColorStop::auto(Color::srgba(0.10, 0.16, 0.13, 0.14)),
            ColorStop::auto(Color::BLACK.with_alpha(0.24)),
        ])),
        BoxShadow::new(Color::BLACK.with_alpha(0.28), px(0), px(3), px(0), px(8)),
        // 区域框属于桌面底层；实际筹码在独立的高层中移动。
        ZIndex(-10),
        FocusPolicy::Pass,
    ));
    zone
}

fn add_chip_zone_title(
    commands: &mut Commands,
    zone: Entity,
    label: &ActionLabel,
    assets: &UiAssets,
) {
    let title = spawn_node(
        commands,
        zone,
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            right: px(0),
            top: px(5),
            height: px((label.font_size + 5.0).max(22.0)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        None,
    );
    commands.entity(title).insert((
        ZIndex(22),
        FocusPolicy::Pass,
        TexasActionFeedback {
            kind: label.kind,
            elapsed: label.elapsed,
        },
        action_feedback_transform(label.kind, label.elapsed),
    ));
    let text = add_text(
        commands,
        title,
        label.text,
        label.font_size,
        action_feedback_text_color(label.kind, label.color, label.elapsed),
        assets,
    );
    commands.entity(text).insert(TexasActionFeedbackText {
        color: label.color,
        kind: label.kind,
        elapsed: label.elapsed,
    });
}

pub fn add_fold_card_feedback(
    commands: &mut Commands,
    table: Entity,
    zone: Entity,
    elapsed: f32,
    own_cards: Option<&[TexasHoldemCard]>,
    hidden_card_count: usize,
    assets: &UiAssets,
) {
    let own = own_cards.is_some();
    let tooltip = own_cards.and_then(|cards| {
        (!cards.is_empty()).then(|| add_own_fold_tooltip(commands, table, cards, assets))
    });
    let layout = texas_player_chip_zone(0);
    let card_count = own_cards.map_or(hidden_card_count, <[TexasHoldemCard]>::len);
    for index in 0..card_count {
        let face = own_cards
            .and_then(|cards| cards.get(index))
            .copied()
            .map(|card| texas_card_face(card, assets));
        let visual = fold_card_visual(index, card_count, elapsed, own);
        let initial_image = if visual.face_visible {
            face.clone()
                .unwrap_or_else(|| assets.games.card_back.clone())
        } else {
            assets.games.card_back.clone()
        };
        let card = commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: if own {
                        px(layout.left + layout.width * 0.5)
                    } else {
                        percent(50)
                    },
                    top: if own { px(layout.top + 28.0) } else { px(28) },
                    width: px(34),
                    height: px(48),
                    border_radius: BorderRadius::all(px(3)),
                    ..default()
                },
                ImageNode::new(initial_image),
                visual.transform,
                TexasFoldCard {
                    index,
                    total: card_count,
                    elapsed,
                    own,
                    face,
                    back: assets.games.card_back.clone(),
                },
                ZIndex(if own { 44 } else { 24 } + index as i32),
                FocusPolicy::Pass,
            ))
            .id();
        if let Some(tooltip) = tooltip {
            commands.entity(card).insert((
                RelativeCursorPosition::default(),
                TexasOwnFoldCardHover { tooltip },
            ));
        }
        commands
            .entity(if own { table } else { zone })
            .add_child(card);
    }
}

fn add_own_fold_tooltip(
    commands: &mut Commands,
    table: Entity,
    cards: &[TexasHoldemCard],
    assets: &UiAssets,
) -> Entity {
    let layout = texas_player_chip_zone(0);
    let card_gap = 5.0;
    let card_width = 40.0;
    let content_width =
        cards.len() as f32 * card_width + cards.len().saturating_sub(1) as f32 * card_gap;
    let tooltip_width = (content_width + 14.0).max(106.0);
    let tooltip = spawn_node(
        commands,
        table,
        Node {
            position_type: PositionType::Absolute,
            left: px(layout.left + layout.width * 0.5 - tooltip_width / 2.0),
            top: px(layout.top - 79.0),
            width: px(tooltip_width),
            height: px(72),
            padding: UiRect::all(px(7)),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            column_gap: px(card_gap),
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(9)),
            ..default()
        },
        Some(HEADER_BG.with_alpha(0.96)),
    );
    commands.entity(tooltip).insert((
        Visibility::Hidden,
        TexasOwnFoldTooltip,
        BorderColor::all(MUTED.with_alpha(0.65)),
        ZIndex(80),
        FocusPolicy::Pass,
    ));
    for card in cards.iter().copied() {
        let face = commands
            .spawn((
                Node {
                    width: px(40),
                    height: px(54),
                    border_radius: BorderRadius::all(px(4)),
                    ..default()
                },
                ImageNode::new(texas_card_face(card, assets)),
                FocusPolicy::Pass,
            ))
            .id();
        commands.entity(tooltip).add_child(face);
    }
    tooltip
}

fn add_revealed_hole_cards(
    commands: &mut Commands,
    zone: Entity,
    cards: &[TexasHoldemCard],
    assets: &UiAssets,
) {
    let omaha = cards.len() > 2;
    let (width, card_width, card_height, gap) = if omaha {
        (92.0, 29.0, 40.0, -8.0)
    } else {
        (70.0, 37.0, 49.0, -7.0)
    };
    let hand = spawn_node(
        commands,
        zone,
        Node {
            position_type: PositionType::Absolute,
            left: percent(50),
            bottom: px(4),
            width: px(width),
            height: px(50),
            flex_direction: FlexDirection::Row,
            column_gap: px(gap),
            ..default()
        },
        None,
    );
    commands.entity(hand).insert((
        UiTransform::from_translation(Val2::px(-width / 2.0, 0.0)),
        ZIndex(20),
    ));
    for card in cards.iter().copied() {
        let image = texas_card_face(card, assets);
        let entity = commands
            .spawn((
                Node {
                    width: px(card_width),
                    height: px(card_height),
                    border_radius: BorderRadius::all(px(3)),
                    ..default()
                },
                ImageNode::new(image),
                FocusPolicy::Pass,
            ))
            .id();
        commands.entity(hand).add_child(entity);
    }
}

fn add_chip_sprite(commands: &mut Commands, layer: Entity, chip: &TableChip, assets: &UiAssets) {
    let Some(image) = assets.games.poker_chips.get(&chip.denomination) else {
        return;
    };
    let entity = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(chip.position.x),
                top: px(chip.position.y),
                width: px(CHIP_SIZE),
                height: px(CHIP_SIZE),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            ImageNode::new(image.clone()),
            UiTransform::from_rotation(Rot2::radians(chip.rotation)),
            TexasChipSprite(chip.id),
            ZIndex((chip.id % 200) as i32),
            FocusPolicy::Pass,
        ))
        .id();
    commands.entity(layer).add_child(entity);
    let color = if chip.denomination == 1 {
        HEADER_BG
    } else {
        TEXT
    };
    add_text(
        commands,
        entity,
        chip.denomination.to_string(),
        8.0,
        color,
        assets,
    );
}
