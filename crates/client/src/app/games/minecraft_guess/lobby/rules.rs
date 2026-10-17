use super::super::MinecraftGuessUiAction;
use crate::app::presentation::{EditableRuleSet, RuleConfigRow, UiComposer, add_rule_config_row};
use crate::app::runtime::UiAssets;
use crate::app::shell::UiAction;
use bevy::prelude::*;
use leocard_minecraft_guess::{
    GuessMode, GuessPool, MinecraftGuessRuleSet, RaceRules, TurnBasedRules,
};

impl EditableRuleSet for MinecraftGuessRuleSet {
    const ROW_HEIGHT: f32 = 44.0;
    const VALUE_WIDTH: f32 = 100.0;

    fn update_action(self) -> UiAction {
        UiAction::MinecraftGuess(MinecraftGuessUiAction::UpdateRules(self))
    }
}

pub(super) fn render_rule_rows(
    commands: &mut UiComposer,
    parent: Entity,
    rules: MinecraftGuessRuleSet,
    editable: bool,
    assets: &UiAssets,
) {
    RulesEditor {
        commands,
        parent,
        rules,
        editable,
        assets,
    }
    .render();
}

struct RuleChoice<'a, T> {
    label: &'static str,
    help: &'static str,
    values: &'a [T],
    current: T,
}

struct RulesEditor<'a, 'w, 's> {
    commands: &'a mut UiComposer<'w, 's>,
    parent: Entity,
    rules: MinecraftGuessRuleSet,
    editable: bool,
    assets: &'a UiAssets,
}

impl RulesEditor<'_, '_, '_> {
    fn render(&mut self) {
        let rules = self.rules;
        match rules.mode {
            GuessMode::TurnBased => self.turn_based(),
            GuessMode::Race => self.race(),
        }
        let options = *rules.options();
        self.choice(
            RuleChoice {
                label: "答案范围",
                help: "每局随机选取一个答案。",
                values: if cfg!(debug_assertions) || cfg!(feature = "developer") {
                    &GuessPool::ALL
                } else {
                    &GuessPool::STANDARD
                },
                current: options.pool,
            },
            |rules, value| rules.options_mut().pool = value,
            |value| value.label().to_owned(),
        );
        let mut hints = rules;
        hints.options_mut().candidate_hints = !options.candidate_hints;
        hints.options_mut().allow_unknown_guesses = false;
        self.toggle(
            "候选提示",
            if options.candidate_hints {
                "开启"
            } else {
                "关闭"
            },
            "输入名称时显示未猜过的匹配条目，可点击或用方向键选择。",
            self.editable.then_some(hints),
        );
        let mut unknown = rules;
        unknown.options_mut().allow_unknown_guesses = !options.allow_unknown_guesses;
        self.toggle(
            "允许错猜",
            if options.candidate_hints {
                "不适用"
            } else if options.allow_unknown_guesses {
                "开启"
            } else {
                "关闭"
            },
            "仅在关闭候选提示时可用。最多输入 10 个字，不存在的词也会记录。",
            (self.editable && !options.candidate_hints).then_some(unknown),
        );
    }

    fn turn_based(&mut self) {
        self.choice(
            RuleChoice {
                label: "游戏轮数",
                help: "每轮所有玩家依次担任一次首猜玩家。",
                values: &[1, 2, 3, 4, 5],
                current: self.rules.turn_based.rounds,
            },
            |rules, value| rules.turn_based.rounds = value,
            |value| format!("{value} 轮"),
        );
        self.choice(
            RuleChoice {
                label: "猜测计时",
                help: "每次轮到玩家时重新计时，超时自动跳过。",
                values: &TurnBasedRules::GUESS_SECONDS_OPTIONS,
                current: self.rules.turn_based.guess_seconds,
            },
            |rules, value| rules.turn_based.guess_seconds = value,
            |value| {
                if value == 0 {
                    "不限时".to_owned()
                } else {
                    format!("{value} 秒")
                }
            },
        );
    }

    fn race(&mut self) {
        self.choice(
            RuleChoice {
                label: "游戏局数",
                help: "所有玩家同时猜测同一答案，猜中加 5 分；首个未猜中的全绿条目加 3 分。",
                values: &RaceRules::GAMES_OPTIONS,
                current: self.rules.race.games,
            },
            |rules, value| rules.race.games = value,
            |value| format!("{value} 局"),
        );
        self.choice(
            RuleChoice {
                label: "公共倒计时",
                help: "每题共用倒计时，提交不会重置；超时公布答案。机器人每 8 秒猜测一次。",
                values: &RaceRules::SECONDS_OPTIONS,
                current: self.rules.race.seconds,
            },
            |rules, value| rules.race.seconds = value,
            |value| format!("{value} 秒"),
        );
    }

    fn toggle(
        &mut self,
        label: &'static str,
        value: &str,
        help: &'static str,
        toggled: Option<MinecraftGuessRuleSet>,
    ) {
        add_rule_config_row(
            self.commands,
            self.parent,
            RuleConfigRow {
                label,
                value: value.to_owned(),
                help,
                editable: self.editable,
                previous: toggled,
                next: toggled,
            },
            self.assets,
        );
    }

    fn choice<T: Copy + PartialEq>(
        &mut self,
        choice: RuleChoice<'_, T>,
        set: impl Fn(&mut MinecraftGuessRuleSet, T),
        display: impl Fn(T) -> String,
    ) {
        let index = choice
            .values
            .iter()
            .position(|value| *value == choice.current);
        let updated = |value: &T| {
            let mut rules = self.rules;
            set(&mut rules, *value);
            rules
        };
        add_rule_config_row(
            self.commands,
            self.parent,
            RuleConfigRow {
                label: choice.label,
                value: display(choice.current),
                help: choice.help,
                editable: self.editable,
                previous: index
                    .and_then(|index| index.checked_sub(1))
                    .filter(|_| self.editable)
                    .map(|index| updated(&choice.values[index])),
                next: index
                    .and_then(|index| choice.values.get(index + 1))
                    .filter(|_| self.editable)
                    .map(updated),
            },
            self.assets,
        );
    }
}
