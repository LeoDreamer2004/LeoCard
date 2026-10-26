use super::super::MinecraftGuessUiAction;
use crate::app::presentation::{EditableRuleSet, RuleConfigRow, add_rule_config_row};
use crate::app::runtime::UiAssets;
use crate::app::shell::UiAction;
use bevy::prelude::*;
use leocard_minecraft_guess::{
    GraphRules, GuessMode, GuessPool, MAX_GUESS_CHARACTERS, MinecraftGuessRuleSet, RaceRules,
    TurnBasedRules,
};
use leocard_ui::UiComposer;

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
            GuessMode::Graph => self.graph(),
        }
        let options = *rules.options();
        if rules.mode != GuessMode::Graph {
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
        }
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
            &format!(
                "仅在关闭候选提示时可用。最多输入 {MAX_GUESS_CHARACTERS} 个字，不存在的词也会记录。"
            ),
            (self.editable && !options.candidate_hints).then_some(unknown),
        );
    }

    fn graph(&mut self) {
        let mut next = self.rules;
        next.graph.round_robin = !next.graph.round_robin;
        self.toggle(
            "对局方式",
            if self.rules.graph.round_robin { "轮流首位" } else { "单局" },
            "轮流首位：每人依次先手一局，整场档案分数增减为单局的 3 倍。单局：开场随机打乱座次，只玩一局。两种方式均按节点数量缩放档案分数。",
            self.editable.then_some(next),
        );
        self.choice(
            RuleChoice {
                label: "节点数量",
                help: "只统计可猜节点，标签不占名额。节点基础 1000 分，按度数、未知比例及连击计算，猜中获双倍奖励并续猜相邻节点，周围均已揭晓时可任选。",
                values: GraphRules::NODE_OPTIONS,
                current: self.rules.graph.nodes,
            },
            |rules, value| rules.graph.nodes = value,
            |value| format!("{value} 个"),
        );
        self.choice(
            RuleChoice {
                label: "图谱温度",
                help: "低温偏向均衡度数与短环；高温允许子图内出现枢纽、叶子、长环及交错连边，同时降低原图热门节点的入选权重，偏向低度数冷门节点。",
                values: &GraphRules::TEMPERATURE_OPTIONS,
                current: self.rules.graph.temperature,
            },
            |rules, value| rules.graph.temperature = value,
            |value| format!("{}.{}", value / 10, value % 10),
        );
        self.turn_clock(
            &GraphRules::GUESS_SECONDS_OPTIONS,
            self.rules.graph.guess_seconds,
            |rules, seconds| {
                rules.graph.guess_seconds = seconds;
            },
        );
        for (label, help, enabled, change) in [
            (
                "字数提示",
                "开启后提示未揭晓答案的字数，并按字数显示填空下划线。",
                self.rules.graph.show_character_count,
                0,
            ),
            (
                "未知关系方向",
                "开启后显示未知关系的方向箭头；关闭时，仅在关系两端节点均揭晓后显示。",
                self.rules.graph.show_directions,
                1,
            ),
            (
                "包含标签",
                "标签在全部相邻节点揭晓后自动解锁，不参与计分。",
                self.rules.graph.include_tags,
                2,
            ),
        ] {
            let mut next = self.rules;
            match change {
                0 => next.graph.show_character_count = !enabled,
                1 => next.graph.show_directions = !enabled,
                _ => next.graph.include_tags = !enabled,
            }
            self.toggle(
                label,
                if enabled { "开启" } else { "关闭" },
                help,
                self.editable.then_some(next),
            );
        }
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
        self.turn_clock(
            &TurnBasedRules::GUESS_SECONDS_OPTIONS,
            self.rules.turn_based.guess_seconds,
            |rules, seconds| {
                rules.turn_based.guess_seconds = seconds;
            },
        );
    }

    fn turn_clock(
        &mut self,
        values: &[u16],
        seconds: u16,
        set: impl Fn(&mut MinecraftGuessRuleSet, u16),
    ) {
        self.choice(
            RuleChoice {
                label: "猜测计时",
                help: "每次轮到玩家时重新计时，超时自动跳过。",
                values,
                current: seconds,
            },
            set,
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
        help: &str,
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
