use super::super::UnoUiAction;
use crate::app::presentation::{EditableRuleSet, RuleConfigRow, add_rule_config_row};
use crate::app::runtime::UiAssets;
use crate::app::shell::UiAction;
use bevy::prelude::*;
use leocard_uno::{FlipRuleSet, NoMercyRuleSet, UnoRuleSet};

impl EditableRuleSet for UnoRuleSet {
    const ROW_HEIGHT: f32 = 38.0;
    const VALUE_WIDTH: f32 = 78.0;

    fn update_action(self) -> UiAction {
        UiAction::Uno(UnoUiAction::UpdateRules(self))
    }
}

struct UnoRuleRow {
    label: &'static str,
    enabled: bool,
    help: &'static str,
    toggled: UnoRuleSet,
}

impl UnoRuleRow {
    fn render(
        self,
        commands: &mut Commands,
        parent: Entity,
        can_configure: bool,
        assets: &UiAssets,
    ) {
        add_rule_config_row(
            commands,
            parent,
            RuleConfigRow {
                label: self.label,
                value: if self.enabled { "开启" } else { "关闭" }.to_owned(),
                help: self.help,
                editable: can_configure,
                previous: can_configure.then_some(self.toggled),
                next: can_configure.then_some(self.toggled),
            },
            assets,
        );
    }
}

pub(super) fn render_uno_rule_rows(
    commands: &mut Commands,
    parent: Entity,
    rules: UnoRuleSet,
    can_configure: bool,
    assets: &UiAssets,
) {
    let rows = if rules.is_classic() {
        classic_rows(rules)
    } else if rules.is_no_mercy() {
        no_mercy_rows(rules)
    } else {
        flip_rows(rules)
    };
    for row in rows {
        row.render(commands, parent, can_configure, assets);
    }
}

fn classic_rows(rules: UnoRuleSet) -> Vec<UnoRuleRow> {
    vec![
        UnoRuleRow {
            label: "功能牌堆叠",
            enabled: rules.action_stacking,
            help: "允许禁手、+2、万能 +4 及兼容的扩展功能牌继续累计；+4 可压在 +2 上，+2 不能反压 +4。",
            toggled: UnoRuleSet {
                action_stacking: !rules.action_stacking,
                ..rules
            },
        },
        UnoRuleRow {
            label: "禁手摸牌",
            enabled: rules.skip_draw_penalty,
            help: "玩家每实际跳过一轮时，额外摸一张牌。",
            toggled: UnoRuleSet {
                skip_draw_penalty: !rules.skip_draw_penalty,
                ..rules
            },
        },
        UnoRuleRow {
            label: "抢出",
            enabled: rules.jump_in,
            help: "彩色牌落桌后，非下家若持有颜色和牌面完全相同的另一张牌，可在下家执行动作前抢出。关闭功能牌堆叠时只可抢数字牌。相同双牌可一次打出。",
            toggled: UnoRuleSet {
                jump_in: !rules.jump_in,
                ..rules
            },
        },
        UnoRuleRow {
            label: "UNO 宣告与检举",
            enabled: rules.uno_callout,
            help: "手里恰好两张且轮到自己时可先喊 UNO，随后本回合必须出到一张；未喊直接出到一张者在下次成功出牌前可被检举并罚摸 2 张。",
            toggled: UnoRuleSet {
                uno_callout: !rules.uno_callout,
                ..rules
            },
        },
    ]
}

fn no_mercy_rows(rules: UnoRuleSet) -> Vec<UnoRuleRow> {
    let mode = rules.no_mercy;
    vec![
        UnoRuleRow {
            label: "摸到能出",
            enabled: mode.draw_until_playable,
            help: "无牌可出时持续摸牌，直到摸到一张可出的牌，并必须处理该牌。",
            toggled: with_no_mercy(
                rules,
                NoMercyRuleSet {
                    draw_until_playable: !mode.draw_until_playable,
                    ..mode
                },
            ),
        },
        UnoRuleRow {
            label: "慈悲淘汰",
            enabled: mode.mercy_elimination,
            help: "手牌达到 25 张时立即淘汰；只剩一名未淘汰玩家时结束。",
            toggled: with_no_mercy(
                rules,
                NoMercyRuleSet {
                    mercy_elimination: !mode.mercy_elimination,
                    ..mode
                },
            ),
        },
        UnoRuleRow {
            label: "0 传递手牌",
            enabled: mode.zero_pass,
            help: "打出 0 时，所有未淘汰玩家按当前方向传递整手牌。",
            toggled: with_no_mercy(
                rules,
                NoMercyRuleSet {
                    zero_pass: !mode.zero_pass,
                    ..mode
                },
            ),
        },
        UnoRuleRow {
            label: "7 交换手牌",
            enabled: mode.seven_swap,
            help: "打出 7 后选择一名未淘汰玩家并与其交换整手牌。",
            toggled: with_no_mercy(
                rules,
                NoMercyRuleSet {
                    seven_swap: !mode.seven_swap,
                    ..mode
                },
            ),
        },
        UnoRuleRow {
            label: "UNO 宣告与检举",
            enabled: mode.uno_callout,
            help: "手里恰好两张且轮到自己时可先喊 UNO，随后本回合必须出到一张；未喊直接出到一张者在下次成功出牌前可被检举并罚摸 2 张。",
            toggled: with_no_mercy(
                rules,
                NoMercyRuleSet {
                    uno_callout: !mode.uno_callout,
                    ..mode
                },
            ),
        },
    ]
}

fn flip_rows(rules: UnoRuleSet) -> Vec<UnoRuleRow> {
    let mode = rules.flip;
    vec![
        UnoRuleRow {
            label: "随机正反配对",
            enabled: mode.random_pairing,
            help: "关闭时使用固定的正反面组合；开启后每局重新随机配对全部 112 张牌的两面。",
            toggled: with_flip(
                rules,
                FlipRuleSet {
                    random_pairing: !mode.random_pairing,
                    ..mode
                },
            ),
        },
        UnoRuleRow {
            label: "功能牌堆叠",
            enabled: mode.action_stacking,
            help: "允许亮暗两面的禁手与罚牌继续累计；各罚牌链仍按对应牌型规则结算。",
            toggled: with_flip(
                rules,
                FlipRuleSet {
                    action_stacking: !mode.action_stacking,
                    ..mode
                },
            ),
        },
        UnoRuleRow {
            label: "禁手摸牌",
            enabled: mode.skip_draw_penalty,
            help: "玩家每实际跳过一轮时，额外摸一张牌。",
            toggled: with_flip(
                rules,
                FlipRuleSet {
                    skip_draw_penalty: !mode.skip_draw_penalty,
                    ..mode
                },
            ),
        },
        UnoRuleRow {
            label: "UNO 宣告与检举",
            enabled: mode.uno_callout,
            help: "手里恰好两张且轮到自己时可先喊 UNO；未喊直接出到一张者在下次成功出牌前可被检举并罚摸 2 张。",
            toggled: with_flip(
                rules,
                FlipRuleSet {
                    uno_callout: !mode.uno_callout,
                    ..mode
                },
            ),
        },
        UnoRuleRow {
            label: "抢出",
            enabled: mode.jump_in,
            help: "只比较当前牌面；关闭功能牌堆叠时只可抢数字牌，相同双牌可一次打出。",
            toggled: with_flip(
                rules,
                FlipRuleSet {
                    jump_in: !mode.jump_in,
                    ..mode
                },
            ),
        },
    ]
}

fn with_no_mercy(rules: UnoRuleSet, no_mercy: NoMercyRuleSet) -> UnoRuleSet {
    UnoRuleSet { no_mercy, ..rules }
}

fn with_flip(rules: UnoRuleSet, flip: FlipRuleSet) -> UnoRuleSet {
    UnoRuleSet { flip, ..rules }
}
