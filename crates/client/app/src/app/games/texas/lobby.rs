//! 德州扑克房间规则与玩家席位界面。

use super::TexasUiAction;
use crate::app::presentation::{EditableRuleSet, RuleConfigRow, add_rule_config_row};
use crate::app::runtime::{AvatarImages, ClientResource, UiAssets};
use crate::app::shell::{LobbyPage, LobbyPageConfig, UiAction};
use bevy::prelude::*;
use leocard_protocol::LobbySnapshot;
use leocard_texas_holdem::TexasRuleSet;
use leocard_ui::UiComposer;

type TexasRuleConfigRow<'a> = RuleConfigRow<'a, TexasRuleSet>;

impl EditableRuleSet for TexasRuleSet {
    const ROW_HEIGHT: f32 = 38.0;
    const VALUE_WIDTH: f32 = 78.0;

    fn update_action(self) -> UiAction {
        UiAction::Texas(TexasUiAction::UpdateRules(self))
    }
}

pub(crate) fn render_texas_lobby(
    commands: &mut UiComposer,
    root: Entity,
    client: &ClientResource,
    lobby: &LobbySnapshot,
    assets: &UiAssets,
    avatars: &AvatarImages,
) {
    let rules_value = *lobby.rules.texas().expect("德州扑克大厅应携带对应规则");
    let page = LobbyPage::spawn(
        commands,
        root,
        client,
        lobby,
        assets,
        avatars,
        LobbyPageConfig::new("德州扑克配置").description("至少 3 人开局。"),
    );
    let rules_panel = page.rules;
    let connected_count = page.connected_count;
    let can_configure = page.can_configure;

    let chip_index = TexasRuleSet::STARTING_CHIP_OPTIONS
        .iter()
        .position(|chips| *chips == rules_value.starting_chips)
        .unwrap_or(2);
    let previous = chip_index.checked_sub(1).map(|index| TexasRuleSet {
        starting_chips: TexasRuleSet::STARTING_CHIP_OPTIONS[index],
        ..rules_value
    });
    let next = TexasRuleSet::STARTING_CHIP_OPTIONS
        .get(chip_index + 1)
        .copied()
        .map(|starting_chips| TexasRuleSet {
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
    let maximum_markers = (connected_count.saturating_sub(1)).min(TexasRuleSet::MAX_PLAYERS - 1);
    add_rule_config_row(
        commands,
        rules_panel,
        TexasRuleConfigRow {
            label: "鱿鱼标记",
            value: if rules_value.squid_markers == 0 {
                "关闭".to_owned()
            } else {
                rules_value.squid_markers.to_string()
            },
            help: "每手独赢主池获得一枚标记，可重复获得；平分主池或仅赢边池不发放。标记发完后，零标记者向持有者每枚支付 1BB（2 筹码），筹码不足则按比例支付。下一手重置标记；若有人出局，提前结算已发标记。数量最多为当前玩家数减一，0 为关闭。",
            editable: can_configure,
            previous: rules_value
                .squid_markers
                .checked_sub(1)
                .filter(|_| can_configure)
                .map(|squid_markers| TexasRuleSet {
                    squid_markers,
                    ..rules_value
                }),
            next: (can_configure && rules_value.squid_markers < maximum_markers).then_some(
                TexasRuleSet {
                    squid_markers: rules_value.squid_markers + 1,
                    ..rules_value
                },
            ),
        },
        assets,
    );
    let can_double = can_configure && rules_value.squid_markers > 0;
    let deepseek_toggled = TexasRuleSet {
        deepseek_squid_double: !rules_value.deepseek_squid_double,
        ..rules_value
    };
    add_rule_config_row(
        commands,
        rules_panel,
        TexasRuleConfigRow {
            label: "肥鱼翻倍",
            value: if rules_value.squid_markers == 0 {
                "不适用"
            } else if rules_value.deepseek_squid_double {
                "开启"
            } else {
                "关闭"
            }
            .to_owned(),
            help: "仅在开启鱿鱼时生效。DeepSeek 鱿鱼币每枚按 2BB（4 筹码）结算，普通鱿鱼币仍为 1BB（2 筹码）；筹码不足时按上述权重分配。",
            editable: can_configure,
            previous: can_double.then_some(deepseek_toggled),
            next: can_double.then_some(deepseek_toggled),
        },
        assets,
    );
    let betting_toggled = TexasRuleSet {
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
    let short_deck_toggled = TexasRuleSet {
        short_deck: !rules_value.short_deck,
        ..rules_value
    };
    let omaha_toggled = TexasRuleSet {
        omaha: !rules_value.omaha,
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
            help: "开启后每人发四张底牌；最终牌型必须恰好使用两张底牌和三张公共牌。短牌奥马哈最多支持 6 人。",
            editable: can_configure,
            previous: can_configure.then_some(omaha_toggled),
            next: can_configure.then_some(omaha_toggled),
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
            help: "开启后移除 2、3、4、5。短牌中同花高于葫芦，三条高于顺子；短牌奥马哈最多支持 6 人。",
            editable: can_configure,
            previous: can_configure.then_some(short_deck_toggled),
            next: can_configure.then_some(short_deck_toggled),
        },
        assets,
    );
    let ignore_kickers_toggled = TexasRuleSet {
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
}
