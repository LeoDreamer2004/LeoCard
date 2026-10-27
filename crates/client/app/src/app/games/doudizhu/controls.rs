use super::{DoudizhuUiAction, DoudizhuUiState};
use crate::app::presentation::{
    GameButtonSpec, GameButtonTone, add_no_legal_response_hint, add_textured_game_button,
};
use crate::app::runtime::UiAssets;
use crate::app::shell::UiAction;
use bevy::prelude::*;
use leocard_doudizhu::{Action, Phase, classify, hint};
use leocard_protocol::DoudizhuSnapshot;
use leocard_ui::{UiComposer, spawn_node};

pub(super) fn render(
    commands: &mut UiComposer,
    parent: Entity,
    game: &DoudizhuSnapshot,
    ui: &DoudizhuUiState,
    assets: &UiAssets,
) {
    let row = spawn_node(
        commands,
        parent,
        Node {
            height: px(54),
            column_gap: px(10),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            position_type: PositionType::Relative,
            ..default()
        },
        None,
    );
    if game.result.is_some()
        || game.current != game.you
        || game
            .players
            .iter()
            .any(|player| player.id == game.you && player.auto_play)
    {
        return;
    }
    let mut button = |label: &str, action: Option<DoudizhuUiAction>, tone| {
        add_textured_game_button(
            commands,
            row,
            assets,
            GameButtonSpec::compact(assets, label, action.map(UiAction::Doudizhu), tone),
        );
    };
    if game.phase == Phase::Bidding {
        button(
            "不叫",
            Some(DoudizhuUiAction::Act(Action::Bid(0))),
            GameButtonTone::Pass,
        );
        for score in 1..=3 {
            button(
                &format!("{score} 分"),
                (score > game.highest_bid).then_some(DoudizhuUiAction::Act(Action::Bid(score))),
                GameButtonTone::Play,
            );
        }
    } else {
        let cards: Vec<_> = ui.selected.iter().copied().collect();
        let legal = classify(&cards, game.rules).is_some_and(|play| {
            game.table
                .as_ref()
                .is_none_or(|(_, previous)| play.beats(previous))
        });
        let can_hint = hint(
            &game.your_hand,
            game.table.as_ref().map(|(_, play)| play),
            game.rules,
        )
        .is_some();
        button(
            "出牌",
            legal.then_some(DoudizhuUiAction::Play),
            GameButtonTone::Play,
        );
        button(
            "不出",
            game.table
                .is_some()
                .then_some(DoudizhuUiAction::Act(Action::Pass)),
            GameButtonTone::Pass,
        );
        button(
            "提示",
            can_hint.then_some(DoudizhuUiAction::Hint),
            GameButtonTone::Hint,
        );
        if game.table.is_some() && !can_hint {
            add_no_legal_response_hint(commands, row, assets);
        }
    }
}
