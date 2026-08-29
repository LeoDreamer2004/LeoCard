//! Local input, scaling, card selection, and OS-backed image picking.

use super::*;

pub(super) fn update_ui_scale(
    windows: Query<&Window, With<PrimaryWindow>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut zoom: ResMut<UiZoom>,
    mut scale: ResMut<UiScale>,
    mut ui: ResMut<UiState>,
) {
    let ctrl = keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight);
    if ctrl && (keyboard.just_pressed(KeyCode::Equal) || keyboard.just_pressed(KeyCode::NumpadAdd))
    {
        zoom.manual = (zoom.manual + 0.1).min(MAX_MANUAL_ZOOM);
        ui.dirty = true;
    }
    if ctrl
        && (keyboard.just_pressed(KeyCode::Minus) || keyboard.just_pressed(KeyCode::NumpadSubtract))
    {
        zoom.manual = (zoom.manual - 0.1).max(MIN_MANUAL_ZOOM);
        ui.dirty = true;
    }
    if ctrl && keyboard.just_pressed(KeyCode::Digit0) {
        zoom.manual = 1.0;
        ui.dirty = true;
    }

    let Ok(window) = windows.single() else {
        return;
    };
    let effective = calculate_ui_scale(window.width(), window.height(), zoom.manual);
    if (scale.0 - effective).abs() > 0.005 {
        scale.0 = effective;
        ui.dirty = true;
    }
}

pub(super) fn calculate_ui_scale(width: f32, height: f32, manual_zoom: f32) -> f32 {
    let automatic = (width / DESIGN_WIDTH)
        .min(height / DESIGN_HEIGHT)
        .clamp(MIN_AUTO_SCALE, MAX_AUTO_SCALE);
    automatic * manual_zoom.clamp(MIN_MANUAL_ZOOM, MAX_MANUAL_ZOOM)
}

/// Winit only forwards composition and commit messages from a system input
/// method while IME support is enabled on the window. Keep it active for the
/// player-name field and chat, but disable it for numeric/address/developer
/// inputs so those fields continue to receive exact key presses.
pub(super) fn sync_ime_enabled(
    client: Option<Res<ClientResource>>,
    form: Res<ConnectionForm>,
    chat: Res<ChatPanelState>,
    developer_hand: Res<DeveloperHandInput>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    let Ok(mut window) = windows.single_mut() else {
        return;
    };
    let connection_visible = client.as_deref().is_none_or(|client| {
        client.0.model().lobby().is_none()
            && client.0.model().qigui523_game().is_none()
            && client.0.model().texas_holdem_game().is_none()
            && client.0.model().shengji_game().is_none()
    });
    let enabled = !developer_hand.focused
        && (chat.focused || (connection_visible && form.active == InputField::PlayerName));
    if window.ime_enabled != enabled {
        window.ime_enabled = enabled;
    }
    if enabled {
        window.ime_position = if chat.focused {
            Vec2::new(
                (window.width() - 310.0).max(0.0),
                (window.height() - 54.0).max(0.0),
            )
        } else {
            Vec2::new(window.width() * 0.5, window.height() * 0.24)
        };
    }
}

pub(super) fn update_button_tints(
    mut image_buttons: Query<
        (&Interaction, &ButtonTint, &mut ImageNode),
        (Changed<Interaction>, Without<BackgroundButtonTint>),
    >,
    mut background_buttons: Query<
        (&Interaction, &ButtonTint, &mut BackgroundColor),
        (Changed<Interaction>, With<BackgroundButtonTint>),
    >,
) {
    for (interaction, tint, mut image) in &mut image_buttons {
        image.color = match interaction {
            Interaction::None => tint.normal,
            Interaction::Hovered => tint.hovered,
            Interaction::Pressed => tint.pressed,
        };
    }
    for (interaction, tint, mut background) in &mut background_buttons {
        background.0 = match interaction {
            Interaction::None => tint.normal,
            Interaction::Hovered => tint.hovered,
            Interaction::Pressed => tint.pressed,
        };
    }
}

pub(super) fn play_button_click_sounds(
    buttons: Query<
        &Interaction,
        (
            Changed<Interaction>,
            With<Button>,
            Without<HandCardSlot>,
            Without<UnoHandCardButton>,
        ),
    >,
    assets: Res<UiAssets>,
    mut commands: Commands,
) {
    if assets.button_click_sounds.is_empty() {
        return;
    }
    for interaction in &buttons {
        if !matches!(interaction, Interaction::Pressed) {
            continue;
        }
        let sound =
            assets.button_click_sounds[fastrand::usize(..assets.button_click_sounds.len())].clone();
        commands.spawn((AudioPlayer::new(sound), PlaybackSettings::DESPAWN));
    }
}

pub(super) fn animate_button_presses(
    time: Res<Time>,
    changed: Query<
        (Entity, &Interaction),
        (
            Changed<Interaction>,
            With<Button>,
            Without<HandCardSlot>,
            Without<ShengjiHandCardSlot>,
        ),
    >,
    mut buttons: Query<
        (&Interaction, &mut UiTransform),
        (
            With<Button>,
            Without<HandCardSlot>,
            Without<ShengjiHandCardSlot>,
        ),
    >,
    mut active: Local<HashSet<Entity>>,
) {
    active.extend(changed.iter().map(|(entity, _)| entity));
    if active.is_empty() {
        return;
    }
    let response = 1.0 - (-time.delta_secs() * 28.0).exp();
    let entities = active.iter().copied().collect::<Vec<_>>();
    for entity in entities {
        let Ok((interaction, mut transform)) = buttons.get_mut(entity) else {
            active.remove(&entity);
            continue;
        };
        let (target_scale, target_y) = match interaction {
            Interaction::Pressed => (0.97, 1.5),
            Interaction::Hovered => (1.015, 0.0),
            Interaction::None => (1.0, 0.0),
        };
        let scale = transform.scale.x + (target_scale - transform.scale.x) * response;
        let current_y = match transform.translation.y {
            Val::Px(value) => value,
            _ => 0.0,
        };
        let y = current_y + (target_y - current_y) * response;
        if (scale - target_scale).abs() < 0.000_5 && (y - target_y).abs() < 0.01 {
            if transform.scale.x != target_scale || current_y != target_y {
                transform.scale = Vec2::splat(target_scale);
                transform.translation.y = px(target_y);
            }
            active.remove(&entity);
        } else {
            transform.scale = Vec2::splat(scale);
            transform.translation.y = px(y);
        }
    }
}

pub(super) fn handle_texas_raise_button_hold(
    time: Res<Time>,
    mouse: Res<ButtonInput<MouseButton>>,
    changed: Query<(&Interaction, &TexasRaiseAdjustButton), (Changed<Interaction>, With<Button>)>,
    mut hold: ResMut<TexasRaiseHoldState>,
    mut ui: ResMut<UiState>,
) {
    if mouse.just_pressed(MouseButton::Left) {
        if let Some((_, button)) = changed
            .iter()
            .find(|(interaction, _)| **interaction == Interaction::Pressed)
        {
            *hold = TexasRaiseHoldState {
                direction: button.direction,
                step: button.step,
                minimum: button.minimum,
                maximum: button.maximum,
                elapsed: 0.0,
                next_repeat: 0.42,
            };
        }
        return;
    }
    if mouse.just_released(MouseButton::Left) || !mouse.pressed(MouseButton::Left) {
        *hold = TexasRaiseHoldState::default();
        return;
    }
    if hold.direction == 0 {
        return;
    }

    hold.elapsed += time.delta_secs();
    while hold.elapsed >= hold.next_repeat {
        let current = ui.texas_raise_to;
        let next = texas_raise_repeat_value(
            current,
            hold.direction,
            hold.step,
            hold.minimum,
            hold.maximum,
        );
        if next == current {
            hold.direction = 0;
            break;
        }
        ui.texas_raise_to = next;
        ui.dirty = true;
        hold.next_repeat += if hold.elapsed >= 1.35 { 0.065 } else { 0.11 };
    }
}

pub(super) fn texas_raise_repeat_value(
    current: u32,
    direction: i8,
    step: u32,
    minimum: u32,
    maximum: u32,
) -> u32 {
    if direction < 0 {
        current.saturating_sub(step).max(minimum)
    } else if direction > 0 {
        current.saturating_add(step).min(maximum)
    } else {
        current.clamp(minimum, maximum)
    }
}

pub(super) fn animate_lobby_seat_hover(
    time: Res<Time>,
    mut seats: Query<(&Interaction, &mut LobbySeatHover)>,
    mut visuals: Query<(&LobbySeatVisual, &mut UiTransform)>,
    mut empty_rings: Query<(&LobbyEmptySeatRing, &mut BorderColor, &mut BackgroundColor)>,
    mut empty_labels: Query<(&LobbyEmptySeatLabel, &mut Text, &mut TextColor)>,
) {
    let response = 1.0 - (-time.delta_secs() * 16.0).exp();
    for (interaction, mut hover) in &mut seats {
        let hovered = !matches!(interaction, Interaction::None);
        let target = if hovered { 1.0 } else { 0.0 };
        hover.amount += (target - hover.amount) * response;
        if (hover.amount - target).abs() < 0.002 {
            hover.amount = target;
        }
        let direction = match hover.seat {
            0 => Vec2::new(0.2, -1.0),
            1 => Vec2::new(1.0, 0.0),
            2 => Vec2::new(0.2, 1.0),
            3 => Vec2::new(-0.2, 1.0),
            4 => Vec2::new(-1.0, 0.0),
            5 => Vec2::new(-0.2, -1.0),
            _ => Vec2::ZERO,
        }
        .normalize_or_zero();
        if let Some((_, mut transform)) = visuals
            .iter_mut()
            .find(|(visual, _)| visual.0 == hover.seat)
        {
            transform.translation = Val2::px(
                direction.x * hover.amount * 5.0,
                direction.y * hover.amount * 5.0,
            );
            transform.scale = Vec2::splat(1.0 + hover.amount * 0.035);
        }
        if let Some((_, mut border, mut background)) = empty_rings
            .iter_mut()
            .find(|(ring, _, _)| ring.0 == hover.seat)
        {
            border.set_all(if hovered {
                ACCENT.with_alpha(0.88)
            } else {
                MUTED.with_alpha(0.34)
            });
            background.0 = if hovered {
                PANEL_ALT.with_alpha(0.90)
            } else {
                Color::srgba(0.03, 0.12, 0.085, 0.64)
            };
        }
        if let Some((_, mut text, mut color)) = empty_labels
            .iter_mut()
            .find(|(label, _, _)| label.0 == hover.seat)
        {
            let expected_text = if hovered { "点击入座" } else { "空位" };
            let expected_color = if hovered { ACCENT } else { MUTED };
            if text.0 != expected_text {
                text.0 = expected_text.to_owned();
            }
            if color.0 != expected_color {
                color.0 = expected_color;
            }
        }
    }
}

pub(super) fn handle_lobby_bot_seat_right_click(
    mouse: Res<ButtonInput<MouseButton>>,
    seats: Query<(&Interaction, &LobbySeatHover)>,
    mut client: Option<ResMut<ClientResource>>,
) {
    #[cfg(not(feature = "developer"))]
    let _ = (&mouse, &seats, &mut client);
    #[cfg(feature = "developer")]
    {
        if !mouse.just_pressed(MouseButton::Right) {
            return;
        }
        let Some(seat) = seats
            .iter()
            .find(|(interaction, _)| !matches!(interaction, Interaction::None))
            .map(|(_, hover)| SeatId(hover.seat))
        else {
            return;
        };
        let Some(client) = client.as_deref_mut() else {
            return;
        };
        let occupied = {
            let model = client.0.model();
            let Some(lobby) = model.lobby() else {
                return;
            };
            if model.you() != lobby.host {
                return;
            }
            match lobby
                .players
                .iter()
                .find(|player| player.seat == Some(seat))
            {
                None => true,
                Some(player) if player.profile_id.0 == [0; 32] => false,
                Some(_) => return,
            }
        };
        client
            .0
            .send(ClientCommand::ConfigureBotSeat { seat, occupied });
    }
}

pub(super) fn update_rule_help_tooltips(
    helps: Query<(&Interaction, &RuleHelp), Changed<Interaction>>,
    mut tooltips: Query<&mut Visibility>,
) {
    for (interaction, help) in &helps {
        if let Ok(mut visibility) = tooltips.get_mut(help.tooltip) {
            *visibility = if matches!(interaction, Interaction::Hovered | Interaction::Pressed) {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
        }
    }
}

pub(super) fn table_brightness_fraction(brightness: f32) -> f32 {
    ((normalize_table_brightness(brightness) - MIN_TABLE_BRIGHTNESS)
        / (MAX_TABLE_BRIGHTNESS - MIN_TABLE_BRIGHTNESS))
        .clamp(0.0, 1.0)
}

pub(super) fn slider_fraction_from_relative_x(relative_x: f32) -> f32 {
    // Bevy 的 RelativeCursorPosition 以节点中心为 0，左右边缘分别为 -0.5 和 0.5。
    (relative_x + 0.5).clamp(0.0, 1.0)
}

pub(super) fn table_appearance_fraction(
    setting: TableAppearanceSetting,
    form: &ConnectionForm,
) -> f32 {
    match setting {
        TableAppearanceSetting::Brightness => table_brightness_fraction(form.table_brightness),
        TableAppearanceSetting::Vignette => ((form.table_vignette - MIN_TABLE_VIGNETTE)
            / (MAX_TABLE_VIGNETTE - MIN_TABLE_VIGNETTE))
            .clamp(0.0, 1.0),
        TableAppearanceSetting::Volume => form.audio_volume,
    }
}

pub(super) fn table_appearance_label(
    setting: TableAppearanceSetting,
    form: &ConnectionForm,
) -> String {
    match setting {
        TableAppearanceSetting::Brightness => {
            format!("亮度  {:.0}%", form.table_brightness * 100.0)
        }
        TableAppearanceSetting::Vignette => {
            format!("四周视角阴影  {:.0}%", form.table_vignette * 100.0)
        }
        TableAppearanceSetting::Volume => format!("音量  {:.0}%", form.audio_volume * 100.0),
    }
}

fn set_table_appearance_from_fraction(
    setting: TableAppearanceSetting,
    fraction: f32,
    form: &mut ConnectionForm,
) {
    match setting {
        TableAppearanceSetting::Brightness => {
            form.table_brightness =
                MIN_TABLE_BRIGHTNESS + fraction * (MAX_TABLE_BRIGHTNESS - MIN_TABLE_BRIGHTNESS);
        }
        TableAppearanceSetting::Vignette => {
            form.table_vignette =
                MIN_TABLE_VIGNETTE + fraction * (MAX_TABLE_VIGNETTE - MIN_TABLE_VIGNETTE);
        }
        TableAppearanceSetting::Volume => {
            form.audio_volume = fraction;
        }
    }
}

pub(super) fn handle_table_appearance_sliders(
    mouse: Res<ButtonInput<MouseButton>>,
    sliders: Query<(
        &Interaction,
        &RelativeCursorPosition,
        &TableAppearanceSlider,
    )>,
    mut indicators: Query<(&TableAppearanceIndicator, &mut Node)>,
    mut labels: Query<(&TableAppearanceLabel, &mut Text)>,
    backgrounds: Query<&MaterialNode<TableBackgroundMaterial>, With<TableBackground>>,
    mut materials: ResMut<Assets<TableBackgroundMaterial>>,
    mut global_volume: ResMut<GlobalVolume>,
    mut form: ResMut<ConnectionForm>,
    mut dragging: Local<Option<TableAppearanceSetting>>,
) {
    if mouse.just_pressed(MouseButton::Left) {
        *dragging = sliders.iter().find_map(|(interaction, _, slider)| {
            (*interaction == Interaction::Pressed).then_some(slider.0)
        });
    }

    if let Some(setting) = *dragging
        && mouse.pressed(MouseButton::Left)
        && let Some(position) = sliders
            .iter()
            .find(|(_, _, slider)| slider.0 == setting)
            .and_then(|(_, cursor, _)| cursor.normalized.map(|position| position.x))
    {
        let fraction = slider_fraction_from_relative_x(position);
        let before = table_appearance_fraction(setting, &form);
        if (before - fraction).abs() > 0.001 {
            set_table_appearance_from_fraction(setting, fraction, &mut form);
            for (indicator, mut node) in &mut indicators {
                if indicator.setting != setting {
                    continue;
                }
                match indicator.part {
                    TableAppearanceIndicatorPart::Fill => node.width = percent(fraction * 100.0),
                    TableAppearanceIndicatorPart::Knob => node.left = percent(fraction * 100.0),
                }
            }
            for (label, mut text) in &mut labels {
                if label.0 == setting {
                    text.0 = table_appearance_label(setting, &form);
                }
            }
            for background in &backgrounds {
                if let Some(mut material) = materials.get_mut(background) {
                    material.params.x = form.table_vignette;
                    material.params.y = form.table_brightness;
                }
            }
            global_volume.volume = Volume::Linear(form.audio_volume);
        }
    }

    if dragging.is_some() && mouse.just_released(MouseButton::Left) {
        *dragging = None;
        if let Err(error) = save_preferences(&form) {
            warn!("{error}");
        }
    }
}

pub(super) fn animate_hand_card_slots(
    time: Res<Time>,
    drag: Res<CardDragSelection>,
    mut ui: ResMut<UiState>,
    mut cards: Query<
        (
            &Interaction,
            &RelativeCursorPosition,
            &mut HandCardSlot,
            &mut Node,
        ),
        With<Button>,
    >,
) {
    let response = 1.0 - (-18.0 * time.delta_secs()).exp();
    for (interaction, cursor, mut slot, mut node) in &mut cards {
        let hovered = if drag.active {
            cursor.cursor_over()
        } else {
            matches!(*interaction, Interaction::Hovered | Interaction::Pressed)
        };
        let target = f32::from(hovered);
        if (target - slot.hover_amount).abs() < 0.001 {
            continue;
        }
        let next = slot.hover_amount + (target - slot.hover_amount) * response;
        slot.hover_amount = if (target - next).abs() < 0.001 {
            target
        } else {
            next
        };
        ui.card_animations
            .entry(slot.card)
            .or_default()
            .slot_hover_amount = slot.hover_amount;
        node.width = px(if slot.is_last {
            CardSize::Hand.dimensions().0
        } else {
            HAND_CARD_REVEAL + (HAND_CARD_HOVER_WIDTH - HAND_CARD_REVEAL) * slot.hover_amount
        });
    }
}

pub(super) fn animate_shengji_hand_card_slots(
    time: Res<Time>,
    drag: Res<ShengjiCardDragSelection>,
    mut ui: ResMut<UiState>,
    mut cards: Query<
        (
            &Interaction,
            &RelativeCursorPosition,
            &mut ShengjiHandCardSlot,
            &mut Node,
        ),
        With<Button>,
    >,
) {
    let response = 1.0 - (-18.0 * time.delta_secs()).exp();
    for (interaction, cursor, mut slot, mut node) in &mut cards {
        let hovered = if drag.active {
            cursor.cursor_over()
        } else {
            matches!(*interaction, Interaction::Hovered | Interaction::Pressed)
        };
        let target = f32::from(hovered);
        let next = slot.hover_amount + (target - slot.hover_amount) * response;
        slot.hover_amount = if (target - next).abs() < 0.001 {
            target
        } else {
            next
        };
        ui.shengji_card_animations
            .entry(slot.card)
            .or_default()
            .slot_hover_amount = slot.hover_amount;
        node.width = px(if slot.is_last {
            CardSize::Hand.dimensions().0
        } else {
            shengji_hand_card_reveal(slot.hand_len)
                + (HAND_CARD_HOVER_WIDTH - shengji_hand_card_reveal(slot.hand_len))
                    * slot.hover_amount
        });
    }
}

pub(super) fn handle_card_drag_selection(
    mouse: Res<ButtonInput<MouseButton>>,
    mut drag: ResMut<CardDragSelection>,
    mut ui: ResMut<UiState>,
    cards: Query<(&Interaction, &RelativeCursorPosition, &HandCardSlot)>,
) {
    if mouse.just_pressed(MouseButton::Left)
        && let Some((_, _, slot)) = cards
            .iter()
            .find(|(interaction, _, _)| **interaction == Interaction::Pressed)
    {
        drag.active = true;
        drag.anchor = slot.index;
        drag.current = slot.index;
        drag.select = !ui.selected.contains(&slot.card);
    }

    if drag.active
        && mouse.pressed(MouseButton::Left)
        && let Some((_, _, slot)) = cards
            .iter()
            .filter(|(_, cursor, _)| cursor.cursor_over())
            .max_by_key(|(_, _, slot)| slot.index)
    {
        drag.current = slot.index;
    }

    if drag.active && mouse.just_released(MouseButton::Left) {
        for (_, _, slot) in &cards {
            if drag.contains(slot.index) {
                if drag.select {
                    ui.selected.insert(slot.card);
                } else {
                    ui.selected.remove(&slot.card);
                }
            }
        }
        drag.active = false;
    }
}

pub(super) fn handle_shengji_card_drag_selection(
    mouse: Res<ButtonInput<MouseButton>>,
    mut drag: ResMut<ShengjiCardDragSelection>,
    mut ui: ResMut<UiState>,
    cards: Query<(&Interaction, &RelativeCursorPosition, &ShengjiHandCardSlot)>,
) {
    if mouse.just_pressed(MouseButton::Left)
        && let Some((_, _, slot)) = cards
            .iter()
            .find(|(interaction, _, _)| **interaction == Interaction::Pressed)
    {
        drag.active = true;
        drag.anchor = slot.index;
        drag.current = slot.index;
        drag.select = !ui.selected_shengji.contains(&slot.card);
    }
    if drag.active
        && mouse.pressed(MouseButton::Left)
        && let Some((_, _, slot)) = cards
            .iter()
            .filter(|(_, cursor, _)| cursor.cursor_over())
            .max_by_key(|(_, _, slot)| slot.index)
    {
        drag.current = slot.index;
    }
    if drag.active && mouse.just_released(MouseButton::Left) {
        let mut changed = false;
        for (_, _, slot) in &cards {
            if drag.contains(slot.index) {
                if drag.select {
                    changed |= ui.selected_shengji.insert(slot.card);
                } else {
                    changed |= ui.selected_shengji.remove(&slot.card);
                }
            }
        }
        drag.active = false;
        // 双升的出牌/埋底按钮会根据已选张数重建；选择变化必须立即刷新，
        // 否则牌面虽已抬起，按钮仍停留在“请选择要出的牌”。
        ui.dirty |= changed;
    }
}

pub(super) fn sync_card_drag_preview(
    drag: Res<CardDragSelection>,
    mut overlays: Query<(&HandCardSelectionOverlay, &mut BackgroundColor)>,
) {
    if !drag.is_changed() {
        return;
    }
    for (overlay, mut color) in &mut overlays {
        let expected = if drag.contains(overlay.index) {
            Color::srgba(0.12, 0.14, 0.14, 0.52)
        } else {
            Color::NONE
        };
        if color.0 != expected {
            color.0 = expected;
        }
    }
}

pub(super) fn sync_shengji_card_drag_preview(
    drag: Res<ShengjiCardDragSelection>,
    mut overlays: Query<(&ShengjiHandCardSelectionOverlay, &mut BackgroundColor)>,
) {
    if !drag.is_changed() {
        return;
    }
    for (overlay, mut color) in &mut overlays {
        let expected = if drag.contains(overlay.index) {
            Color::srgba(0.12, 0.14, 0.14, 0.52)
        } else {
            Color::NONE
        };
        if color.0 != expected {
            color.0 = expected;
        }
    }
}

pub(super) fn animate_hand_cards(
    time: Res<Time>,
    drag: Res<CardDragSelection>,
    mut ui: ResMut<UiState>,
    buttons: Query<&Interaction, With<Button>>,
    mut cards: HandCardAnimations,
) {
    let response = 1.0 - (-14.0 * time.delta_secs()).exp();
    let pulse = 0.72 + 0.28 * (time.elapsed_secs() * 7.0).sin();

    for (mut visual, mut transform, mut outline, mut shadow, mut image, mut border) in &mut cards {
        let Ok(interaction) = buttons.get(visual.button) else {
            continue;
        };
        let selected = ui.selected.contains(&visual.card);
        let hovered = if drag.active {
            visual.index == drag.current
        } else {
            matches!(*interaction, Interaction::Hovered | Interaction::Pressed)
        };
        let hover_target = f32::from(hovered);
        let selected_target = f32::from(selected);
        let transitioning = (hover_target - visual.hover_amount).abs() >= 0.001
            || (selected_target - visual.selected_amount).abs() >= 0.001
            || visual.selected != selected
            || visual.dealing;
        // 静止且未选中的牌无需每帧重写七个 UI 组件。选中/悬停牌仍保留呼吸光效。
        if !transitioning && !hovered && !selected {
            continue;
        }
        visual.selected = selected;
        let next_hover = visual.hover_amount + (hover_target - visual.hover_amount) * response;
        visual.hover_amount = if (hover_target - next_hover).abs() < 0.001 {
            hover_target
        } else {
            next_hover
        };
        let next_selected =
            visual.selected_amount + (selected_target - visual.selected_amount) * response;
        visual.selected_amount = if (selected_target - next_selected).abs() < 0.001 {
            selected_target
        } else {
            next_selected
        };
        if visual.dealing {
            visual.deal_elapsed += time.delta_secs();
            if visual.deal_elapsed >= 0.20 {
                visual.dealing = false;
            }
        }
        let next_animation = CardAnimationState {
            face_hover_amount: visual.hover_amount,
            selected_amount: visual.selected_amount,
            deal_elapsed: visual.deal_elapsed,
            dealing: visual.dealing,
            ..ui.card_animations
                .get(&visual.card)
                .copied()
                .unwrap_or_default()
        };
        if ui.card_animations.get(&visual.card).copied() != Some(next_animation) {
            ui.card_animations.insert(visual.card, next_animation);
        }

        let hover = visual.hover_amount;
        let selected = visual.selected_amount;
        let glow = (hover * pulse + selected * 0.72).clamp(0.0, 1.0);
        let pose = hand_card_pose(
            visual.index,
            visual.hand_len,
            hover,
            selected,
            visual.deal_elapsed,
            visual.dealing,
        );

        transform.scale = Vec2::ONE;
        transform.translation = pose.translation;
        transform.rotation = pose.rotation;
        outline.width = px(0.75 + glow * 1.5);
        outline.offset = px(0);
        outline.color = ACCENT.with_alpha(glow * 0.92);
        image.color = Color::srgb(1.0, 1.0 - glow * 0.035, 1.0 - glow * 0.16);
        border.set_all(if visual.selected { ACCENT } else { BORDER });

        if let Some(style) = shadow.0.first_mut() {
            style.color = ACCENT.with_alpha(glow * 0.58);
            style.spread_radius = px(glow * 2.0);
            style.blur_radius = px(2.0 + glow * 10.0);
        }
    }
}

pub(super) fn hand_card_pose(
    index: usize,
    hand_len: usize,
    hover_amount: f32,
    selected_amount: f32,
    deal_elapsed: f32,
    dealing: bool,
) -> HandCardPose {
    let lift = hover_amount * 5.0 + selected_amount * HAND_CARD_SELECTED_LIFT;
    let deal_progress = if dealing {
        (deal_elapsed / 0.20).clamp(0.0, 1.0)
    } else {
        1.0
    };
    let deal_progress = 1.0 - (1.0 - deal_progress).powi(3);
    let deal_offset = 1.0 - deal_progress;
    let reveal = shengji_hand_card_reveal(hand_len);
    let hand_width = if hand_len == 0 {
        0.0
    } else {
        (hand_len.saturating_sub(1) as f32 * reveal) + CardSize::Hand.dimensions().0
    };
    let final_center_x =
        index as f32 * reveal + CardSize::Hand.dimensions().0 * 0.5 - hand_width * 0.5;

    HandCardPose {
        translation: Val2::px(-final_center_x * deal_offset, -270.0 * deal_offset - lift),
        rotation: Rot2::radians(final_center_x * 0.0015 * deal_offset),
    }
}

pub(super) fn shengji_hand_card_reveal(hand_len: usize) -> f32 {
    let card_width = CardSize::Hand.dimensions().0;
    let available_width = DESIGN_WIDTH - 96.0;
    if hand_len <= 1 {
        HAND_CARD_REVEAL
    } else {
        HAND_CARD_REVEAL.min((available_width - card_width) / (hand_len - 1) as f32)
    }
}

pub(super) fn animate_turn_clocks(
    time: Res<Time>,
    mut clocks: Query<(&mut UiTransform, &mut BorderColor), With<TurnClock>>,
    mut hands: Query<&mut UiTransform, (With<TurnClockHand>, Without<TurnClock>)>,
) {
    let elapsed = time.elapsed_secs();
    let ring = (elapsed * 8.0).sin();
    for (mut transform, mut border) in &mut clocks {
        transform.scale = Vec2::splat(1.04 + ring.abs() * 0.05);
        transform.rotation = Rot2::radians(ring * 0.055);
        border.set_all(ACCENT.with_alpha(0.72 + ring.abs() * 0.28));
    }
    for mut transform in &mut hands {
        transform.rotation = Rot2::radians(elapsed * 2.8);
    }
}

pub(super) fn handle_text_input(
    mut keyboard_inputs: MessageReader<KeyboardInput>,
    mut ime_inputs: MessageReader<Ime>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut clipboard: ResMut<Clipboard>,
    mut form: ResMut<ConnectionForm>,
    mut chat: ResMut<ChatPanelState>,
    mut developer_hand: ResMut<DeveloperHandInput>,
    mut client: Option<ResMut<ClientResource>>,
    mut ui: ResMut<UiState>,
) {
    for input in ime_inputs.read() {
        let Ime::Commit { value, .. } = input else {
            continue;
        };
        if chat.focused {
            append_chat_input(&mut chat.input, value);
        } else if !developer_hand.focused && form.active == InputField::PlayerName {
            append_filtered_input(
                &mut form.player_name,
                InputField::PlayerName,
                value,
                MAX_PLAYER_NAME_CHARS,
            );
            form.error = None;
            ui.dirty = true;
        }
    }

    let control = keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight);
    if control {
        if keyboard.just_pressed(KeyCode::KeyV) && developer_hand.focused {
            let mut read = clipboard.fetch_text();
            if let Some(Ok(text)) = read.poll_result() {
                append_developer_hand_input(&mut developer_hand.value, &text);
            }
        } else if keyboard.just_pressed(KeyCode::KeyV) && chat.focused {
            let mut read = clipboard.fetch_text();
            if let Some(Ok(text)) = read.poll_result() {
                append_chat_input(&mut chat.input, &text);
            }
        } else if form.active == InputField::JoinAddress && keyboard.just_pressed(KeyCode::KeyV) {
            let mut read = clipboard.fetch_text();
            match read.poll_result() {
                Some(Ok(text)) => {
                    let mut address = String::new();
                    append_filtered_input(&mut address, InputField::JoinAddress, &text, 64);
                    if address.is_empty() {
                        form.error = Some("剪贴板中没有可用的服务器地址".to_owned());
                    } else {
                        form.join_address = address;
                        form.error = None;
                    }
                }
                Some(Err(error)) => {
                    form.error = Some(format!("无法读取剪贴板：{error}"));
                }
                None => {
                    form.error = Some("剪贴板内容尚未准备好".to_owned());
                }
            }
            ui.dirty = true;
        }
        return;
    }
    for input in keyboard_inputs.read() {
        if input.state != ButtonState::Pressed {
            continue;
        }
        match input.logical_key {
            Key::Backspace => {
                if developer_hand.focused {
                    developer_hand.value.pop();
                    form.error = None;
                    continue;
                }
                if chat.focused {
                    chat.input.pop();
                    continue;
                }
                active_input_mut(&mut form).pop();
                form.error = None;
                ui.dirty = true;
            }
            Key::Tab => {
                if chat.focused || developer_hand.focused {
                    continue;
                }
                form.active = match form.active {
                    InputField::PlayerName => InputField::HostPort,
                    InputField::HostPort => InputField::JoinAddress,
                    InputField::JoinAddress => InputField::PlayerName,
                };
                ui.dirty = true;
            }
            Key::Enter if chat.focused => {
                let message = chat.input.trim();
                if !message.is_empty()
                    && let Some(client) = client.as_deref_mut()
                    && client.0.send(ClientCommand::Chat {
                        content: ChatContent::Text(message.to_owned()),
                    })
                {
                    chat.input.clear();
                }
            }
            Key::Enter if developer_hand.focused => {
                developer_hand.focused = false;
                let input = developer_hand.value.trim();
                if input.is_empty() {
                    form.error = None;
                    continue;
                }
                #[cfg(feature = "developer")]
                {
                    let Some(client) = client.as_deref_mut() else {
                        continue;
                    };
                    match parse_developer_hand(input) {
                        Ok(cards) => {
                            if client.0.send(ClientCommand::Game(GameCommand::QiGui523(
                                QiGui523Command::SetDeveloperHand { cards },
                            ))) {
                                form.error = None;
                                developer_hand.value.clear();
                            } else {
                                form.error = Some("当前未连接，无法编辑开发者手牌".to_owned());
                            }
                        }
                        Err(error) => form.error = Some(error),
                    }
                }
            }
            Key::Escape if chat.focused => {
                chat.focused = false;
                chat.quick_voice_open = false;
            }
            Key::Escape if developer_hand.focused => {
                developer_hand.focused = false;
            }
            _ => {
                let Some(text) = input.text.as_deref() else {
                    continue;
                };
                if chat.focused {
                    append_chat_input(&mut chat.input, text);
                    continue;
                }
                if developer_hand.focused {
                    append_developer_hand_input(&mut developer_hand.value, text);
                    form.error = None;
                    continue;
                }
                let active = form.active;
                let value = active_input_mut(&mut form);
                let maximum = match active {
                    InputField::PlayerName => MAX_PLAYER_NAME_CHARS,
                    InputField::HostPort => 5,
                    InputField::JoinAddress => 64,
                };
                append_filtered_input(value, active, text, maximum);
                form.error = None;
                ui.dirty = true;
            }
        }
    }
}

pub(super) fn append_chat_input(value: &mut String, text: &str) {
    for character in text.chars().filter(|character| !character.is_control()) {
        if value.chars().count() >= MAX_CHAT_MESSAGE_CHARS {
            break;
        }
        value.push(character);
    }
}

fn append_developer_hand_input(value: &mut String, text: &str) {
    const MAX_DEVELOPER_HAND_INPUT: usize = 192;
    for character in text
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
    {
        if value.len() >= MAX_DEVELOPER_HAND_INPUT {
            break;
        }
        value.push(character.to_ascii_uppercase());
    }
}

#[cfg(feature = "developer")]
pub(super) fn parse_developer_hand(input: &str) -> Result<Vec<Card>, String> {
    let source = input
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .map(|character| character.to_ascii_uppercase())
        .collect::<Vec<_>>();
    if source.is_empty() {
        return Ok(Vec::new());
    }
    if !matches!(source[0], 'S' | 'H' | 'C' | 'D' | 'R' | 'B') {
        return parse_developer_hand_by_rank(&source);
    }
    if !source.len().is_multiple_of(2) {
        return Err("每张牌必须使用两个字符，例如 S4、ST、RJ".to_owned());
    }
    let mut cards = Vec::new();
    let mut copies = HashMap::<(Suit, Rank), u8>::new();
    for (index, code) in source.chunks_exact(2).enumerate() {
        let (suit, rank) = match (code[0], code[1]) {
            ('R', 'J') => (Suit::Spade, Rank::Joker),
            ('B', 'J') => (Suit::Club, Rank::Joker),
            (suit, rank) => {
                let suit = match suit {
                    'S' => Suit::Spade,
                    'H' => Suit::Heart,
                    'C' => Suit::Club,
                    'D' => Suit::Diamond,
                    other => {
                        return Err(format!("开发者手牌第 {} 项的花色 {other} 无效", index + 1));
                    }
                };
                let rank = parse_developer_rank(rank)
                    .ok_or_else(|| format!("开发者手牌第 {} 项的点数 {rank} 无效", index + 1))?;
                (suit, rank)
            }
        };

        let copy = copies.entry((suit, rank)).or_default();
        cards.push(Card::suited(*copy, suit, rank));
        *copy += 1;
    }
    Ok(cards)
}

#[cfg(feature = "developer")]
fn parse_developer_hand_by_rank(source: &[char]) -> Result<Vec<Card>, String> {
    let mut cards = Vec::with_capacity(source.len());
    let mut copies = HashMap::<(Suit, Rank), u8>::new();
    for (index, code) in source.iter().copied().enumerate() {
        let rank = if code == '0' {
            Rank::Joker
        } else {
            parse_developer_rank(code)
                .ok_or_else(|| format!("开发者手牌第 {} 项的点数 {code} 无效", index + 1))?
        };
        let suits = if rank == Rank::Joker {
            &[Suit::Spade, Suit::Club][..]
        } else {
            &Suit::IN_STRENGTH_ORDER
        };
        let suit = suits[fastrand::usize(..suits.len())];
        let copy = copies.entry((suit, rank)).or_default();
        cards.push(Card::suited(*copy, suit, rank));
        *copy += 1;
    }
    Ok(cards)
}

#[cfg(feature = "developer")]
fn parse_developer_rank(code: char) -> Option<Rank> {
    Some(match code {
        '2' => Rank::Two,
        '3' => Rank::Three,
        '4' => Rank::Four,
        '5' => Rank::Five,
        '6' => Rank::Six,
        '7' => Rank::Seven,
        '8' => Rank::Eight,
        '9' => Rank::Nine,
        'T' => Rank::Ten,
        'J' => Rank::Jack,
        'Q' => Rank::Queen,
        'K' => Rank::King,
        'A' => Rank::Ace,
        _ => return None,
    })
}

pub(super) fn append_filtered_input(
    value: &mut String,
    field: InputField,
    text: &str,
    maximum: usize,
) {
    for character in text.chars().filter(|character| !character.is_control()) {
        let allowed = match field {
            InputField::PlayerName => true,
            InputField::HostPort => character.is_ascii_digit(),
            InputField::JoinAddress => {
                character.is_ascii_alphanumeric()
                    || matches!(character, '.' | ':' | '[' | ']' | '-')
            }
        };
        if allowed && value.chars().count() < maximum {
            value.push(character);
        }
    }
}

fn active_input_mut(form: &mut ConnectionForm) -> &mut String {
    match form.active {
        InputField::PlayerName => &mut form.player_name,
        InputField::HostPort => &mut form.host_port,
        InputField::JoinAddress => &mut form.join_address,
    }
}

pub(super) fn handle_avatar_drop(
    mut dropped_files: MessageReader<FileDragAndDrop>,
    client: Option<Res<ClientResource>>,
    mut form: ResMut<ConnectionForm>,
    mut ui: ResMut<UiState>,
) {
    for event in dropped_files.read() {
        let FileDragAndDrop::DroppedFile { path_buf, .. } = event else {
            continue;
        };
        if client.is_some() {
            continue;
        }
        match normalize_avatar(path_buf) {
            Ok(png) => {
                form.avatar_png = Some(png);
                form.error = save_preferences(&form).err();
            }
            Err(error) => form.error = Some(error),
        }
        ui.dirty = true;
    }
}

pub(super) fn start_avatar_picker() -> Result<AvatarPickerReceiver, String> {
    let (sender, receiver) = mpsc::channel();
    thread::Builder::new()
        .name("leocard-avatar-picker".to_owned())
        .spawn(move || {
            let _ = sender.send(open_avatar_dialog());
        })
        .map_err(|error| format!("无法启动头像选择器：{error}"))?;
    Ok(Mutex::new(receiver))
}

pub(super) fn poll_avatar_picker(
    mut picker: ResMut<AvatarPicker>,
    mut form: ResMut<ConnectionForm>,
    mut ui: ResMut<UiState>,
) {
    let Some(receiver) = picker.pending.as_ref() else {
        return;
    };
    let result = receiver
        .lock()
        .expect("avatar picker mutex poisoned")
        .try_recv();
    let result = match result {
        Ok(result) => result,
        Err(TryRecvError::Empty) => return,
        Err(TryRecvError::Disconnected) => Err("头像选择器意外关闭".to_owned()),
    };
    picker.pending = None;
    match result {
        Ok(Some(path)) => match normalize_avatar(&path) {
            Ok(png) => {
                form.avatar_png = Some(png);
                form.error = save_preferences(&form).err();
            }
            Err(error) => form.error = Some(error),
        },
        Ok(None) => {}
        Err(error) => form.error = Some(error),
    }
    ui.dirty = true;
}

fn open_avatar_dialog() -> Result<Option<PathBuf>, String> {
    Ok(rfd::FileDialog::new()
        .set_title("选择玩家头像")
        .add_filter("头像图片", &["png", "jpg", "jpeg"])
        .pick_file())
}

pub(super) fn start_table_felt_picker() -> Result<TableFeltPickerReceiver, String> {
    let (sender, receiver) = mpsc::channel();
    thread::Builder::new()
        .name("leocard-table-felt-picker".to_owned())
        .spawn(move || {
            let _ = sender.send(open_table_felt_dialog());
        })
        .map_err(|error| format!("无法启动桌布选择器：{error}"))?;
    Ok(Mutex::new(receiver))
}

fn open_table_felt_dialog() -> Result<Option<PathBuf>, String> {
    Ok(rfd::FileDialog::new()
        .set_title("选择自定义桌布背景")
        .add_filter("桌布图片", &["png", "jpg", "jpeg"])
        .pick_file())
}

pub(super) fn decode_table_felt_image(bytes: &[u8]) -> Result<image::DynamicImage, String> {
    image::load_from_memory(bytes)
        .map_err(|error| format!("桌布必须是有效的 PNG、JPG 或 JPEG 图片：{error}"))
}

pub(super) fn poll_table_felt_picker(
    mut picker: ResMut<TableFeltPicker>,
    mut form: ResMut<ConnectionForm>,
    mut appearance: ResMut<TableAppearance>,
    mut ui: ResMut<UiState>,
) {
    let Some(receiver) = picker.pending.as_ref() else {
        return;
    };
    let result = receiver
        .lock()
        .expect("table felt picker mutex poisoned")
        .try_recv();
    let result = match result {
        Ok(result) => result,
        Err(TryRecvError::Empty) => return,
        Err(TryRecvError::Disconnected) => Err("桌布选择器意外关闭".to_owned()),
    };
    picker.pending = None;
    match result {
        Ok(Some(path)) => {
            form.table_felt_path = Some(path);
            appearance.error = save_preferences(&form).err();
        }
        Ok(None) => {}
        Err(error) => appearance.error = Some(error),
    }
    ui.dirty = true;
}

pub(super) fn sync_table_appearance(
    form: Res<ConnectionForm>,
    mut appearance: ResMut<TableAppearance>,
    mut images: ResMut<Assets<Image>>,
    mut ui: ResMut<UiState>,
) {
    if appearance.loaded_path == form.table_felt_path {
        return;
    }
    appearance.loaded_path.clone_from(&form.table_felt_path);
    appearance.custom_felt = None;
    appearance.error = None;
    if let Some(path) = &form.table_felt_path {
        let result = fs::read(path)
            .map_err(|error| format!("无法读取桌布图片：{error}"))
            .and_then(|bytes| decode_table_felt_image(&bytes));
        match result {
            Ok(image) => {
                appearance.custom_felt = Some(images.add(Image::from_dynamic(
                    image,
                    true,
                    RenderAssetUsages::default(),
                )));
            }
            Err(error) => appearance.error = Some(error),
        }
    }
    ui.dirty = true;
}

pub(super) fn sync_avatar_images(
    client: Option<Res<ClientResource>>,
    form: Res<ConnectionForm>,
    mut avatar_images: ResMut<AvatarImages>,
    mut images: ResMut<Assets<Image>>,
    mut ui: ResMut<UiState>,
) {
    if avatar_images.local_png != form.avatar_png {
        avatar_images.local = form
            .avatar_png
            .as_deref()
            .and_then(|png| image_handle_from_png(png, &mut images));
        avatar_images.local_png = form.avatar_png.clone();
        ui.dirty = true;
    }
    let Some(client) = client.as_deref() else {
        return;
    };
    for (id, png) in client.0.model().avatars() {
        if avatar_images.remote.contains_key(id) {
            continue;
        }
        if let Some(handle) = image_handle_from_png(png, &mut images) {
            avatar_images.remote.insert(*id, handle);
            ui.dirty = true;
        }
    }
}
