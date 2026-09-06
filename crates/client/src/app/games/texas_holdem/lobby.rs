//! 德州扑克房间规则与玩家席位界面。

use super::*;
use leocard_protocol::LobbySnapshot;

pub fn render_texas_holdem_lobby(
    commands: &mut Commands,
    root: Entity,
    client: &ClientResource,
    lobby: &LobbySnapshot,
    assets: &UiAssets,
    avatars: &AvatarImages,
) {
    let rules_value = *lobby
        .texas_holdem_rules()
        .expect("德州扑克大厅应携带对应规则");
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
    add_section_title(commands, rules_panel, "德州扑克配置", assets);
    add_text(
        commands,
        rules_panel,
        format!(
            "当前人数 {}/{}，至少 3 人开局。",
            connected_count, TABLE_SEAT_COUNT
        ),
        13.0,
        MUTED,
        assets,
    );

    let chip_index = TexasHoldemRuleSet::STARTING_CHIP_OPTIONS
        .iter()
        .position(|chips| *chips == rules_value.starting_chips)
        .unwrap_or(2);
    let previous = chip_index.checked_sub(1).map(|index| TexasHoldemRuleSet {
        starting_chips: TexasHoldemRuleSet::STARTING_CHIP_OPTIONS[index],
        ..rules_value
    });
    let next = TexasHoldemRuleSet::STARTING_CHIP_OPTIONS
        .get(chip_index + 1)
        .copied()
        .map(|starting_chips| TexasHoldemRuleSet {
            starting_chips,
            ..rules_value
        });
    add_rule_config_row(
        commands,
        rules_panel,
        TexasRuleConfigRow {
            label: "初始筹码",
            value: rules_value.starting_chips.to_string(),
            help: "每位玩家入桌时拥有的筹码。大盲固定为 2，小盲固定为 1。",
            editable: can_configure,
            previous: previous.filter(|_| can_configure),
            next: next.filter(|_| can_configure),
        },
        assets,
    );
    let toggled = TexasHoldemRuleSet {
        short_deck: !rules_value.short_deck,
        ..rules_value
    };
    add_rule_config_row(
        commands,
        rules_panel,
        TexasRuleConfigRow {
            label: "奥马哈",
            value: if rules_value.omaha {
                "开启".to_owned()
            } else {
                "关闭".to_owned()
            },
            help: "开启后每人发四张底牌；最终牌型必须恰好使用两张底牌和三张公共牌。",
            editable: can_configure,
            previous: can_configure.then_some(TexasHoldemRuleSet {
                omaha: !rules_value.omaha,
                ..rules_value
            }),
            next: can_configure.then_some(TexasHoldemRuleSet {
                omaha: !rules_value.omaha,
                ..rules_value
            }),
        },
        assets,
    );
    add_rule_config_row(
        commands,
        rules_panel,
        TexasRuleConfigRow {
            label: "短牌模式",
            value: if rules_value.short_deck {
                "开启".to_owned()
            } else {
                "关闭".to_owned()
            },
            help: "开启后移除 2、3、4、5。短牌中同花高于葫芦，三条高于顺子。",
            editable: can_configure,
            previous: can_configure.then_some(toggled),
            next: can_configure.then_some(toggled),
        },
        assets,
    );
    let ignore_kickers_toggled = TexasHoldemRuleSet {
        ignore_kickers: !rules_value.ignore_kickers,
        ..rules_value
    };
    add_rule_config_row(
        commands,
        rules_panel,
        TexasRuleConfigRow {
            label: "只比较最大牌型",
            value: if rules_value.ignore_kickers {
                "开启".to_owned()
            } else {
                "关闭".to_owned()
            },
            help: "开启后忽略踢脚牌；高牌和同花只比较最大的一张牌。",
            editable: can_configure,
            previous: can_configure.then_some(ignore_kickers_toggled),
            next: can_configure.then_some(ignore_kickers_toggled),
        },
        assets,
    );

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
        format!("玩家席位  {connected_count}/{TABLE_SEAT_COUNT}"),
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
        let can_start = connected_count >= usize::from(TexasHoldemRuleSet::MIN_PLAYERS)
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
}
