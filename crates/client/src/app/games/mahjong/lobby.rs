use super::*;
use leocard_mahjong::{MahjongMatchLength, MahjongRuleSet};
use leocard_protocol::LobbySnapshot;

pub fn render_mahjong_lobby(
    commands: &mut Commands,
    root: Entity,
    client: &ClientResource,
    lobby: &LobbySnapshot,
    assets: &UiAssets,
    avatars: &AvatarImages,
) {
    let rules = *lobby.mahjong_rules().expect("麻将大厅应携带对应规则");
    let connected = LobbyMetrics::new(lobby).connected_player_count();
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
            min_width: px(340),
            flex_basis: px(390),
            flex_grow: 1.0,
            flex_direction: FlexDirection::Column,
            row_gap: px(15),
            ..default()
        },
        PANEL,
        PanelSkin::Section,
        assets,
    );
    add_section_title(commands, rules_panel, "国标麻将配置", assets);
    add_text(
        commands,
        rules_panel,
        format!("当前人数 {connected}/4，需要四人开局。采用 2014 版国标规则。"),
        13.0,
        MUTED,
        assets,
    );
    let can_configure = client.0.model().you() == lobby.host;
    let lengths = [
        MahjongMatchLength::SingleHand,
        MahjongMatchLength::EastRound,
        MahjongMatchLength::HalfGame,
        MahjongMatchLength::FullGame,
    ];
    let index = lengths
        .iter()
        .position(|length| *length == rules.match_length)
        .unwrap_or_default();
    add_rule_config_row(
        commands,
        rules_panel,
        MahjongRuleConfigRow {
            label: "场次",
            value: match rules.match_length {
                MahjongMatchLength::SingleHand => "单局结算",
                MahjongMatchLength::EastRound => "东风场（4 局）",
                MahjongMatchLength::HalfGame => "半庄场（8 局）",
                MahjongMatchLength::FullGame => "全庄场（16 局）",
            }
            .to_owned(),
            help: "每盘结算后全员可直接准备下一盘，不会返回大厅。单局模式仍会继续轮庄。",
            editable: can_configure,
            previous: can_configure.then_some(MahjongRuleSet {
                match_length: lengths[(index + lengths.len() - 1) % lengths.len()],
                ..rules
            }),
            next: can_configure.then_some(MahjongRuleSet {
                match_length: lengths[(index + 1) % lengths.len()],
                ..rules
            }),
        },
        assets,
    );
    let minimum_toggled = MahjongRuleSet {
        minimum_eight_points: !rules.minimum_eight_points,
        false_win: rules.false_win && !rules.minimum_eight_points,
        ..rules
    };
    add_rule_config_row(
        commands,
        rules_panel,
        MahjongRuleConfigRow {
            label: "8 番起和",
            value: if rules.minimum_eight_points {
                "开启"
            } else {
                "关闭"
            }
            .to_owned(),
            help: "开启时严格按 2014 国标要求至少 8 番（花牌不计入起和）；关闭后只按实际番数结算，不另加 8 分。",
            editable: can_configure,
            previous: can_configure.then_some(minimum_toggled),
            next: can_configure.then_some(minimum_toggled),
        },
        assets,
    );
    let winners_toggled = MahjongRuleSet {
        multiple_winners: !rules.multiple_winners,
        ..rules
    };
    add_rule_config_row(
        commands,
        rules_panel,
        MahjongRuleConfigRow {
            label: "一炮多响",
            value: if rules.multiple_winners {
                "允许"
            } else {
                "截和"
            }
            .to_owned(),
            help: "关闭时按出牌者之后的座次由最近一家截和；开启时所有合法和牌同时结算。",
            editable: can_configure,
            previous: can_configure.then_some(winners_toggled),
            next: can_configure.then_some(winners_toggled),
        },
        assets,
    );
    let false_win_toggled = MahjongRuleSet {
        false_win: !rules.false_win,
        ..rules
    };
    add_rule_config_row(
        commands,
        rules_panel,
        MahjongRuleConfigRow {
            label: "允许错和",
            value: if !rules.minimum_eight_points {
                "不适用"
            } else if rules.false_win {
                "开启"
            } else {
                "关闭"
            }
            .to_owned(),
            help: "开启后牌型已经完整便显示和牌按钮；不足 8 番属于错和，向其余三家各付 10 分并公开手牌，本盘继续。",
            editable: can_configure,
            previous: (can_configure && rules.minimum_eight_points).then_some(false_win_toggled),
            next: (can_configure && rules.minimum_eight_points).then_some(false_win_toggled),
        },
        assets,
    );

    let players_panel = add_panel(
        commands,
        content,
        Node {
            min_width: px(500),
            flex_basis: px(650),
            flex_grow: 2.0,
            flex_direction: FlexDirection::Column,
            row_gap: px(11),
            ..default()
        },
        PANEL_ALT,
        PanelSkin::Section,
        assets,
    );
    add_section_title(
        commands,
        players_panel,
        format!("玩家席位  {connected}/4"),
        assets,
    );
    LobbySeatSelector::new(client, lobby, assets, avatars).render(commands, players_panel);
    let actions = spawn_node(
        commands,
        players_panel,
        Node {
            width: percent(100),
            min_height: px(48),
            flex_direction: FlexDirection::Row,
            column_gap: px(12),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::FlexEnd,
            ..default()
        },
        None,
    );
    let you = client.0.model().you();
    let ready = you
        .and_then(|you| lobby.players.iter().find(|player| player.id == you))
        .is_some_and(|player| player.ready);
    let is_host = you == lobby.host;
    add_action_button(
        commands,
        actions,
        "退出房间",
        UiAction::Lobby(LobbyUiAction::LeaveRoom),
        ButtonKind::Pass,
        assets,
    );
    if is_host {
        let can_start = connected == 4
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
                UiAction::Lobby(LobbyUiAction::StartGame),
                ButtonKind::Primary,
                assets,
            );
        } else {
            add_disabled_action_button(commands, actions, "等待四名玩家", assets);
        }
    } else {
        add_action_button(
            commands,
            actions,
            if ready { "取消准备" } else { "准备" },
            UiAction::Lobby(LobbyUiAction::ToggleReady),
            if ready {
                ButtonKind::Secondary
            } else {
                ButtonKind::Primary
            },
            assets,
        );
    }
}
