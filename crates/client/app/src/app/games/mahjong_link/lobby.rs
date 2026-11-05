use super::{MahjongLinkUiAction, state::MahjongLinkUiState};
use crate::app::games::GameLobbyContext;
use crate::app::presentation::{MUTED, TEXT, add_text};
use crate::app::shell::{CozyDropdown, CozyDropdownOption, LobbyPage, LobbyPageConfig, UiAction};
use bevy::prelude::*;
use leocard_mahjong_link::{MahjongLinkMode, MahjongLinkRuleSet};
use leocard_protocol::LobbySnapshot;
use leocard_ui::UiComposer;

pub(super) fn render(
    commands: &mut UiComposer,
    root: Entity,
    lobby: &LobbySnapshot,
    ui: &MahjongLinkUiState,
    context: GameLobbyContext<'_>,
) {
    let rules = *lobby.rules.mahjong_link().expect("mahjong link rules");
    let modes = [
        (MahjongLinkMode::TwoPlayers, "双人对局"),
        (MahjongLinkMode::FourPlayers, "四人对局"),
    ];
    let dropdown = CozyDropdown {
        label: if rules.player_count() == 2 {
            "双人对局"
        } else {
            "四人对局"
        },
        toggle_action: UiAction::MahjongLink(MahjongLinkUiAction::ToggleModeMenu),
        close_action: UiAction::MahjongLink(MahjongLinkUiAction::CloseModeMenu),
        options: modes
            .into_iter()
            .map(|(mode, label)| CozyDropdownOption {
                label,
                selected: rules.mode == mode,
                action: UiAction::MahjongLink(MahjongLinkUiAction::UpdateRules(
                    MahjongLinkRuleSet { mode },
                )),
            })
            .collect(),
    };
    let page = LobbyPage::spawn(
        commands,
        root,
        context.client,
        lobby,
        context.assets,
        context.avatars,
        LobbyPageConfig::new("麻将连连看")
            .description("8 行 17 列铺满 136 张麻将，按座位轮流操作。")
            .mode(dropdown, ui.mode_menu_open),
    );
    for text in [
        "每回合可直接消除一对，或先推动一次再消除一对。消除或跳过后换人，每回合最多推动一次。",
        "相同牌以最多两次转弯的连线相连即可消除，连线可绕外沿，不可穿牌。分别向上、向下露出的相同牌也可直接配对。",
        "推牌须从一排最左端向右推，或最右端向左推。距离任选，碰到同排牌时继续连带推动，任何牌都不能越界。",
        "每对只计一次：字牌 10 分，万、筒、条按数字计 1 至 9 分。",
        "牌全部消除，或全员连续跳过后结算。分数最高者获胜，同分并列。",
    ] {
        add_text(commands, page.rules, text, 18.0, TEXT, context.assets);
    }
    add_text(
        commands,
        page.rules,
        "房主可在右上方切换双人、四人。切换为双人前，房内人数须不超过两人。",
        16.0,
        MUTED,
        context.assets,
    );
}
