//! UI actions, network polling, rule hints, notifications, and summary state.

use super::*;

pub(super) fn validated_host_form(form: &ConnectionForm) -> Result<(String, u16), String> {
    let name = form.player_name.trim().to_owned();
    if name.is_empty() {
        return Err("玩家名称不能为空".to_owned());
    }
    if name.chars().count() > MAX_PLAYER_NAME_CHARS {
        return Err(format!("玩家名称不能超过 {MAX_PLAYER_NAME_CHARS} 个字符"));
    }
    let port = form
        .host_port
        .trim()
        .parse::<u16>()
        .ok()
        .filter(|port| *port > 0)
        .ok_or_else(|| "端口必须是 1 到 65535 之间的整数".to_owned())?;
    Ok((name, port))
}

pub(super) fn handle_buttons(
    interactions: ButtonInteractions,
    mut client: Option<ResMut<ClientResource>>,
    mut form: ResMut<ConnectionForm>,
    profile: Res<LocalPlayerProfile>,
    mut ui: ResMut<UiState>,
    mut shengji_presentation: ResMut<ShengjiPresentationState>,
    mut local: LocalUiResources,
    mut interaction_cooldown: ResMut<PlayerInteractionCooldown>,
    mut chat: ResMut<ChatPanelState>,
    mut developer_hand: ResMut<DeveloperHandInput>,
    no_response_hints: Query<Entity, With<NoLegalResponseHint>>,
    mut commands: Commands,
) {
    #[cfg(not(feature = "developer"))]
    let _ = &mut developer_hand;
    for (interaction, action) in &interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let redraw = !matches!(
            action,
            UiAction::ToggleCard
                | UiAction::ToggleShengjiCard
                | UiAction::ToggleInteractionMenu(_)
                | UiAction::SendInteraction { .. }
                | UiAction::ToggleChatPanel
                | UiAction::ToggleAutoPlay
                | UiAction::FocusChatInput
                | UiAction::ToggleQuickVoiceMenu
                | UiAction::SendQuickVoice(_)
                | UiAction::Pass
        );
        #[cfg(feature = "developer")]
        let redraw = redraw && !matches!(action, UiAction::FocusDeveloperHand);
        match action {
            UiAction::FocusInput(field) => {
                chat.focused = false;
                developer_hand.focused = false;
                form.active = *field;
                form.error = None;
            }
            UiAction::OpenHostGamePicker => match validated_host_form(&form) {
                Ok(_) => {
                    ui.host_game_picker_open = true;
                    ui.profile_open = false;
                    ui.settings_open = false;
                    form.error = None;
                }
                Err(error) => form.error = Some(error),
            },
            UiAction::CloseHostGamePicker => {
                ui.host_game_picker_open = false;
            }
            UiAction::CreateRoom(game_kind) => {
                let result = validated_host_form(&form).and_then(|(name, port)| match game_kind {
                    GameKind::QiGui523 => {
                        let rules = normalize_host_rules(form.host_rules);
                        TcpGameClient::host_with_profile(
                            &name,
                            port,
                            rules,
                            form.avatar_png.clone(),
                            profile.identity.clone(),
                            profile.reference_points(),
                            profile.completed_games(),
                        )
                        .map_err(|error| error.to_string())
                    }
                    GameKind::TexasHoldem => TcpGameClient::host_texas_holdem_with_profile(
                        &name,
                        port,
                        normalize_texas_holdem_rules(form.texas_holdem_rules),
                        form.avatar_png.clone(),
                        profile.identity.clone(),
                        profile.reference_points(),
                        profile.completed_games(),
                    )
                    .map_err(|error| error.to_string()),
                    GameKind::Shengji => TcpGameClient::host_shengji_with_profile(
                        &name,
                        port,
                        normalize_shengji_rules(form.shengji_rules),
                        form.avatar_png.clone(),
                        profile.identity.clone(),
                        profile.reference_points(),
                        profile.completed_games(),
                    )
                    .map_err(|error| error.to_string()),
                });
                match result {
                    Ok(network) => {
                        *chat = ChatPanelState::default();
                        if let Err(error) = save_preferences(&form) {
                            warn!("{error}");
                        }
                        local.avatar_images.remote.clear();
                        commands.insert_resource(ClientResource(network));
                        ui.host_game_picker_open = false;
                        form.error = None;
                    }
                    Err(error) => form.error = Some(error),
                }
            }
            UiAction::JoinRoom => {
                let name = form.player_name.trim().to_owned();
                let result = if name.is_empty() {
                    Err("玩家名称不能为空".to_owned())
                } else if name.chars().count() > MAX_PLAYER_NAME_CHARS {
                    Err(format!("玩家名称不能超过 {MAX_PLAYER_NAME_CHARS} 个字符"))
                } else {
                    TcpGameClient::join_with_profile(
                        &name,
                        &form.join_address,
                        form.avatar_png.clone(),
                        profile.identity.clone(),
                        profile.reference_points(),
                        profile.completed_games(),
                    )
                    .map_err(|error| error.to_string())
                };
                match result {
                    Ok(network) => {
                        *chat = ChatPanelState::default();
                        if let Err(error) = save_preferences(&form) {
                            warn!("{error}");
                        }
                        local.avatar_images.remote.clear();
                        commands.insert_resource(ClientResource(network));
                        ui.host_game_picker_open = false;
                        form.error = None;
                    }
                    Err(error) => form.error = Some(error),
                }
            }
            UiAction::ChooseAvatar => {
                if local.avatar_picker.pending.is_none() {
                    match start_avatar_picker() {
                        Ok(receiver) => {
                            local.avatar_picker.pending = Some(receiver);
                            form.error = None;
                        }
                        Err(error) => form.error = Some(error),
                    }
                }
            }
            UiAction::ClearAvatar => {
                form.avatar_png = None;
                form.error = save_preferences(&form).err();
            }
            UiAction::ToggleProfile => {
                ui.profile_open = !ui.profile_open;
                if ui.profile_open {
                    ui.settings_open = false;
                    ui.host_game_picker_open = false;
                }
            }
            UiAction::ToggleSettings => {
                ui.settings_open = !ui.settings_open;
                if ui.settings_open {
                    ui.profile_open = false;
                    ui.host_game_picker_open = false;
                }
            }
            UiAction::ChooseTableFelt => {
                if local.table_felt_picker.pending.is_none() {
                    match start_table_felt_picker() {
                        Ok(receiver) => {
                            local.table_felt_picker.pending = Some(receiver);
                            local.table_appearance.error = None;
                        }
                        Err(error) => local.table_appearance.error = Some(error),
                    }
                }
            }
            UiAction::UseDefaultTableFelt => {
                form.table_felt_path = None;
                local.table_appearance.error = save_preferences(&form).err();
            }
            UiAction::SelectSeat(seat) => {
                if let Some(client) = client.as_deref_mut() {
                    client.0.send(ClientCommand::SelectSeat { seat: *seat });
                }
            }
            UiAction::ToggleReady => {
                let Some(client) = client.as_deref_mut() else {
                    continue;
                };
                let ready = client
                    .0
                    .model()
                    .lobby()
                    .and_then(|lobby| {
                        let you = client.0.model().you()?;
                        lobby.players.iter().find(|player| player.id == you)
                    })
                    .is_some_and(|player| player.ready);
                client.0.send(ClientCommand::SetReady { ready: !ready });
            }
            UiAction::UpdateRules(rules) => {
                if let Some(client) = client.as_deref_mut() {
                    client.0.send(ClientCommand::Game(GameCommand::QiGui523(
                        QiGui523Command::UpdateRules { rules: *rules },
                    )));
                }
            }
            UiAction::UpdateTexasRules(rules) => {
                if let Some(client) = client.as_deref_mut() {
                    client.0.send(ClientCommand::Game(GameCommand::TexasHoldem(
                        TexasHoldemCommand::UpdateRules { rules: *rules },
                    )));
                }
            }
            UiAction::UpdateShengjiRules(rules) => {
                if let Some(client) = client.as_deref_mut() {
                    client.0.send(ClientCommand::Game(GameCommand::Shengji(
                        ShengjiCommand::UpdateRules { rules: *rules },
                    )));
                }
            }
            UiAction::SetTexasRaiseTo(target) => {
                ui.texas_raise_to = *target;
            }
            UiAction::TexasAct(action) => {
                if let Some(client) = client.as_deref_mut() {
                    client.0.send(ClientCommand::Game(GameCommand::TexasHoldem(
                        TexasHoldemCommand::Act { action: *action },
                    )));
                }
            }
            UiAction::ShengjiDeclare(cards) => {
                if let Some(client) = client.as_deref_mut() {
                    client.0.send(ClientCommand::Game(GameCommand::Shengji(
                        ShengjiCommand::Declare {
                            cards: cards.clone(),
                        },
                    )));
                }
            }
            UiAction::ConfirmShengjiBidPass => {
                if let Some(client) = client.as_deref_mut() {
                    client.0.send(ClientCommand::Game(GameCommand::Shengji(
                        ShengjiCommand::ConfirmBidPass,
                    )));
                }
            }
            UiAction::ShengjiBottomCopy(cards) => {
                if let Some(client) = client.as_deref_mut() {
                    client.0.send(ClientCommand::Game(GameCommand::Shengji(
                        ShengjiCommand::ChooseBottomCopy {
                            cards: Some(cards.clone()),
                        },
                    )));
                }
            }
            UiAction::DeclineBottomCopy => {
                if let Some(client) = client.as_deref_mut() {
                    client.0.send(ClientCommand::Game(GameCommand::Shengji(
                        ShengjiCommand::ChooseBottomCopy { cards: None },
                    )));
                }
            }
            // 与七鬼五二三一致：单击和按住拖选都由手牌拖选系统在松开时结算。
            UiAction::ToggleShengjiCard => {}
            UiAction::ShengjiHint => {
                let Some(client) = client.as_deref_mut() else {
                    continue;
                };
                let Some(game) = client.0.model().shengji_game() else {
                    continue;
                };
                let Some(cards) = next_shengji_hint(game, &ui.selected_shengji) else {
                    continue;
                };
                ui.selected_shengji.clear();
                ui.selected_shengji.extend(cards);
            }
            UiAction::ShowShengjiPreviousTrick => {
                shengji_presentation.reveal_previous_trick();
            }
            UiAction::ToggleShengjiBuried => {
                ui.shengji_buried_open = !ui.shengji_buried_open;
            }
            UiAction::SubmitShengjiCards => {
                let Some(client) = client.as_deref_mut() else {
                    continue;
                };
                let Some(game) = client.0.model().shengji_game() else {
                    continue;
                };
                let cards = game
                    .your_hand
                    .iter()
                    .copied()
                    .filter(|card| ui.selected_shengji.contains(card))
                    .collect::<Vec<_>>();
                let command = match game.phase {
                    ShengjiPhaseView::Burying => ShengjiCommand::Bury { cards },
                    ShengjiPhaseView::BottomCopyBurying { .. } => ShengjiCommand::Bury { cards },
                    ShengjiPhaseView::FiveTrumpCrossing {
                        stage: ShengjiFiveTrumpCrossingStage::Deciding,
                        ..
                    } => ShengjiCommand::ChooseFiveTrumpCrossing { cards: Some(cards) },
                    ShengjiPhaseView::FiveTrumpCrossing {
                        stage: ShengjiFiveTrumpCrossingStage::Returning,
                        ..
                    } => ShengjiCommand::ReturnFiveTrumpCrossing { cards },
                    ShengjiPhaseView::Playing => ShengjiCommand::PlayCards { cards },
                    _ => continue,
                };
                client
                    .0
                    .send(ClientCommand::Game(GameCommand::Shengji(command)));
                ui.selected_shengji.clear();
            }
            UiAction::DeclineFiveTrumpCrossing => {
                if let Some(client) = client.as_deref_mut() {
                    client.0.send(ClientCommand::Game(GameCommand::Shengji(
                        ShengjiCommand::ChooseFiveTrumpCrossing { cards: None },
                    )));
                }
                ui.selected_shengji.clear();
            }
            UiAction::StartGame => {
                if let Some(client) = client.as_deref_mut() {
                    client.0.send(ClientCommand::StartGame);
                }
            }
            UiAction::ReturnToLobby => {
                if let Some(client) = client.as_deref_mut() {
                    client.0.send(ClientCommand::ReturnToLobby);
                }
            }
            UiAction::PlayAgain => {
                if let Some(client) = client.as_deref_mut() {
                    client.0.send(ClientCommand::PlayAgain);
                }
            }
            UiAction::LeaveRoom => {
                if let Some(client) = client.as_deref_mut() {
                    client.0.send(ClientCommand::LeaveRoom);
                }
            }
            #[cfg(feature = "developer")]
            UiAction::FocusDeveloperHand => {
                chat.focused = false;
                developer_hand.focused = true;
                form.active = InputField::PlayerName;
                form.error = None;
            }
            UiAction::ToggleInteractionMenu(player) => {
                ui.interaction_menu_open = if ui.interaction_menu_open == Some(*player) {
                    None
                } else {
                    Some(*player)
                };
            }
            UiAction::SendInteraction { target, kind } => {
                if interaction_cooldown.is_active(*kind) {
                    continue;
                }
                if let Some(client) = client.as_deref_mut()
                    && client.0.send(ClientCommand::Interact {
                        target: *target,
                        kind: *kind,
                    })
                {
                    let duration = match kind {
                        PlayerInteractionKind::Flower | PlayerInteractionKind::Egg => 0.5,
                        PlayerInteractionKind::Wine | PlayerInteractionKind::Shoe => 5.0,
                    };
                    interaction_cooldown.start(*kind, duration);
                    if matches!(
                        kind,
                        PlayerInteractionKind::Wine | PlayerInteractionKind::Shoe
                    ) {
                        ui.interaction_menu_open = None;
                    }
                }
            }
            UiAction::ToggleChatPanel => {
                developer_hand.focused = false;
                chat.open = !chat.open;
                if !chat.open {
                    chat.focused = false;
                    chat.quick_voice_open = false;
                }
            }
            UiAction::ToggleAutoPlay => {
                if let Some(client) = client.as_deref_mut() {
                    if let Some(game) = client.0.model().qigui523_game() {
                        let enabled = game
                            .players
                            .iter()
                            .find(|player| player.id == game.you)
                            .is_some_and(|player| player.auto_play);
                        client.0.send(ClientCommand::Game(GameCommand::QiGui523(
                            QiGui523Command::SetAutoPlay { enabled: !enabled },
                        )));
                    } else if let Some(game) = client.0.model().texas_holdem_game() {
                        let enabled = game
                            .players
                            .iter()
                            .find(|player| player.id == game.you)
                            .is_some_and(|player| player.auto_play);
                        client.0.send(ClientCommand::Game(GameCommand::TexasHoldem(
                            TexasHoldemCommand::SetAutoPlay { enabled: !enabled },
                        )));
                    } else if let Some(game) = client.0.model().shengji_game() {
                        let enabled = game
                            .players
                            .iter()
                            .find(|player| player.id == game.you)
                            .is_some_and(|player| player.auto_play);
                        client.0.send(ClientCommand::Game(GameCommand::Shengji(
                            ShengjiCommand::SetAutoPlay { enabled: !enabled },
                        )));
                    }
                }
            }
            UiAction::FocusChatInput => {
                developer_hand.focused = false;
                chat.open = true;
                chat.focused = true;
                chat.quick_voice_open = false;
            }
            UiAction::ToggleQuickVoiceMenu => {
                chat.open = true;
                chat.focused = false;
                chat.quick_voice_open = !chat.quick_voice_open;
            }
            UiAction::SendQuickVoice(index) => {
                if let Some(client) = client.as_deref_mut() {
                    client.0.send(ClientCommand::Chat {
                        content: ChatContent::QuickVoice(*index),
                    });
                }
                chat.quick_voice_open = false;
            }
            UiAction::Hint => {
                let Some(client) = client.as_deref_mut() else {
                    continue;
                };
                let decision = {
                    let model = client.0.model();
                    let Some(game) = model.qigui523_game() else {
                        continue;
                    };
                    let Some(rules) = model.qigui523_rules() else {
                        continue;
                    };
                    let Some(current) = game
                        .trick
                        .as_ref()
                        .and_then(|trick| trick.winning_play.as_ref())
                    else {
                        continue;
                    };
                    let Ok(current_play) = classify(&current.cards, rules) else {
                        continue;
                    };
                    let played_cards = game
                        .trick
                        .as_ref()
                        .into_iter()
                        .flat_map(|trick| &trick.records)
                        .flat_map(|record| match record {
                            PublicPlayRecord::Played { play, .. } => play.cards.as_slice(),
                            PublicPlayRecord::Passed { .. } => &[],
                        })
                        .copied()
                        .collect::<Vec<_>>();
                    next_greedy_hint(
                        &mut ui.greedy_hint,
                        &game.your_hand,
                        &current_play,
                        &played_cards,
                        rules,
                    )
                };
                match decision {
                    HintDecision::Select(cards) => {
                        ui.selected.clear();
                        ui.selected.extend(cards);
                    }
                    HintDecision::Pass => {
                        ui.selected.clear();
                        client.0.send(ClientCommand::Game(GameCommand::QiGui523(
                            QiGui523Command::Pass,
                        )));
                    }
                }
            }
            UiAction::ToggleCard => {}
            UiAction::Play => {
                let Some(client) = client.as_deref_mut() else {
                    continue;
                };
                if let Some(snapshot) = client.0.model().qigui523_game() {
                    let selected: Vec<_> = ui.selected.iter().copied().collect();
                    let cards = selected_cards_in_hand(&selected, snapshot);
                    if !cards.is_empty() {
                        client.0.send(ClientCommand::Game(GameCommand::QiGui523(
                            QiGui523Command::PlayCards { cards },
                        )));
                        ui.selected.clear();
                    }
                }
            }
            UiAction::Pass => {
                if let Some(client) = client.as_deref_mut() {
                    client.0.send(ClientCommand::Game(GameCommand::QiGui523(
                        QiGui523Command::Pass,
                    )));
                }
                for hint in &no_response_hints {
                    commands.entity(hint).despawn();
                }
            }
        }
        if redraw {
            ui.dirty = true;
        }
    }
}

pub(super) fn tick_player_interaction_cooldown(
    time: Res<Time>,
    mut cooldown: ResMut<PlayerInteractionCooldown>,
) {
    if cooldown.timers.is_empty() {
        return;
    }
    cooldown.tick(time.delta_secs());
}

pub(super) fn close_interaction_menu_on_outside_click(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut ui: ResMut<UiState>,
    badges: Query<&Interaction, With<OpponentBadge>>,
    menus: Query<(&InteractionMenuPanel, &ComputedNode, &UiGlobalTransform)>,
) {
    let Some(open_player) = ui.interaction_menu_open else {
        return;
    };
    if !mouse.just_pressed(MouseButton::Left)
        || badges
            .iter()
            .any(|interaction| *interaction == Interaction::Pressed)
    {
        return;
    }
    let Some(cursor) = windows
        .single()
        .ok()
        .and_then(Window::physical_cursor_position)
    else {
        return;
    };
    let inside = menus.iter().any(|(panel, node, transform)| {
        panel.0 == open_player && node.contains_point(*transform, cursor)
    });
    if !inside {
        ui.interaction_menu_open = None;
    }
}

pub(super) fn sync_interaction_cooldown_masks(
    cooldown: Res<PlayerInteractionCooldown>,
    ui: Res<UiState>,
    assets: Res<UiAssets>,
    mut masks: Query<(&InteractionCooldownMask, &mut ImageNode, &mut Visibility)>,
) {
    for (mask, mut image, mut visibility) in &mut masks {
        let frame = if ui.interaction_menu_open == Some(mask.player) {
            (cooldown.fraction(mask.kind) * INTERACTION_COOLDOWN_MASK_FRAMES as f32).ceil() as usize
        } else {
            0
        };
        let expected_visibility = if frame == 0 {
            Visibility::Hidden
        } else {
            Visibility::Visible
        };
        if *visibility != expected_visibility {
            *visibility = expected_visibility;
        }
        if let Some(mask) = assets.interaction_cooldown_masks.get(frame) {
            if image.image != *mask {
                image.image = mask.clone();
            }
        }
    }
}

pub(super) fn sync_opponent_badge_popups(
    ui: Res<UiState>,
    badges: Query<(&Interaction, &OpponentBadge)>,
    mut visibility: Query<&mut Visibility>,
) {
    for (interaction, badge) in &badges {
        let menu_open = ui.interaction_menu_open == Some(badge.player);
        if let Some(score_popup) = badge.score_popup
            && let Ok(mut popup) = visibility.get_mut(score_popup)
        {
            let expected = if !menu_open
                && matches!(interaction, Interaction::Hovered | Interaction::Pressed)
            {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
            if *popup != expected {
                *popup = expected;
            }
        }
        if let Ok(mut menu) = visibility.get_mut(badge.interaction_menu) {
            let expected = if menu_open {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
            if *menu != expected {
                *menu = expected;
            }
        }
    }
}

pub(super) fn next_greedy_hint(
    strategy: &mut GreedyStrategy,
    hand: &[Card],
    current_play: &ClassifiedPlay,
    played_cards: &[Card],
    rules: &RuleSet,
) -> HintDecision {
    let request = GreedyRequest {
        hand,
        current_play,
        played_cards,
        rules,
    };
    if let Some(play) = strategy.next_response(request) {
        return HintDecision::Select(play.cards().to_vec());
    }

    strategy.reset();
    strategy
        .next_response(request)
        .map_or(HintDecision::Pass, |play| {
            HintDecision::Select(play.cards().to_vec())
        })
}

pub(super) fn next_shengji_hint(
    game: &ShengjiSnapshot,
    selected: &HashSet<ShengjiCard>,
) -> Option<Vec<ShengjiCard>> {
    const MAX_HINTS: usize = 64;

    if game.current_player != Some(game.you) {
        return None;
    }
    let trump = game.trump?;
    let lead = game
        .trick
        .as_ref()
        .and_then(|trick| trick.plays.first())
        .map(|play| &play.play);
    let mut candidates = if let Some(lead) = lead {
        shengji_follow_suggestions(&game.your_hand, lead, trump, MAX_HINTS)
            .into_iter()
            .map(|play| play.cards)
            .collect::<Vec<_>>()
    } else {
        let mut cards = game.your_hand.clone();
        cards.sort_by_key(|card| {
            (
                trump.is_trump(*card),
                trump.strength(*card),
                card.suit().map_or(4, ShengjiSuit::bid_strength),
                card.deck(),
            )
        });
        cards.into_iter().map(|card| vec![card]).collect()
    };
    if candidates.is_empty()
        && let Ok(play) = ShengjiGreedyBot::choose(ShengjiGreedyBotRequest {
            hand: &game.your_hand,
            lead,
            trump,
        })
    {
        candidates.push(play.cards);
    }
    let current = candidates.iter().position(|cards| {
        cards.len() == selected.len() && cards.iter().all(|card| selected.contains(card))
    });
    let next = current.map_or(0, |index| (index + 1) % candidates.len());
    candidates.into_iter().nth(next)
}

pub(super) fn game_has_legal_response(
    game: &leocard_protocol::QiGui523Snapshot,
    rules: &RuleSet,
) -> bool {
    let Some(trick) = game.trick.as_ref() else {
        return false;
    };
    let Some(current) = trick.winning_play.as_ref() else {
        return true;
    };
    let Ok(current_play) = classify(&current.cards, rules) else {
        return false;
    };
    let played_cards = trick
        .records
        .iter()
        .flat_map(|record| match record {
            PublicPlayRecord::Played { play, .. } => play.cards.as_slice(),
            PublicPlayRecord::Passed { .. } => &[],
        })
        .copied()
        .collect::<Vec<_>>();
    has_legal_response(GreedyRequest {
        hand: &game.your_hand,
        current_play: &current_play,
        played_cards: &played_cards,
        rules,
    })
}

pub(super) fn sync_selection_label(
    ui: Res<UiState>,
    mut labels: Query<&mut Text, With<PlaySelectionCount>>,
) {
    let expected = format!("出牌 ({})", ui.selected.len());
    for mut label in &mut labels {
        if label.0 != expected {
            label.0.clone_from(&expected);
        }
    }
}

pub(super) fn developer_hand_input_label(input: &DeveloperHandInput) -> String {
    if input.value.is_empty() {
        if input.focused {
            "│".to_owned()
        } else {
            "编辑手牌，如 70523".to_owned()
        }
    } else {
        format!("{}{}", input.value, if input.focused { "│" } else { "" })
    }
}

pub(super) fn sync_developer_hand_input_text(
    input: Res<DeveloperHandInput>,
    mut labels: Query<(&mut Text, &mut TextColor), With<DeveloperHandInputText>>,
    mut fields: Query<(&mut Node, &mut BorderColor), With<DeveloperHandInputField>>,
) {
    if !input.is_changed() {
        return;
    }
    let expected = developer_hand_input_label(&input);
    let expected_color = if input.value.is_empty() { MUTED } else { TEXT };
    for (mut text, mut color) in &mut labels {
        if text.0 != expected {
            text.0.clone_from(&expected);
        }
        if color.0 != expected_color {
            color.0 = expected_color;
        }
    }
    for (mut node, mut border) in &mut fields {
        node.border = UiRect::all(px(if input.focused { 2 } else { 1 }));
        border.set_all(if input.focused { ACCENT } else { BORDER });
    }
}

pub(super) fn animate_no_legal_response_hint(
    time: Res<Time>,
    mut hints: Query<&mut UiTransform, With<NoLegalResponseHint>>,
) {
    let offset = (time.elapsed_secs() * 3.2).sin() * 3.0;
    for mut transform in &mut hints {
        transform.translation = Val2::px(0.0, offset);
    }
}

pub(super) fn poll_network(
    mut client: Option<ResMut<ClientResource>>,
    mut form: ResMut<ConnectionForm>,
    mut profile: ResMut<LocalPlayerProfile>,
    mut ui: ResMut<UiState>,
    mut seat_transition: ResMut<StartGameSeatTransition>,
    lobby_seats: Query<(
        &LobbySeatTransitionSource,
        &ComputedNode,
        &UiGlobalTransform,
    )>,
    mut commands: Commands,
) {
    let Some(client) = client.as_deref_mut() else {
        return;
    };
    let previous_state = client.0.state().clone();
    let previous_game = client.0.model().qigui523_game().cloned();
    let previous_shengji = client.0.model().shengji_game().cloned();
    let previous_lobby = client.0.model().lobby().cloned();
    let mut lobby_seat_snapshots = previous_lobby
        .as_ref()
        .map(|lobby| {
            lobby_seats
                .iter()
                .filter_map(|(source, node, transform)| {
                    let player = lobby.players.iter().find(|player| player.id == source.0)?;
                    let size = node.size() * node.inverse_scale_factor();
                    (size.min_element() > 1.0).then(|| LobbySeatTransitionSnapshot {
                        player: player.id,
                        center_global: transform.to_scale_angle_translation().2,
                        size,
                    })
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let was_host = client
        .0
        .model()
        .qigui523_game()
        .is_some_and(|game| game.you == game.host)
        || client
            .0
            .model()
            .texas_holdem_game()
            .is_some_and(|game| game.you == game.host)
        || client.0.model().lobby().is_some_and(|lobby| {
            client
                .0
                .model()
                .you()
                .is_some_and(|you| lobby.host == Some(you))
        });
    if !client.0.poll() {
        return;
    }
    if previous_lobby.is_some()
        && let Some(game) = client.0.model().qigui523_game()
    {
        lobby_seat_snapshots
            .retain(|seat| game.players.iter().any(|player| player.id == seat.player));
        seat_transition.begin(game.match_id, lobby_seat_snapshots);
    } else if client.0.model().lobby().is_some() || client.0.model().qigui523_game().is_none() {
        seat_transition.clear();
    }
    if client
        .0
        .model()
        .last_finished_match()
        .is_some_and(|(match_id, changes)| profile.apply_finished_match(match_id, changes))
        && let Err(error) = profile.save()
    {
        form.error = Some(error);
    }
    let accepted_host_rules = client.0.model().lobby().and_then(|lobby| {
        let you = client.0.model().you()?;
        (lobby.host == Some(you))
            .then(|| lobby.qigui523_rules().copied())
            .flatten()
            .map(normalize_host_rules)
    });
    if let Some(rules) = accepted_host_rules
        && form.host_rules != rules
    {
        form.host_rules = rules;
        if let Err(error) = save_preferences(&form) {
            form.error = Some(error);
        }
    }
    let accepted_texas_rules = client.0.model().lobby().and_then(|lobby| {
        let you = client.0.model().you()?;
        (lobby.host == Some(you))
            .then(|| lobby.texas_holdem_rules().copied())
            .flatten()
            .map(normalize_texas_holdem_rules)
    });
    if let Some(rules) = accepted_texas_rules
        && form.texas_holdem_rules != rules
    {
        form.texas_holdem_rules = rules;
        if let Err(error) = save_preferences(&form) {
            form.error = Some(error);
        }
    }
    let accepted_shengji_rules = client.0.model().lobby().and_then(|lobby| {
        let you = client.0.model().you()?;
        (lobby.host == Some(you))
            .then(|| lobby.shengji_rules().copied())
            .flatten()
            .map(normalize_shengji_rules)
    });
    if let Some(rules) = accepted_shengji_rules
        && form.shengji_rules != rules
    {
        form.shengji_rules = rules;
        if let Err(error) = save_preferences(&form) {
            form.error = Some(error);
        }
    }
    if let Some(game) = client.0.model().shengji_game() {
        let previous_stage = previous_shengji
            .as_ref()
            .and_then(|snapshot| match &snapshot.phase {
                ShengjiPhaseView::FiveTrumpCrossing { stage, .. } => Some(*stage),
                _ => None,
            });
        if let ShengjiPhaseView::FiveTrumpCrossing {
            stage,
            eligible,
            decided,
            ..
        } = &game.phase
            && previous_stage != Some(*stage)
        {
            ui.selected_shengji.clear();
            if *stage == ShengjiFiveTrumpCrossingStage::Deciding
                && eligible.contains(&game.you)
                && !decided.contains(&game.you)
                && let Some(trump) = game.trump
            {
                ui.selected_shengji.extend(
                    game.your_hand
                        .iter()
                        .copied()
                        .filter(|card| trump.is_trump(*card)),
                );
            }
        }
    }
    if client.0.model().room_closed() {
        if !was_host {
            form.error = Some("房主结束了游戏".to_owned());
        }
        commands.remove_resource::<ClientResource>();
    } else if client.0.model().left_room() {
        form.error = None;
        commands.remove_resource::<ClientResource>();
    } else if let NetworkState::Failed(error) = client.0.state() {
        form.error = Some(error.clone());
        commands.remove_resource::<ClientResource>();
    }
    if previous_state == *client.0.state() {
        if only_turn_timer_changed(previous_game.as_ref(), client.0.model().qigui523_game())
            || only_shengji_transient_progress_changed(
                previous_shengji.as_ref(),
                client.0.model().shengji_game(),
            )
        {
            return;
        }
    }
    ui.dirty = true;
}

/// 发牌时，接收者每四张全桌牌才真正拿到一张自己的牌。其间的三个快照只改变
/// 其他玩家手牌张数和未展示的剩余张数，无需销毁重建整张牌桌。抢主宽限期现在也
/// 不显示倒计时，因此仅毫秒数变化同样可以直接吸收。抄底询问的倒计时
/// 同样不绘制，避免每个计时快照重建手牌与玩家框。
pub(super) fn only_shengji_transient_progress_changed(
    before: Option<&leocard_protocol::ShengjiSnapshot>,
    after: Option<&leocard_protocol::ShengjiSnapshot>,
) -> bool {
    let (Some(before), Some(after)) = (before, after) else {
        return false;
    };
    match (&before.phase, &after.phase) {
        (ShengjiPhaseView::Dealing { .. }, ShengjiPhaseView::Dealing { .. }) => {
            if before.your_hand != after.your_hand {
                return false;
            }
            let mut normalized = before.clone();
            normalized.phase = after.phase.clone();
            for player in &mut normalized.players {
                if let Some(latest) = after.players.iter().find(|latest| latest.id == player.id) {
                    player.hand_len = latest.hand_len;
                }
            }
            normalized == *after
        }
        (ShengjiPhaseView::BiddingGrace { .. }, ShengjiPhaseView::BiddingGrace { .. }) => {
            let mut normalized = before.clone();
            let ShengjiPhaseView::BiddingGrace {
                milliseconds_remaining,
                ..
            } = &mut normalized.phase
            else {
                unreachable!();
            };
            let ShengjiPhaseView::BiddingGrace {
                milliseconds_remaining: latest,
                ..
            } = &after.phase
            else {
                unreachable!();
            };
            *milliseconds_remaining = *latest;
            normalized == *after
        }
        (ShengjiPhaseView::BottomCopying { .. }, ShengjiPhaseView::BottomCopying { .. }) => {
            let mut normalized = before.clone();
            let ShengjiPhaseView::BottomCopying {
                milliseconds_remaining,
                ..
            } = &mut normalized.phase
            else {
                unreachable!();
            };
            let ShengjiPhaseView::BottomCopying {
                milliseconds_remaining: latest,
                ..
            } = &after.phase
            else {
                unreachable!();
            };
            *milliseconds_remaining = *latest;
            normalized == *after
        }
        (
            ShengjiPhaseView::FiveTrumpCrossing { .. },
            ShengjiPhaseView::FiveTrumpCrossing { .. },
        ) => {
            let mut normalized = before.clone();
            let ShengjiPhaseView::FiveTrumpCrossing {
                milliseconds_remaining,
                ..
            } = &mut normalized.phase
            else {
                unreachable!();
            };
            let ShengjiPhaseView::FiveTrumpCrossing {
                milliseconds_remaining: latest,
                ..
            } = &after.phase
            else {
                unreachable!();
            };
            *milliseconds_remaining = *latest;
            normalized == *after
        }
        _ => false,
    }
}

pub(super) fn only_turn_timer_changed(
    before: Option<&leocard_protocol::QiGui523Snapshot>,
    after: Option<&leocard_protocol::QiGui523Snapshot>,
) -> bool {
    let (Some(before), Some(after)) = (before, after) else {
        return false;
    };
    if before.turn_timer == after.turn_timer {
        return false;
    }
    let mut normalized = before.clone();
    normalized.turn_timer = after.turn_timer;
    normalized == *after
}

pub(super) fn sync_turn_timer_label(
    client: Option<Res<ClientResource>>,
    mut labels: Query<&mut Text, With<TurnClockLabel>>,
) {
    let timer = client
        .as_deref()
        .and_then(|client| client.0.model().qigui523_game())
        .and_then(|game| game.turn_timer);
    let expected = turn_timer_label(timer);
    for mut label in &mut labels {
        if label.0 != expected {
            label.0.clone_from(&expected);
        }
    }
}

pub(super) fn sync_play_error_toast(
    client: Option<Res<ClientResource>>,
    form: Res<ConnectionForm>,
    appearance: Res<TableAppearance>,
    mut toast: ResMut<PlayErrorToast>,
    mut ui: ResMut<UiState>,
    assets: Res<UiAssets>,
    mut commands: Commands,
) {
    let mut next_message = None;
    let mut play_error_sound = false;
    if let Some(model) = client.as_deref().map(|client| client.0.model()) {
        if model.rejection_serial() != toast.seen_rejection_serial {
            toast.seen_rejection_serial = model.rejection_serial();
            next_message = model.last_rejection().and_then(rejection_label);
            play_error_sound = next_message.is_some();
        }
        if model.notice_serial() != toast.seen_notice_serial {
            toast.seen_notice_serial = model.notice_serial();
            next_message = model.last_notice().map(str::to_owned);
            play_error_sound = false;
        }
    }
    if toast.observed_form_error != form.error {
        toast.observed_form_error.clone_from(&form.error);
        if let Some(error) = &form.error {
            next_message = Some(error.clone());
            play_error_sound = true;
        }
    }
    if toast.observed_appearance_error != appearance.error {
        toast
            .observed_appearance_error
            .clone_from(&appearance.error);
        if let Some(error) = &appearance.error {
            next_message = Some(format!("桌布加载失败：{error}"));
            play_error_sound = true;
        }
    }
    let Some(message) = next_message else {
        return;
    };
    let retriggered = toast.active;
    toast.message = Some(message);
    toast.elapsed = 0.0;
    toast.shake_elapsed = retriggered.then_some(0.0);
    toast.entering = !retriggered;
    toast.active = true;
    if play_error_sound {
        commands.spawn((
            AudioPlayer::new(assets.error_popup_sound.clone()),
            PlaybackSettings::DESPAWN,
        ));
    }
    ui.dirty = true;
}

#[derive(Clone, Copy, Debug)]
pub(super) struct PlayErrorToastVisual {
    pub(super) x: f32,
    pub(super) y: f32,
    pub(super) opacity: f32,
}

pub(super) fn play_error_toast_visual(toast: &PlayErrorToast) -> PlayErrorToastVisual {
    let entry = if toast.entering {
        (toast.elapsed / PLAY_ERROR_TOAST_ENTRY_DURATION).clamp(0.0, 1.0)
    } else {
        1.0
    };
    let entry = 1.0 - (1.0 - entry).powi(3);
    let fade = ((PLAY_ERROR_TOAST_DURATION - toast.elapsed) / PLAY_ERROR_TOAST_FADE_DURATION)
        .clamp(0.0, 1.0);
    let life = (toast.elapsed / PLAY_ERROR_TOAST_DURATION).clamp(0.0, 1.0);
    let x = toast.shake_elapsed.map_or(0.0, |elapsed| {
        let strength = 1.0 - (elapsed / PLAY_ERROR_TOAST_SHAKE_DURATION).clamp(0.0, 1.0);
        (elapsed * 58.0).sin() * 11.0 * strength
    });
    PlayErrorToastVisual {
        x,
        y: -31.0 + (1.0 - entry) * 25.0 - life * 4.0,
        opacity: entry * fade,
    }
}

pub(super) fn animate_play_error_popup(
    time: Res<Time>,
    mut toast: ResMut<PlayErrorToast>,
    mut popups: Query<
        (
            &mut UiTransform,
            &mut BackgroundColor,
            &mut BorderColor,
            &mut BoxShadow,
            &mut Visibility,
        ),
        With<PlayErrorPopup>,
    >,
    mut texts: Query<&mut TextColor, With<PlayErrorPopupText>>,
) {
    if !toast.active {
        for (_, _, _, _, mut visibility) in &mut popups {
            *visibility = Visibility::Hidden;
        }
        return;
    }

    toast.elapsed += time.delta_secs();
    if toast.entering && toast.elapsed >= PLAY_ERROR_TOAST_ENTRY_DURATION {
        toast.entering = false;
    }
    if let Some(elapsed) = toast.shake_elapsed.as_mut() {
        *elapsed += time.delta_secs();
        if *elapsed >= PLAY_ERROR_TOAST_SHAKE_DURATION {
            toast.shake_elapsed = None;
        }
    }
    if toast.elapsed >= PLAY_ERROR_TOAST_DURATION {
        toast.active = false;
    }

    let visual = play_error_toast_visual(&toast);
    for (mut transform, mut background, mut border, mut shadow, mut visibility) in &mut popups {
        transform.translation = Val2::px(visual.x, visual.y);
        background.0 = HEADER_BG.with_alpha(0.97 * visual.opacity);
        border.set_all(DANGER.with_alpha(0.9 * visual.opacity));
        if let Some(style) = shadow.0.first_mut() {
            style.color = Color::BLACK.with_alpha(0.45 * visual.opacity);
        }
        *visibility = if toast.active {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    for mut text_color in &mut texts {
        text_color.0 = DANGER.with_alpha(visual.opacity);
    }
}

pub(super) fn update_summary_animation(
    mut commands: Commands,
    time: Res<Time>,
    client: Option<Res<ClientResource>>,
    assets: Res<UiAssets>,
    mut animation: ResMut<GameSummaryAnimation>,
) {
    let summary = client.as_deref().and_then(|client| {
        if let Some(game) = client.0.model().qigui523_game()
            && let GamePhaseView::Finished {
                match_id,
                scores,
                reference_changes,
                ..
            } = &game.phase
        {
            return Some((
                *match_id,
                None,
                sorted_summary_scores(scores)
                    .into_iter()
                    .map(|score| (score.player, score.score))
                    .collect::<Vec<_>>(),
                reference_changes
                    .iter()
                    .find(|change| change.player == game.you)
                    .is_none_or(|change| change.delta >= 0),
                SUMMARY_HAND_REVEAL_DURATION,
            ));
        }
        let game = client.0.model().texas_holdem_game()?;
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
        let nonnegative = if *tournament_complete {
            reference_changes
                .iter()
                .find(|change| change.player == game.you)
                .is_none_or(|change| change.delta >= 0)
        } else {
            own.stack >= own.hand_start_stack
        };
        let mut rows = game
            .players
            .iter()
            .map(|player| (player.id, player.stack))
            .collect::<Vec<_>>();
        if *tournament_complete {
            rows.extend(game.players.iter().map(|player| (player.id, player.stack)));
        }
        Some((
            game.match_id,
            Some(game.hand_number),
            rows,
            nonnegative,
            if *showdown {
                TEXAS_SHOWDOWN_REVEAL_DURATION
            } else {
                TEXAS_UNCONTESTED_REVEAL_DURATION
            },
        ))
    });
    let Some((match_id, texas_hand_number, scores, nonnegative_outcome, reveal_duration)) = summary
    else {
        if animation.match_id.is_some() {
            *animation = GameSummaryAnimation::default();
        }
        return;
    };

    if animation.match_id != Some(match_id) || animation.texas_hand_number != texas_hand_number {
        animation.match_id = Some(match_id);
        animation.texas_hand_number = texas_hand_number;
        animation.scores = scores;
        animation.elapsed = -reveal_duration;
        animation.nonnegative_outcome = nonnegative_outcome;
        animation.outcome_sound_played = false;
    } else {
        let duration = summary_animation_duration(animation.scores.len());
        if animation.elapsed < duration {
            animation.elapsed = (animation.elapsed + time.delta_secs()).min(duration);
        }
    }

    if animation.elapsed >= 0.0 && !animation.outcome_sound_played {
        let sound = if animation.nonnegative_outcome {
            assets.summary_score_sound.clone()
        } else {
            assets.summary_die_sound.clone()
        };
        commands.spawn((AudioPlayer::new(sound), PlaybackSettings::DESPAWN));
        animation.outcome_sound_played = true;
    }
}

fn summary_animation_duration(player_count: usize) -> f32 {
    let last_row = SUMMARY_ROW_START_DELAY
        + player_count.saturating_sub(1) as f32 * SUMMARY_ROW_INTERVAL
        + SUMMARY_SCORE_COUNT_DURATION.max(SUMMARY_ROW_ENTRY_DURATION);
    let actions = SUMMARY_ROW_START_DELAY
        + player_count as f32 * SUMMARY_ROW_INTERVAL
        + SUMMARY_ACTIONS_EXTRA_DELAY;
    SUMMARY_MODAL_ENTRY_DURATION.max(last_row).max(actions)
}

pub(super) fn sorted_summary_scores(scores: &[PlayerScore]) -> Vec<PlayerScore> {
    let mut ranked = scores.to_vec();
    ranked.sort_by(|left, right| {
        right
            .score
            .cmp(&left.score)
            .then_with(|| left.player.0.cmp(&right.player.0))
    });
    ranked
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct SummaryModalVisual {
    pub(super) offset_y: f32,
    pub(super) opacity: f32,
}

pub(super) fn summary_modal_visual(elapsed: f32) -> SummaryModalVisual {
    let progress = ease_out_cubic((elapsed / SUMMARY_MODAL_ENTRY_DURATION).clamp(0.0, 1.0));
    SummaryModalVisual {
        offset_y: -52.0 * (1.0 - progress),
        opacity: progress,
    }
}

pub(super) fn summary_row_progress(elapsed: f32, delay: f32) -> f32 {
    ease_out_cubic(((elapsed - delay) / SUMMARY_ROW_ENTRY_DURATION).clamp(0.0, 1.0))
}

pub(super) fn animate_game_summary_visuals(
    animation: Res<GameSummaryAnimation>,
    mut panels: ParamSet<(
        Query<(&mut UiTransform, &mut Visibility), With<GameSummaryModal>>,
        Query<
            (
                &GameSummaryRow,
                &mut UiTransform,
                &mut BackgroundColor,
                &mut Visibility,
            ),
            Without<GameSummaryModal>,
        >,
        Query<(&GameSummaryActions, &mut Visibility)>,
        Query<(&GameSummaryDivider, &mut BackgroundColor, &mut Visibility)>,
    )>,
    mut texts: Query<(&AnimatedSummaryText, &mut TextColor)>,
    mut panel_textures: Query<&mut ImageNode, With<GameSummaryPanelTexture>>,
) {
    if !animation.is_changed() {
        return;
    }
    let modal = summary_modal_visual(animation.elapsed);
    for mut image in &mut panel_textures {
        image.color = Color::WHITE.with_alpha(0.98 * modal.opacity);
    }
    for (mut transform, mut visibility) in &mut panels.p0() {
        transform.translation = Val2::px(0.0, modal.offset_y);
        *visibility = if animation.elapsed >= 0.0 {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    for (row, mut transform, mut background, mut visibility) in &mut panels.p1() {
        let progress = summary_row_progress(animation.elapsed, row.delay);
        transform.translation = Val2::px(0.0, 12.0 * (1.0 - progress));
        background.0 = PANEL_ALT.with_alpha(0.82 * progress);
        *visibility = if progress > 0.0 {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    for (animated, mut color) in &mut texts {
        let opacity = if animated.delay == 0.0 {
            modal.opacity
        } else {
            summary_row_progress(animation.elapsed, animated.delay)
        };
        color.0 = animated.color.with_alpha(opacity);
    }

    for (animated, mut visibility) in &mut panels.p2() {
        *visibility = if animation.elapsed >= animated.delay {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    for (divider, mut background, mut visibility) in &mut panels.p3() {
        let progress = summary_row_progress(animation.elapsed, divider.delay);
        background.0 = Color::srgb(0.52, 0.55, 0.54).with_alpha(0.28 * progress);
        *visibility = if progress > 0.0 {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

pub(super) fn animated_summary_score(elapsed: f32, target: u32, delay: f32) -> u32 {
    let progress = ((elapsed - delay) / SUMMARY_SCORE_COUNT_DURATION).clamp(0.0, 1.0);
    let eased = 1.0 - (1.0 - progress).powi(3);
    (target as f32 * eased).round() as u32
}

pub(super) fn animate_summary_scores(
    animation: Res<GameSummaryAnimation>,
    mut scores: Query<(&AnimatedSummaryScore, &mut Text)>,
) {
    if !animation.is_changed() {
        return;
    }
    for (score, mut text) in &mut scores {
        let displayed = animated_summary_score(animation.elapsed, score.target, score.delay);
        let expected = format!("{displayed} 分");
        if text.0 != expected {
            text.0 = expected;
        }
    }
}
