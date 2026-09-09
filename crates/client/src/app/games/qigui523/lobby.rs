//! 七鬼五二三准备大厅与规则配置。

use super::QiGui523UiAction;
use crate::app::presentation::{
    EditableRuleSet, MUTED, RuleConfigRow, add_rule_config_row, add_section_title, add_text,
    next_suit_comparison, next_time_control, previous_suit_comparison, previous_time_control,
    suit_comparison_label, time_control_label,
};
use crate::app::runtime::{AvatarImages, ClientResource, UiAssets};
use crate::app::shell::{LobbyPage, LobbyPageStyle, LobbyPlayerSection, UiAction};
use bevy::prelude::*;
use leocard_protocol::LobbySnapshot;
use leocard_protocol::TABLE_SEAT_COUNT;
use leocard_qigui523::{QiGuiRuleSet, SameCardPolicy};

impl EditableRuleSet for QiGuiRuleSet {
    const ROW_HEIGHT: f32 = 34.0;
    const VALUE_WIDTH: f32 = 68.0;

    fn update_action(self) -> UiAction {
        UiAction::QiGui523(QiGui523UiAction::UpdateRules(self))
    }
}

pub(crate) fn render_qigui523_lobby(
    commands: &mut Commands,
    root: Entity,
    client: &ClientResource,
    lobby: &LobbySnapshot,
    assets: &UiAssets,
    avatars: &AvatarImages,
) {
    let game_rules = *lobby
        .rules
        .qigui523()
        .expect("七鬼五二三大厅应携带对应规则");
    let page = LobbyPage::spawn(
        commands,
        root,
        client,
        lobby,
        assets,
        LobbyPageStyle {
            rules_gap: 12.0,
            ..default()
        },
    );
    let rules = page.rules;
    let connected_count = page.connected_count;
    let can_configure = page.can_configure;
    add_section_title(commands, rules, "游戏配置", assets);
    add_text(
        commands,
        rules,
        format!("当前人数 {connected_count}/{TABLE_SEAT_COUNT}。"),
        13.0,
        MUTED,
        assets,
    );

    let deck_previous = (game_rules.deck_count > QiGuiRuleSet::MIN_DECK_COUNT)
        .then(|| QiGuiRuleSet {
            deck_count: game_rules.deck_count - 1,
            ..game_rules
        })
        .filter(|rules| rules.validate().is_ok());
    let deck_next = (game_rules.deck_count < QiGuiRuleSet::MAX_DECK_COUNT)
        .then(|| QiGuiRuleSet {
            deck_count: game_rules.deck_count + 1,
            ..game_rules
        })
        .filter(|rules| rules.validate().is_ok());
    add_rule_config_row(
        commands,
        rules,
        RuleConfigRow {
            label: "牌副数",
            value: format!("{} 副", game_rules.deck_count),
            help: "使用几副完整扑克牌，可设置 1–8 副。多副牌会出现同花色同点数的重复牌。",
            editable: can_configure,
            previous: deck_previous.filter(|_| can_configure),
            next: deck_next.filter(|_| can_configure),
        },
        assets,
    );

    #[cfg(feature = "developer")]
    {
        let toggled_developer_deck = QiGuiRuleSet {
            developer_deck: !game_rules.developer_deck,
            ..game_rules
        };
        add_rule_config_row(
            commands,
            rules,
            RuleConfigRow {
                label: "开发者牌堆",
                value: if game_rules.developer_deck {
                    "开启".to_owned()
                } else {
                    "关闭".to_owned()
                },
                help: "开启后摸牌堆只生成所有玩家的初始手牌；关闭则使用完整牌堆。",
                editable: can_configure,
                previous: can_configure.then_some(toggled_developer_deck),
                next: can_configure.then_some(toggled_developer_deck),
            },
            assets,
        );
    }

    let hand_previous = (game_rules.hand_size > QiGuiRuleSet::MIN_HAND_SIZE)
        .then(|| QiGuiRuleSet {
            hand_size: game_rules.hand_size - 1,
            ..game_rules
        })
        .filter(|rules| rules.validate().is_ok());
    let hand_next = (game_rules.hand_size < QiGuiRuleSet::MAX_HAND_SIZE)
        .then(|| QiGuiRuleSet {
            hand_size: game_rules.hand_size + 1,
            ..game_rules
        })
        .filter(|rules| rules.validate().is_ok());
    add_rule_config_row(
        commands,
        rules,
        RuleConfigRow {
            label: "手牌张数",
            value: format!("{} 张", game_rules.hand_size),
            help: "每轮结束后，仍有摸牌时所有玩家会把手牌补到该数量，可设置 5–15 张。",
            editable: can_configure,
            previous: hand_previous.filter(|_| can_configure),
            next: hand_next.filter(|_| can_configure),
        },
        assets,
    );

    add_rule_config_row(
        commands,
        rules,
        RuleConfigRow {
            label: "出牌计时",
            value: time_control_label(game_rules.time_control).to_owned(),
            help: "不限时不会倒计时；X+Y 表示每次轮到玩家时重置 X 秒，之后扣除该玩家本局共享的 Y 秒，全部耗尽后自动出牌。",
            editable: can_configure,
            previous: can_configure
                .then(|| previous_time_control(game_rules.time_control))
                .flatten()
                .map(|time_control| QiGuiRuleSet {
                    time_control,
                    ..game_rules
                }),
            next: can_configure
                .then(|| next_time_control(game_rules.time_control))
                .flatten()
                .map(|time_control| QiGuiRuleSet {
                    time_control,
                    ..game_rules
                }),
        },
        assets,
    );

    let suit_previous = QiGuiRuleSet {
        suit_comparison: previous_suit_comparison(game_rules.suit_comparison),
        ..game_rules
    };
    let suit_next = QiGuiRuleSet {
        suit_comparison: next_suit_comparison(game_rules.suit_comparison),
        ..game_rules
    };
    add_rule_config_row(
        commands,
        rules,
        RuleConfigRow {
            label: "花色比较",
            value: suit_comparison_label(game_rules.suit_comparison).to_owned(),
            help: "点数组成相同时：极大法只比最大牌；逐项法从最大牌依次比较；记点法按黑桃/红桃/梅花/方块 4/3/2/1 点求和。",
            editable: can_configure,
            previous: can_configure.then_some(suit_previous),
            next: can_configure.then_some(suit_next),
        },
        assets,
    );

    let toggled_policy = QiGuiRuleSet {
        same_card_policy: match game_rules.same_card_policy {
            SameCardPolicy::MustBeHigher => SameCardPolicy::CanFollow,
            SameCardPolicy::CanFollow => SameCardPolicy::MustBeHigher,
        },
        ..game_rules
    };
    add_rule_config_row(
        commands,
        rules,
        RuleConfigRow {
            label: "同强度跟牌",
            value: match game_rules.same_card_policy {
                SameCardPolicy::MustBeHigher => "不允许".to_owned(),
                SameCardPolicy::CanFollow => "允许".to_owned(),
            },
            help: "整手牌比较结果完全相同时，决定后出的玩家是否仍可跟牌。关闭时必须严格更大。",
            editable: can_configure,
            previous: can_configure.then_some(toggled_policy),
            next: can_configure.then_some(toggled_policy),
        },
        assets,
    );

    let toggled_advanced_play_types = QiGuiRuleSet {
        advanced_play_types: !game_rules.advanced_play_types,
        ..game_rules
    };
    add_rule_config_row(
        commands,
        rules,
        RuleConfigRow {
            label: "牌型进阶",
            value: if game_rules.advanced_play_types {
                "开启".to_owned()
            } else {
                "关闭".to_owned()
            },
            help: "开启后允许三带一和三带一对；纯三张压任意三张顺子；两连对压任意四张顺子；两连飞机压任意三连对。三带牌不能压顺子，其他关系也不能反向压制。",
            editable: can_configure,
            previous: can_configure.then_some(toggled_advanced_play_types),
            next: can_configure.then_some(toggled_advanced_play_types),
        },
        assets,
    );
    let can_start = connected_count >= 2
        && lobby
            .players
            .iter()
            .filter(|player| player.connected)
            .all(|player| player.seat.is_some() && player.ready);
    page.render_players(
        commands,
        LobbyPlayerSection::new(
            client,
            lobby,
            assets,
            avatars,
            game_rules.player_count,
            can_start,
            "等待玩家中",
        ),
    );
}
