use super::actions::GuandanUiAction;
use crate::app::presentation::{EditableRuleSet, RuleConfigRow, add_rule_config_row};
use crate::app::runtime::{AvatarImages, ClientResource, UiAssets};
use crate::app::shell::{LobbyPage, LobbyPageConfig, UiAction};
use bevy::prelude::*;
use leocard_guandan::{GuandanRuleSet, rank_label};
use leocard_protocol::LobbySnapshot;
use leocard_ui::UiComposer;

impl EditableRuleSet for GuandanRuleSet {
    const ROW_HEIGHT: f32 = 38.0;
    const VALUE_WIDTH: f32 = 110.0;

    fn update_action(self) -> UiAction {
        UiAction::Guandan(GuandanUiAction::UpdateRules(self))
    }
}

pub(super) fn render_lobby(
    commands: &mut UiComposer,
    root: Entity,
    client: &ClientResource,
    lobby: &LobbySnapshot,
    assets: &UiAssets,
    avatars: &AvatarImages,
) {
    let rules = *lobby.rules.guandan().expect("guandan lobby rules");
    let page = LobbyPage::spawn(
        commands,
        root,
        client,
        lobby,
        assets,
        avatars,
        LobbyPageConfig::new("掼蛋配置"),
    );
    add_rule_config_row(
        commands,
        page.rules,
        RuleConfigRow {
            label: "起始级牌",
            value: rank_label(rules.starting_level).to_owned(),
            help: "四人两队，对家为搭档。两副牌每人 27 张，从设定级牌打到 A。红桃级牌可以百搭，双大王可以抗贡。",
            editable: page.can_configure,
            previous: (page.can_configure && rules.starting_level > 2).then_some(GuandanRuleSet {
                starting_level: rules.starting_level.saturating_sub(1),
            }),
            next: (page.can_configure && rules.starting_level < 14).then_some(GuandanRuleSet {
                starting_level: rules.starting_level.saturating_add(1),
            }),
        },
        assets,
    );
}
