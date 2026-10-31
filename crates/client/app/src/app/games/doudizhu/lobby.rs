use super::{DoudizhuUiAction, DoudizhuUiState};
use crate::app::presentation::{EditableRuleSet, RuleConfigRow, add_rule_config_row};
use crate::app::runtime::{AvatarImages, ClientResource, UiAssets};
use crate::app::shell::{CozyDropdown, CozyDropdownOption, LobbyPage, LobbyPageConfig, UiAction};
use bevy::prelude::*;
use leocard_doudizhu::{DoudizhuMode, DoudizhuRuleSet};
use leocard_protocol::LobbySnapshot;
use leocard_ui::UiComposer;

impl EditableRuleSet for DoudizhuRuleSet {
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
    ui: &DoudizhuUiState,
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
        avatars,
        LobbyPageConfig::new(format!("{}配置", rules.mode.label()))
            .mode(mode_dropdown(rules.mode), ui.mode_menu_open),
    );
    add_rule_config_row(
        commands,
        page.rules,
        RuleConfigRow {
            label: "游戏轮数",
            value: format!("{} 轮", rules.rounds),
            help: match rules.mode {
                DoudizhuMode::Three => {
                    "每轮三局，所有玩家依次首叫一次。全员不叫时重新发牌，不计入局数。整场按累计分数排名结算积分。"
                }
                DoudizhuMode::Four => {
                    "每轮四局，所有玩家依次首叫一次。全员不叫时重新发牌，不计入局数。整场按累计分数排名结算积分。"
                }
            },
            editable: page.can_configure,
            previous: (page.can_configure && rules.rounds > 1).then(|| DoudizhuRuleSet {
                rounds: rules.rounds - 1,
                ..rules
            }),
            next: (page.can_configure && rules.rounds < 5).then(|| DoudizhuRuleSet {
                rounds: rules.rounds + 1,
                ..rules
            }),
        },
        assets,
    );
    if rules.mode == DoudizhuMode::Four {
        let toggled = DoudizhuRuleSet {
            allow_single_wings: !rules.allow_single_wings,
            ..rules
        };
        add_rule_config_row(
            commands,
            page.rules,
            RuleConfigRow {
                label: "允许三带一",
                value: if rules.allow_single_wings {
                    "开启"
                } else {
                    "关闭"
                }
                .to_owned(),
                help: "关闭时三张只能不带或带一对，飞机只能不带或带对应数量的对子；开启后也允许三带一和飞机带单张。",
                editable: page.can_configure,
                previous: page.can_configure.then_some(toggled),
                next: page.can_configure.then_some(toggled),
            },
            assets,
        );
    }
}

fn mode_dropdown(selected: DoudizhuMode) -> CozyDropdown {
    CozyDropdown {
        label: selected.label(),
        toggle_action: UiAction::Doudizhu(DoudizhuUiAction::ToggleModeMenu),
        close_action: UiAction::Doudizhu(DoudizhuUiAction::CloseModeMenu),
        options: [DoudizhuMode::Three, DoudizhuMode::Four]
            .into_iter()
            .map(|mode| CozyDropdownOption {
                label: mode.label(),
                selected: mode == selected,
                action: UiAction::Doudizhu(DoudizhuUiAction::SelectMode(mode)),
            })
            .collect(),
    }
}
