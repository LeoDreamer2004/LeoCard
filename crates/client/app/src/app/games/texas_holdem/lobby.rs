//! 德州扑克房间规则与玩家席位界面。

use super::TexasHoldemUiAction;
use crate::app::presentation::{
    EditableRuleSet, MUTED, RuleConfigRow, UiComposer, add_rule_config_row, add_text,
};
use crate::app::runtime::{AvatarImages, ClientResource, UiAssets};
use crate::app::shell::{
    LobbyPage, LobbyPageStyle, LobbyPlayerSection, UiAction, add_lobby_rules_heading,
};
use bevy::prelude::*;
use leocard_protocol::{LobbySnapshot, TABLE_SEAT_COUNT};
use leocard_texas_holdem::TexasHoldemRuleSet;

type TexasRuleConfigRow<'a> = RuleConfigRow<'a, TexasHoldemRuleSet>;

impl EditableRuleSet for TexasHoldemRuleSet {
    const ROW_HEIGHT: f32 = 38.0;
    const VALUE_WIDTH: f32 = 78.0;

    fn update_action(self) -> UiAction {
        UiAction::TexasHoldem(TexasHoldemUiAction::UpdateRules(self))
    }
}

pub(crate) fn render_texas_holdem_lobby(
    commands: &mut UiComposer,
    root: Entity,
    client: &ClientResource,
    lobby: &LobbySnapshot,
    assets: &UiAssets,
    avatars: &AvatarImages,
) {
    let rules_value = *lobby
        .rules
        .texas_holdem()
        .expect("德州扑克大厅应携带对应规则");
    let page = LobbyPage::spawn(
        commands,
        root,
        client,
        lobby,
        assets,
        LobbyPageStyle::default(),
    );
    let rules_panel = page.rules;
    let connected_count = page.connected_count;
    let can_configure = page.can_configure;
    add_lobby_rules_heading(commands, rules_panel, "德州扑克配置", assets);
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
    let betting_toggled = TexasHoldemRuleSet {
        fixed_limit: !rules_value.fixed_limit,
        ..rules_value
    };
    add_rule_config_row(
        commands,
        rules_panel,
        TexasRuleConfigRow {
            label: "下注规则",
            value: if rules_value.fixed_limit {
                "有限注"
            } else {
                "无限注"
            }
            .to_owned(),
            help: "有限注：翻前和翻牌每次下注或加注 2，转牌和河牌每次 4；每街最多一次下注加三次加注，大盲视为翻前的首次下注。全下也须遵守限额。无限注可加注至全部筹码。",
            editable: can_configure,
            previous: can_configure.then_some(betting_toggled),
            next: can_configure.then_some(betting_toggled),
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

    let can_start = connected_count >= usize::from(TexasHoldemRuleSet::MIN_PLAYERS)
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
            TABLE_SEAT_COUNT,
            can_start,
            "等待玩家中",
        ),
    );
}
