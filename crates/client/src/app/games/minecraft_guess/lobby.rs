use super::MinecraftGuessUiAction;
use crate::app::{
    presentation::{EditableRuleSet, RuleConfigRow, add_rule_config_row},
    runtime::{AvatarImages, ClientResource, UiAssets},
    shell::{LobbyPage, LobbyPageStyle, LobbyPlayerSection, UiAction, add_lobby_rules_heading},
};
use bevy::prelude::*;
use leocard_minecraft_guess::{GuessPool, MinecraftGuessRuleSet};
use leocard_protocol::{LobbySnapshot, TABLE_SEAT_COUNT};

impl EditableRuleSet for MinecraftGuessRuleSet {
    const ROW_HEIGHT: f32 = 44.0;
    const VALUE_WIDTH: f32 = 100.0;

    fn update_action(self) -> UiAction {
        UiAction::MinecraftGuess(MinecraftGuessUiAction::UpdateRules(self))
    }
}

pub(crate) fn render_guess_lobby(
    commands: &mut Commands,
    root: Entity,
    client: &ClientResource,
    lobby: &LobbySnapshot,
    assets: &UiAssets,
    avatars: &AvatarImages,
) {
    let rules = *lobby.rules.minecraft_guess().expect("guessing lobby rules");
    let page = LobbyPage::spawn(
        commands,
        root,
        client,
        lobby,
        assets,
        LobbyPageStyle::default(),
    );
    add_lobby_rules_heading(commands, page.rules, "竞猜配置", assets);
    add_rule_config_row(
        commands,
        page.rules,
        RuleConfigRow {
            label: "游戏轮数",
            value: format!("{} 轮", rules.rounds),
            help: "每轮所有玩家依次担任一次首猜玩家。例如 3 人、2 轮共进行 6 局。",
            editable: page.can_configure,
            previous: (page.can_configure && rules.rounds > 1).then_some(MinecraftGuessRuleSet {
                rounds: rules.rounds.saturating_sub(1),
                ..rules
            }),
            next: (page.can_configure && rules.rounds < 5).then_some(MinecraftGuessRuleSet {
                rounds: rules.rounds.saturating_add(1),
                ..rules
            }),
        },
        assets,
    );
    let (label, previous, next) = match rules.pool {
        GuessPool::Blocks => ("方块", GuessPool::Mixed, GuessPool::Items),
        GuessPool::Items => ("物品", GuessPool::Blocks, GuessPool::Mixed),
        GuessPool::Mixed => ("方块与物品", GuessPool::Items, GuessPool::Blocks),
    };
    add_rule_config_row(
        commands,
        page.rules,
        RuleConfigRow {
            label: "答案范围",
            value: label.to_owned(),
            help: "每轮随机选取一个答案。同一轮只猜对应种类。",
            editable: page.can_configure,
            previous: page.can_configure.then_some(MinecraftGuessRuleSet {
                pool: previous,
                ..rules
            }),
            next: page.can_configure.then_some(MinecraftGuessRuleSet {
                pool: next,
                ..rules
            }),
        },
        assets,
    );
    let toggled = page.can_configure.then_some(MinecraftGuessRuleSet {
        candidate_hints: !rules.candidate_hints,
        allow_unknown_guesses: false,
        ..rules
    });
    add_rule_config_row(
        commands,
        page.rules,
        RuleConfigRow {
            label: "候选提示",
            value: if rules.candidate_hints {
                "开启"
            } else {
                "关闭"
            }
            .to_owned(),
            help: "输入名称时显示匹配条目，可点击或用方向键选择。关闭后须自行输入完整名称。",
            editable: page.can_configure,
            previous: toggled,
            next: toggled,
        },
        assets,
    );
    let toggled = (page.can_configure && !rules.candidate_hints).then_some(MinecraftGuessRuleSet {
        allow_unknown_guesses: !rules.allow_unknown_guesses,
        ..rules
    });
    add_rule_config_row(
        commands,
        page.rules,
        RuleConfigRow {
            label: "允许错猜",
            value: if rules.candidate_hints {
                "不适用"
            } else if rules.allow_unknown_guesses {
                "开启"
            } else {
                "关闭"
            }
            .to_owned(),
            help: "仅在关闭候选提示时可用。最多输入 10 个字；不存在的词也会记录并消耗本次回合。",
            editable: page.can_configure,
            previous: toggled,
            next: toggled,
        },
        assets,
    );
    let can_start = page.connected_count >= 2
        && lobby
            .players
            .iter()
            .filter(|player| player.connected)
            .all(|player| player.ready && player.seat.is_some());
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
