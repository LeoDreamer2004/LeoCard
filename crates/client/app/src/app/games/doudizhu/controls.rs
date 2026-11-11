use super::{DoudizhuUiAction, DoudizhuUiState};
use crate::app::presentation::{
    GAME_SIDE_CONTROL_SCALE, PaperCardButtonTone, TableActionButton, TableButtonSkin,
    add_no_legal_response_hint,
};
use crate::app::runtime::UiAssets;
use crate::app::shell::UiAction;
use bevy::prelude::*;
use leocard_doudizhu::{DoudizhuAction, DoudizhuPhase, classify, hint};
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
            min_height: px(54.0 * GAME_SIDE_CONTROL_SCALE),
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
            .any(|player| player.player.id == game.you && player.auto_play)
    {
        return;
    }
    let mut button = |label: &str, action: Option<DoudizhuUiAction>, tone| {
        TableActionButton {
            size: Vec2::new(142.0, 48.0),
            ..TableActionButton::new(
                label,
                action.map(UiAction::Doudizhu),
                TableButtonSkin::PaperCard(tone),
            )
        }
        .scaled(GAME_SIDE_CONTROL_SCALE)
        .mount(commands, row, assets);
    };
    if game.phase == DoudizhuPhase::Bidding {
        button(
            "不叫",
            Some(DoudizhuUiAction::Act(DoudizhuAction::Bid(0))),
            PaperCardButtonTone::Stone,
        );
        for score in 1..=3 {
            button(
                &format!("{score} 分"),
                (score > game.highest_bid)
                    .then_some(DoudizhuUiAction::Act(DoudizhuAction::Bid(score))),
                PaperCardButtonTone::Rust,
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
            PaperCardButtonTone::Rust,
        );
        button(
            "不出",
            game.table
                .is_some()
                .then_some(DoudizhuUiAction::Act(DoudizhuAction::Pass)),
            PaperCardButtonTone::Stone,
        );
        button(
            "提示",
            can_hint.then_some(DoudizhuUiAction::Hint),
            PaperCardButtonTone::Jade,
        );
        if game.table.is_some() && !can_hint {
            add_no_legal_response_hint(commands, row, "没有牌能大过上家", assets);
        }
    }
}
