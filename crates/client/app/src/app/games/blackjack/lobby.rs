use super::BlackjackUiAction;
use crate::app::presentation::{EditableRuleSet, RuleConfigRow, add_rule_config_row};
use crate::app::runtime::{AvatarImages, ClientResource, UiAssets};
use crate::app::shell::{LobbyPage, LobbyPageConfig, UiAction};
use bevy::prelude::*;
use leocard_blackjack::BlackjackRuleSet;
use leocard_protocol::LobbySnapshot;
use leocard_ui::UiComposer;

impl EditableRuleSet for BlackjackRuleSet {
    const ROW_HEIGHT: f32 = 38.0;
    const VALUE_WIDTH: f32 = 98.0;

    fn update_action(self) -> UiAction {
        UiAction::Blackjack(BlackjackUiAction::UpdateRules(self))
    }
}

pub(crate) fn render_blackjack_lobby(
    commands: &mut UiComposer,
    root: Entity,
    client: &ClientResource,
    lobby: &LobbySnapshot,
    assets: &UiAssets,
    avatars: &AvatarImages,
) {
    let rules = *lobby.rules.blackjack().expect("blackjack lobby rules");
    let page = LobbyPage::spawn(
        commands,
        root,
        client,
        lobby,
        assets,
        avatars,
        LobbyPageConfig::new("21 点配置")
            .description("1–6 人与系统庄家对局，六副牌，每小局重新洗牌。"),
    );

    let choices = [
        (
            "初始筹码",
            rules.initial_chips.to_string(),
            "每位玩家的房间筹码；每手下注上限等于初始筹码。",
            [100, 200, 500, 1000]
                .map(|initial_chips| BlackjackRuleSet {
                    initial_chips,
                    ..rules
                })
                .to_vec(),
        ),
        (
            "游戏局数",
            rules.rounds.to_string(),
            "完成指定局数后按剩余筹码排名；筹码不足最低下注者退出后续小局。",
            [3, 5, 10, 20]
                .map(|rounds| BlackjackRuleSet { rounds, ..rules })
                .to_vec(),
        ),
        (
            "最低下注",
            rules.minimum_bet.to_string(),
            "下注按 2 筹码递增，保证黑杰克 3:2 的赔付为整数。",
            [2, 10, 20]
                .map(|minimum_bet| BlackjackRuleSet {
                    minimum_bet,
                    ..rules
                })
                .to_vec(),
        ),
        (
            "回合时限",
            if rules.turn_seconds == 0 {
                "不限制".to_owned()
            } else {
                format!("{} 秒", rules.turn_seconds)
            },
            "超时自动按最低额度下注或停牌。",
            [15, 30, 60, 0]
                .map(|turn_seconds| BlackjackRuleSet {
                    turn_seconds,
                    ..rules
                })
                .to_vec(),
        ),
    ];
    for (label, value, help, options) in choices {
        let index = options.iter().position(|choice| *choice == rules);
        let previous = index
            .and_then(|index| index.checked_sub(1))
            .map(|index| options[index]);
        let next = index.and_then(|index| options.get(index + 1)).copied();
        add_rule_config_row(
            commands,
            page.rules,
            RuleConfigRow {
                label,
                value,
                help,
                editable: page.can_configure,
                previous: previous.filter(|_| page.can_configure),
                next: next.filter(|_| page.can_configure),
            },
            assets,
        );
    }
}
