use super::DoudizhuUiAction;
use crate::app::presentation::{EditableRuleSet, RuleConfigRow, add_rule_config_row};
use crate::app::runtime::{AvatarImages, ClientResource, UiAssets};
use crate::app::shell::{
    LobbyPage, LobbyPageStyle, LobbyPlayerSection, UiAction, add_lobby_rules_heading,
};
use bevy::prelude::*;
use leocard_doudizhu::{Mode, RuleSet};
use leocard_protocol::LobbySnapshot;
use leocard_ui::UiComposer;

impl EditableRuleSet for RuleSet {
    const ROW_HEIGHT: f32 = 38.0;
    const VALUE_WIDTH: f32 = 110.0;

    fn update_action(self) -> UiAction {
        UiAction::Doudizhu(DoudizhuUiAction::UpdateRules(self))
    }
}

pub(crate) fn render_doudizhu_lobby(
    commands: &mut UiComposer,
    root: Entity,
    client: &ClientResource,
    lobby: &LobbySnapshot,
    assets: &UiAssets,
    avatars: &AvatarImages,
) {
    let rules = *lobby.rules.doudizhu().expect("doudizhu lobby rules");
    let page = LobbyPage::spawn(
        commands,
        root,
        client,
        lobby,
        assets,
        LobbyPageStyle::default(),
    );
    add_lobby_rules_heading(commands, page.rules, "斗地主配置", assets);
    let switched = RuleSet {
        mode: if rules.mode == Mode::Three {
            Mode::Four
        } else {
            Mode::Three
        },
        ..rules
    };
    let can_switch = page.can_configure && page.connected_count <= switched.mode.players();
    add_rule_config_row(
        commands,
        page.rules,
        RuleConfigRow {
            label: "玩法模式",
            value: rules.mode.label().to_owned(),
            help: "三人使用一副牌，每人17张、底牌3张；四人使用两副牌，每人25张、底牌8张。一名地主对抗其余农民。",
            editable: page.can_configure,
            previous: can_switch.then_some(switched),
            next: can_switch.then_some(switched),
        },
        assets,
    );
    add_rule_config_row(
        commands,
        page.rules,
        RuleConfigRow {
            label: "游戏轮数",
            value: format!("{} 轮", rules.rounds),
            help: "每轮所有玩家依次首叫一次。三人一轮为三局，四人一轮为四局；全员不叫时重新发牌，不计入局数。整场按累计分数排名结算积分。",
            editable: page.can_configure,
            previous: (page.can_configure && rules.rounds > 1).then(|| RuleSet {
                rounds: rules.rounds - 1,
                ..rules
            }),
            next: (page.can_configure && rules.rounds < 5).then(|| RuleSet {
                rounds: rules.rounds + 1,
                ..rules
            }),
        },
        assets,
    );
    let can_start = page.connected_count == rules.mode.players()
        && lobby
            .players
            .iter()
            .all(|p| p.connected && p.ready && p.seat.is_some());
    page.render_players(
        commands,
        LobbyPlayerSection::new(
            client,
            lobby,
            assets,
            avatars,
            rules.mode.players(),
            can_start,
            "等待玩家中",
        ),
    );
}
