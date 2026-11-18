use super::actions::DrawGuessUiAction;
use super::libraries::{
    self, LibraryView, WordLibraryManager, WordLibraryRoom, WordLibraryUiAction,
};
use crate::app::games::GameLobbyContext;
use crate::app::presentation::{EditableRuleSet, RuleConfigRow, add_rule_config_row};
use crate::app::shell::{LobbyPage, LobbyPageConfig, UiAction, add_cozy_button};
use bevy::prelude::*;
use leocard_draw_guess::DrawGuessRuleSet;
use leocard_protocol::LobbySnapshot;
use leocard_ui::UiComposer;

impl EditableRuleSet for DrawGuessRuleSet {
    const ROW_HEIGHT: f32 = 38.0;
    const VALUE_WIDTH: f32 = 92.0;

    fn update_action(self) -> UiAction {
        UiAction::DrawGuess(DrawGuessUiAction::UpdateRules(self))
    }
}

pub(super) fn render(
    commands: &mut UiComposer,
    root: Entity,
    lobby: &LobbySnapshot,
    context: GameLobbyContext<'_>,
    libraries: &WordLibraryManager,
    room: &WordLibraryRoom,
) {
    let rules = *lobby.rules.draw_guess().expect("drawing lobby rules");
    let page = LobbyPage::spawn(
        commands,
        root,
        context.client,
        lobby,
        context.assets,
        context.avatars,
        LobbyPageConfig::new("你画我猜配置"),
    );
    for (label, value, help, previous, next) in [
        (
            "轮数",
            rules.rounds.to_string(),
            "每轮所有玩家各画一次，整场积分按总分排名结算，同分并列。3 轮及以上积分翻倍。",
            (rules.rounds > 1).then_some(DrawGuessRuleSet {
                rounds: rules.rounds.saturating_sub(1),
                ..rules
            }),
            (rules.rounds < 5).then_some(DrawGuessRuleSet {
                rounds: rules.rounds + 1,
                ..rules
            }),
        ),
        (
            "时限",
            format!("{} 秒", rules.drawing_seconds),
            "每次绘画与猜词的时限，所有人猜中后提前结束。机器人不绘画，在剩余 5 秒时猜中答案。",
            (rules.drawing_seconds > 30).then_some(DrawGuessRuleSet {
                drawing_seconds: rules.drawing_seconds.saturating_sub(30),
                ..rules
            }),
            (rules.drawing_seconds < 180).then_some(DrawGuessRuleSet {
                drawing_seconds: rules.drawing_seconds + 30,
                ..rules
            }),
        ),
    ] {
        add_rule_config_row(
            commands,
            page.rules,
            RuleConfigRow {
                label,
                value,
                help,
                previous: previous.filter(|_| page.can_configure),
                next: next.filter(|_| page.can_configure),
            },
            context.assets,
        );
    }
    add_cozy_button(
        commands,
        page.rules,
        "词库管理",
        UiAction::DrawGuess(DrawGuessUiAction::Library(WordLibraryUiAction::Open)),
        context.assets,
        percent(100),
        48.0,
    );
    libraries::render(
        commands,
        root,
        LibraryView {
            manager: libraries,
            room,
            host: page.can_configure,
            assets: context.assets,
        },
    );
}
