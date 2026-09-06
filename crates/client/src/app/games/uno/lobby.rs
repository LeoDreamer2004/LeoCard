//! UNO 模式、扩展包与房间规则界面。

use super::*;
use leocard_protocol::LobbySnapshot;
use leocard_uno::{FlipRuleSet, Mode, NoMercyRuleSet};

pub fn render_uno_lobby(
    commands: &mut Commands,
    root: Entity,
    client: &ClientResource,
    lobby: &LobbySnapshot,
    ui: &UiState,
    assets: &UiAssets,
    avatars: &AvatarImages,
) {
    let rules_value = *lobby.uno_rules().expect("UNO 大厅应携带对应规则");
    let connected_count = connected_lobby_player_count(lobby);
    let content = spawn_node(
        commands,
        root,
        Node {
            width: percent(100),
            max_width: px(1180),
            flex_grow: 1.0,
            align_self: AlignSelf::Center,
            padding: UiRect::all(px(22)),
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            row_gap: px(18),
            column_gap: px(18),
            align_items: AlignItems::Stretch,
            ..default()
        },
        None,
    );
    let rules_panel = add_panel(
        commands,
        content,
        Node {
            min_width: px(300),
            flex_basis: px(330),
            flex_grow: 1.0,
            flex_direction: FlexDirection::Column,
            row_gap: px(14),
            ..default()
        },
        PANEL,
        PanelSkin::Section,
        assets,
    );
    let can_configure = client.0.model().you() == lobby.host;
    let title_row = spawn_node(
        commands,
        rules_panel,
        Node {
            width: percent(100),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            column_gap: px(12),
            ..default()
        },
        None,
    );
    add_section_title(
        commands,
        title_row,
        if rules_value.is_no_mercy() {
            "No Mercy 配置"
        } else if rules_value.is_flip() {
            "UNO FLIP 配置"
        } else {
            "UNO 配置"
        },
        assets,
    );
    render_uno_mode_dropdown(
        commands,
        root,
        title_row,
        rules_value,
        can_configure,
        ui.uno.mode_menu_open,
        assets,
    );
    add_text(
        commands,
        rules_panel,
        format!("当前人数 {connected_count}/{}", UnoRuleSet::MAX_PLAYERS),
        13.0,
        MUTED,
        assets,
    );
    if rules_value.is_classic() {
        let stack_toggled = UnoRuleSet {
            action_stacking: !rules_value.action_stacking,
            ..rules_value
        };
        add_rule_config_row(
            commands,
            rules_panel,
            UnoRuleConfigRow {
                label: "功能牌堆叠",
                value: if rules_value.action_stacking {
                    "开启"
                } else {
                    "关闭"
                }
                .to_owned(),
                help: "允许禁手、+2、万能 +4 及兼容的扩展功能牌继续累计；+4 可压在 +2 上，+2 不能反压 +4。",
                editable: can_configure,
                previous: can_configure.then_some(stack_toggled),
                next: can_configure.then_some(stack_toggled),
            },
            assets,
        );
    } else if rules_value.is_no_mercy() {
        for (label, enabled, help, toggled) in [
            (
                "摸到能出",
                rules_value.no_mercy.draw_until_playable,
                "无牌可出时持续摸牌，直到摸到一张可出的牌，并必须处理该牌。",
                UnoRuleSet {
                    no_mercy: NoMercyRuleSet {
                        draw_until_playable: !rules_value.no_mercy.draw_until_playable,
                        ..rules_value.no_mercy
                    },
                    ..rules_value
                },
            ),
            (
                "慈悲淘汰",
                rules_value.no_mercy.mercy_elimination,
                "手牌达到 25 张时立即淘汰；只剩一名未淘汰玩家时结束。",
                UnoRuleSet {
                    no_mercy: NoMercyRuleSet {
                        mercy_elimination: !rules_value.no_mercy.mercy_elimination,
                        ..rules_value.no_mercy
                    },
                    ..rules_value
                },
            ),
            (
                "0 传递手牌",
                rules_value.no_mercy.zero_pass,
                "打出 0 时，所有未淘汰玩家按当前方向传递整手牌。",
                UnoRuleSet {
                    no_mercy: NoMercyRuleSet {
                        zero_pass: !rules_value.no_mercy.zero_pass,
                        ..rules_value.no_mercy
                    },
                    ..rules_value
                },
            ),
            (
                "7 交换手牌",
                rules_value.no_mercy.seven_swap,
                "打出 7 后选择一名未淘汰玩家并与其交换整手牌。",
                UnoRuleSet {
                    no_mercy: NoMercyRuleSet {
                        seven_swap: !rules_value.no_mercy.seven_swap,
                        ..rules_value.no_mercy
                    },
                    ..rules_value
                },
            ),
            (
                "UNO 宣告与检举",
                rules_value.no_mercy.uno_callout,
                "手里恰好两张且轮到自己时可先喊 UNO，随后本回合必须出到一张；未喊直接出到一张者在下次成功出牌前可被检举并罚摸 2 张。",
                UnoRuleSet {
                    no_mercy: NoMercyRuleSet {
                        uno_callout: !rules_value.no_mercy.uno_callout,
                        ..rules_value.no_mercy
                    },
                    ..rules_value
                },
            ),
        ] {
            add_rule_config_row(
                commands,
                rules_panel,
                UnoRuleConfigRow {
                    label,
                    value: if enabled { "开启" } else { "关闭" }.to_owned(),
                    help,
                    editable: can_configure,
                    previous: can_configure.then_some(toggled),
                    next: can_configure.then_some(toggled),
                },
                assets,
            );
        }
    }
    if rules_value.is_classic() {
        let skip_draw_toggled = UnoRuleSet {
            skip_draw_penalty: !rules_value.skip_draw_penalty,
            ..rules_value
        };
        add_rule_config_row(
            commands,
            rules_panel,
            UnoRuleConfigRow {
                label: "禁手摸牌",
                value: if rules_value.skip_draw_penalty {
                    "开启"
                } else {
                    "关闭"
                }
                .to_owned(),
                help: "玩家每实际跳过一轮时，额外摸一张牌。",
                editable: can_configure,
                previous: can_configure.then_some(skip_draw_toggled),
                next: can_configure.then_some(skip_draw_toggled),
            },
            assets,
        );
        let jump_in_toggled = UnoRuleSet {
            jump_in: !rules_value.jump_in,
            ..rules_value
        };
        add_rule_config_row(
            commands,
            rules_panel,
            UnoRuleConfigRow {
                label: "抢出",
                value: if rules_value.jump_in {
                    "开启"
                } else {
                    "关闭"
                }
                .to_owned(),
                help: "彩色牌落桌后，非下家若持有颜色和牌面完全相同的另一张牌，可在下家执行动作前抢出。关闭功能牌堆叠时只可抢数字牌。相同双牌可一次打出。",
                editable: can_configure,
                previous: can_configure.then_some(jump_in_toggled),
                next: can_configure.then_some(jump_in_toggled),
            },
            assets,
        );
        let callout_toggled = UnoRuleSet {
            uno_callout: !rules_value.uno_callout,
            ..rules_value
        };
        add_rule_config_row(
            commands,
            rules_panel,
            UnoRuleConfigRow {
                label: "UNO 宣告与检举",
                value: if rules_value.uno_callout {
                    "开启"
                } else {
                    "关闭"
                }
                .to_owned(),
                help: "手里恰好两张且轮到自己时可先喊 UNO，随后本回合必须出到一张；未喊直接出到一张者在下次成功出牌前可被检举并罚摸 2 张。",
                editable: can_configure,
                previous: can_configure.then_some(callout_toggled),
                next: can_configure.then_some(callout_toggled),
            },
            assets,
        );
    } else if rules_value.is_flip() {
        for (label, enabled, help, toggled) in [
            (
                "随机正反配对",
                rules_value.flip.random_pairing,
                "关闭时使用固定的正反面组合；开启后每局重新随机配对全部 112 张牌的两面。",
                UnoRuleSet {
                    flip: FlipRuleSet {
                        random_pairing: !rules_value.flip.random_pairing,
                        ..rules_value.flip
                    },
                    ..rules_value
                },
            ),
            (
                "功能牌堆叠",
                rules_value.flip.action_stacking,
                "允许亮暗两面的禁手与罚牌继续累计；各罚牌链仍按对应牌型规则结算。",
                UnoRuleSet {
                    flip: FlipRuleSet {
                        action_stacking: !rules_value.flip.action_stacking,
                        ..rules_value.flip
                    },
                    ..rules_value
                },
            ),
            (
                "禁手摸牌",
                rules_value.flip.skip_draw_penalty,
                "玩家每实际跳过一轮时，额外摸一张牌。",
                UnoRuleSet {
                    flip: FlipRuleSet {
                        skip_draw_penalty: !rules_value.flip.skip_draw_penalty,
                        ..rules_value.flip
                    },
                    ..rules_value
                },
            ),
            (
                "UNO 宣告与检举",
                rules_value.flip.uno_callout,
                "手里恰好两张且轮到自己时可先喊 UNO；未喊直接出到一张者在下次成功出牌前可被检举并罚摸 2 张。",
                UnoRuleSet {
                    flip: FlipRuleSet {
                        uno_callout: !rules_value.flip.uno_callout,
                        ..rules_value.flip
                    },
                    ..rules_value
                },
            ),
        ] {
            add_rule_config_row(
                commands,
                rules_panel,
                UnoRuleConfigRow {
                    label,
                    value: if enabled { "开启" } else { "关闭" }.to_owned(),
                    help,
                    editable: can_configure,
                    previous: can_configure.then_some(toggled),
                    next: can_configure.then_some(toggled),
                },
                assets,
            );
        }
        let toggled = UnoRuleSet {
            flip: FlipRuleSet {
                jump_in: !rules_value.flip.jump_in,
                ..rules_value.flip
            },
            ..rules_value
        };
        add_rule_config_row(
            commands,
            rules_panel,
            UnoRuleConfigRow {
                label: "抢出",
                value: if rules_value.flip.jump_in {
                    "开启"
                } else {
                    "关闭"
                }
                .to_owned(),
                help: "只比较当前牌面；关闭功能牌堆叠时只可抢数字牌，相同双牌可一次打出。",
                editable: can_configure,
                previous: can_configure.then_some(toggled),
                next: can_configure.then_some(toggled),
            },
            assets,
        );
    }
    let expansion_button = add_action_button(
        commands,
        rules_panel,
        "扩展包设置",
        UiAction::ToggleUnoExpansionSettings,
        ButtonKind::Secondary,
        assets,
    );
    commands.entity(expansion_button).insert(Node {
        width: percent(100),
        min_width: px(0),
        height: px(56),
        padding: UiRect::axes(px(18), px(8)),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        ..default()
    });

    let players = add_panel(
        commands,
        content,
        Node {
            min_width: px(380),
            flex_basis: px(560),
            flex_grow: 2.0,
            flex_direction: FlexDirection::Column,
            row_gap: px(10),
            ..default()
        },
        PANEL_ALT,
        PanelSkin::Section,
        assets,
    );
    add_section_title(
        commands,
        players,
        format!("玩家席位  {connected_count}/{}", UnoRuleSet::MAX_PLAYERS),
        assets,
    );
    render_seat_selector(commands, players, client, lobby, assets, avatars);
    let actions = spawn_node(
        commands,
        players,
        Node {
            width: percent(100),
            min_height: px(48),
            flex_shrink: 0.0,
            flex_direction: FlexDirection::Row,
            column_gap: px(12),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::FlexEnd,
            ..default()
        },
        None,
    );
    commands
        .entity(actions)
        .insert((GlobalZIndex(800), FocusPolicy::Pass));
    let you = client.0.model().you();
    let ready = you
        .and_then(|you| lobby.players.iter().find(|player| player.id == you))
        .is_some_and(|player| player.ready);
    let is_host = you == lobby.host;
    add_action_button(
        commands,
        actions,
        "退出房间",
        UiAction::LeaveRoom,
        ButtonKind::Pass,
        assets,
    );
    if is_host {
        let can_start = connected_count >= usize::from(UnoRuleSet::MIN_PLAYERS)
            && lobby
                .players
                .iter()
                .filter(|player| player.connected)
                .all(|player| player.seat.is_some() && player.ready);
        if can_start {
            add_action_button(
                commands,
                actions,
                "开始游戏",
                UiAction::StartGame,
                ButtonKind::Primary,
                assets,
            );
        } else {
            add_disabled_action_button(commands, actions, "等待玩家中", assets);
        }
    } else {
        add_action_button(
            commands,
            actions,
            if ready { "取消准备" } else { "准备" },
            UiAction::ToggleReady,
            if ready {
                ButtonKind::Secondary
            } else {
                ButtonKind::Primary
            },
            assets,
        );
    }
    if ui.uno.expansion_settings_open {
        render_uno_expansion_settings(commands, root, rules_value, can_configure, assets);
    }
}

pub fn render_uno_mode_dropdown(
    commands: &mut Commands,
    root: Entity,
    parent: Entity,
    rules: UnoRuleSet,
    can_configure: bool,
    open: bool,
    assets: &UiAssets,
) {
    let dropdown_accent = Color::srgb(0.24, 0.90, 0.86);
    let mode_label = |mode| match mode {
        Mode::Classic => "UNO",
        Mode::NoMercy => "No Mercy",
        Mode::Flip => "UNO FLIP",
    };

    if open && can_configure {
        let dismiss = commands
            .spawn((
                Button,
                UiAction::CloseUnoModeMenu,
                Node {
                    position_type: PositionType::Absolute,
                    left: px(0),
                    right: px(0),
                    top: px(0),
                    bottom: px(0),
                    ..default()
                },
                BackgroundColor(Color::NONE),
                GlobalZIndex(1850),
                FocusPolicy::Block,
            ))
            .id();
        commands.entity(root).add_child(dismiss);
    }

    let selector = spawn_node(
        commands,
        parent,
        Node {
            position_type: PositionType::Relative,
            width: px(190),
            height: px(40),
            flex_direction: FlexDirection::Column,
            ..default()
        },
        None,
    );
    commands.entity(selector).insert(GlobalZIndex(1870));

    let mut trigger = commands.spawn((
        Node {
            width: percent(100),
            height: px(40),
            padding: UiRect::horizontal(px(13)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(8)),
            ..default()
        },
        BackgroundColor(if can_configure {
            Color::srgba(0.04, 0.48, 0.50, 0.48)
        } else {
            Color::srgba(0.30, 0.32, 0.33, 0.42)
        }),
        BorderColor::all(if can_configure {
            Color::srgba(0.30, 0.92, 0.88, 0.62)
        } else {
            Color::srgba(0.72, 0.74, 0.74, 0.28)
        }),
        BoxShadow::new(Color::BLACK.with_alpha(0.26), px(1), px(3), px(0), px(6)),
    ));
    if can_configure {
        trigger.insert((
            Button,
            BackgroundButtonTint,
            UiAction::ToggleUnoModeMenu,
            ButtonTint {
                normal: Color::srgba(0.04, 0.48, 0.50, 0.48),
                hovered: Color::srgba(0.06, 0.68, 0.70, 0.64),
                pressed: Color::srgba(0.03, 0.36, 0.40, 0.42),
            },
        ));
    } else {
        trigger.insert(FocusPolicy::Block);
    }
    let trigger = trigger.id();
    commands.entity(selector).add_child(trigger);
    let label = add_text(
        commands,
        trigger,
        mode_label(rules.mode),
        14.0,
        Color::WHITE,
        assets,
    );
    commands.entity(label).insert(FocusPolicy::Pass);
    let arrow = add_text(
        commands,
        trigger,
        if can_configure {
            if open { "▲" } else { "▼" }
        } else {
            "—"
        },
        12.0,
        if can_configure {
            dropdown_accent
        } else {
            MUTED
        },
        assets,
    );
    commands.entity(arrow).insert(FocusPolicy::Pass);

    if !open || !can_configure {
        return;
    }
    let menu = spawn_node(
        commands,
        selector,
        Node {
            position_type: PositionType::Absolute,
            right: px(0),
            top: px(44),
            width: px(190),
            padding: UiRect::all(px(5)),
            flex_direction: FlexDirection::Column,
            row_gap: px(3),
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(9)),
            ..default()
        },
        Some(Color::srgba(0.025, 0.090, 0.095, 0.88)),
    );
    commands.entity(menu).insert((
        UnoModeDropdownPanel,
        BorderColor::all(Color::srgba(0.30, 0.92, 0.88, 0.38)),
        BoxShadow::new(Color::BLACK.with_alpha(0.46), px(2), px(6), px(0), px(10)),
        GlobalZIndex(1890),
        FocusPolicy::Block,
    ));
    for (mode, label) in [
        (Mode::Classic, "UNO"),
        (Mode::NoMercy, "No Mercy"),
        (Mode::Flip, "UNO FLIP"),
    ] {
        let selected = mode == rules.mode;
        let option = commands
            .spawn((
                Button,
                BackgroundButtonTint,
                UiAction::UpdateUnoRules(UnoRuleSet { mode, ..rules }),
                ButtonTint {
                    normal: if selected {
                        Color::srgba(0.05, 0.62, 0.62, 0.38)
                    } else {
                        Color::srgba(0.22, 0.66, 0.66, 0.10)
                    },
                    hovered: Color::srgba(0.08, 0.76, 0.74, 0.48),
                    pressed: Color::srgba(0.03, 0.42, 0.44, 0.34),
                },
                Node {
                    width: percent(100),
                    height: px(36),
                    padding: UiRect::horizontal(px(11)),
                    align_items: AlignItems::Center,
                    border_radius: BorderRadius::all(px(6)),
                    ..default()
                },
                BackgroundColor(if selected {
                    Color::srgba(0.05, 0.62, 0.62, 0.38)
                } else {
                    Color::srgba(0.22, 0.66, 0.66, 0.10)
                }),
            ))
            .id();
        commands.entity(menu).add_child(option);
        let label = add_text(
            commands,
            option,
            if selected {
                format!("✓  {label}")
            } else {
                format!("   {label}")
            },
            13.5,
            if selected { dropdown_accent } else { TEXT },
            assets,
        );
        commands.entity(label).insert(FocusPolicy::Pass);
    }
}

pub fn render_uno_expansion_settings(
    commands: &mut Commands,
    root: Entity,
    rules: UnoRuleSet,
    can_configure: bool,
    assets: &UiAssets,
) {
    let overlay = spawn_node(
        commands,
        root,
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
        Some(Color::BLACK.with_alpha(0.62)),
    );
    commands
        .entity(overlay)
        .insert((GlobalZIndex(2100), FocusPolicy::Block));
    let modal = add_panel(
        commands,
        overlay,
        Node {
            width: px(620),
            max_width: percent(92),
            flex_direction: FlexDirection::Column,
            row_gap: px(14),
            ..default()
        },
        PANEL,
        PanelSkin::Window,
        assets,
    );
    add_section_title(commands, modal, "扩展包设置", assets);
    add_text(
        commands,
        modal,
        if rules.is_no_mercy() {
            "No Mercy 使用独立的扩展包设置。"
        } else if rules.is_flip() {
            "UNO FLIP 使用独立的扩展包设置。"
        } else {
            "选择要加入本房间牌堆的可选扩展包。"
        },
        13.0,
        MUTED,
        assets,
    );
    if rules.is_classic() {
        add_uno_expansion_row(
            commands,
            modal,
            "Swap Pack",
            "以交换手牌为特色，你的手牌随时可能变成别人的",
            rules.swap_pack,
            UnoRuleSet {
                swap_pack: !rules.swap_pack,
                ..rules
            },
            can_configure,
            assets,
        );
        add_uno_expansion_row(
            commands,
            modal,
            "Reverse Pack",
            "以改变方向为特色，小心罚牌反弹——你可能会被自己罚到！",
            rules.reverse_pack,
            UnoRuleSet {
                reverse_pack: !rules.reverse_pack,
                ..rules
            },
            can_configure,
            assets,
        );
        add_uno_expansion_row(
            commands,
            modal,
            "Stack Pack",
            "以累计罚牌为特色，加入堆叠 +1、+2、万能 +3 与随机堆叠牌",
            rules.stack_pack,
            UnoRuleSet {
                stack_pack: !rules.stack_pack,
                ..rules
            },
            can_configure,
            assets,
        );
    } else {
        add_text(
            commands,
            modal,
            if rules.is_flip() {
                "当前尚未加入 UNO FLIP 扩展包。"
            } else {
                "当前尚未加入 No Mercy 扩展包。"
            },
            15.0,
            TEXT,
            assets,
        );
    }
    let actions = spawn_node(
        commands,
        modal,
        Node {
            width: percent(100),
            justify_content: JustifyContent::FlexEnd,
            ..default()
        },
        None,
    );
    add_action_button(
        commands,
        actions,
        "关闭",
        UiAction::ToggleUnoExpansionSettings,
        ButtonKind::Secondary,
        assets,
    );
}

#[allow(clippy::too_many_arguments)]
fn add_uno_expansion_row(
    commands: &mut Commands,
    parent: Entity,
    name: &str,
    description: &str,
    enabled: bool,
    toggled_rules: UnoRuleSet,
    editable: bool,
    assets: &UiAssets,
) {
    let row = spawn_node(
        commands,
        parent,
        Node {
            width: percent(100),
            min_height: px(86),
            padding: UiRect::all(px(13)),
            align_items: AlignItems::Center,
            column_gap: px(14),
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(8)),
            ..default()
        },
        Some(PANEL_ALT.with_alpha(0.86)),
    );
    commands.entity(row).insert(BorderColor::all(BORDER));
    let name_slot = spawn_node(
        commands,
        row,
        Node {
            width: px(112),
            flex_shrink: 0.0,
            ..default()
        },
        None,
    );
    add_text(commands, name_slot, name, 16.0, TEXT, assets);
    add_uno_expansion_status(commands, row, enabled, toggled_rules, editable, assets);
    let description_slot = spawn_node(
        commands,
        row,
        Node {
            min_width: px(0),
            flex_grow: 1.0,
            ..default()
        },
        None,
    );
    add_text(commands, description_slot, description, 13.0, MUTED, assets);
}

fn add_uno_expansion_status(
    commands: &mut Commands,
    parent: Entity,
    enabled: bool,
    toggled_rules: UnoRuleSet,
    editable: bool,
    assets: &UiAssets,
) {
    let mut status = commands.spawn((
        UnoExpansionStatus,
        Node {
            width: px(38),
            height: px(38),
            flex_shrink: 0.0,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        BackgroundColor(Color::NONE),
    ));
    if editable {
        status.insert((
            Button,
            UiAction::UpdateUnoRules(toggled_rules),
            UnoExpansionStatusFrame,
            BorderColor::all(if enabled {
                READY.with_alpha(0.82)
            } else {
                DANGER.with_alpha(0.82)
            }),
            BackgroundColor(HEADER_BG.with_alpha(0.92)),
            Node {
                width: px(38),
                height: px(38),
                flex_shrink: 0.0,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border: UiRect::all(px(2)),
                border_radius: BorderRadius::all(px(6)),
                ..default()
            },
        ));
    }
    let status = status.id();
    commands.entity(parent).add_child(status);
    add_text(
        commands,
        status,
        if enabled { "✓" } else { "×" },
        24.0,
        if enabled { READY } else { DANGER },
        assets,
    );
}
