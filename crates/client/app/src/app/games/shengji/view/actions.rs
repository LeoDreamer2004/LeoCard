use super::super::skin::add_shengji_button;
use super::super::{ShengjiUiAction, ShengjiUiState};
use crate::app::presentation::{GAME_SIDE_CONTROL_SCALE, PaperCardButtonTone};
use crate::app::runtime::UiAssets;
use crate::app::shell::UiAction;
use bevy::prelude::*;
use leocard_game_common::PlayerId;
use leocard_protocol::{ShengjiPhaseView, ShengjiSnapshot};
use leocard_shengji::{ShengjiFiveTrumpCrossingStage, forced_follow_cards};
use leocard_ui::{UiComposer, spawn_node};

pub(super) fn add_shengji_actions(
    commands: &mut UiComposer,
    hand_area: Entity,
    game: &ShengjiSnapshot,
    ui: &ShengjiUiState,
    assets: &UiAssets,
) {
    let actions = spawn_node(
        commands,
        hand_area,
        Node {
            position_type: PositionType::Absolute,
            left: percent(if cfg!(target_os = "android") { 18 } else { 25 }),
            right: percent(if cfg!(target_os = "android") { 18 } else { 25 }),
            top: px(0),
            height: px(54.0 * GAME_SIDE_CONTROL_SCALE),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            column_gap: px(10),
            ..default()
        },
        None,
    );
    match &game.phase {
        ShengjiPhaseView::Burying if game.dealer == Some(game.you) => {
            let count = ui.selected.len();
            let kitty_size = game.rules.kitty_size();
            let label = format!("埋底 {count}/{kitty_size}");
            add_shengji_button(
                commands,
                actions,
                &label,
                (count == kitty_size).then_some(UiAction::Shengji(ShengjiUiAction::SubmitCards)),
                assets,
                142.0,
                48.0,
                PaperCardButtonTone::Amber,
            );
        }
        ShengjiPhaseView::Burying => {}
        ShengjiPhaseView::BottomCopying { .. } => {}
        ShengjiPhaseView::BottomCopyBurying { player } if *player == game.you => {
            let count = ui.selected.len();
            let kitty_size = game.rules.kitty_size();
            let label = format!("埋底 {count}/{kitty_size}");
            add_shengji_button(
                commands,
                actions,
                &label,
                (count == kitty_size).then_some(UiAction::Shengji(ShengjiUiAction::SubmitCards)),
                assets,
                142.0,
                48.0,
                PaperCardButtonTone::Amber,
            );
        }
        ShengjiPhaseView::BottomCopyBurying { .. } => {}
        ShengjiPhaseView::FiveTrumpCrossing {
            stage: ShengjiFiveTrumpCrossingStage::Deciding,
            eligible,
            decided,
            ..
        } => {
            if eligible.contains(&game.you) && !decided.contains(&game.you) {
                let selected = game
                    .your_hand
                    .iter()
                    .filter(|card| ui.selected.contains(card))
                    .copied()
                    .collect::<Vec<_>>();
                let includes_all_trumps = game.trump.is_some_and(|trump| {
                    game.your_hand
                        .iter()
                        .filter(|card| trump.is_trump(**card))
                        .all(|card| selected.contains(card))
                });
                let label = format!("过江 {}/5", selected.len());
                add_shengji_button(
                    commands,
                    actions,
                    &label,
                    (selected.len() == 5 && includes_all_trumps)
                        .then_some(UiAction::Shengji(ShengjiUiAction::SubmitCards)),
                    assets,
                    142.0,
                    48.0,
                    PaperCardButtonTone::Indigo,
                );
                add_shengji_button(
                    commands,
                    actions,
                    "不过江",
                    Some(UiAction::Shengji(ShengjiUiAction::DeclineFiveTrumpCrossing)),
                    assets,
                    120.0,
                    48.0,
                    PaperCardButtonTone::Stone,
                );
            }
        }
        ShengjiPhaseView::FiveTrumpCrossing {
            stage: ShengjiFiveTrumpCrossingStage::Returning,
            crossing,
            returned,
            ..
        } => {
            let partner = PlayerId((game.you.0 + 2) % 4);
            let must_return = crossing.contains(&partner);
            if must_return && !returned.contains(&game.you) {
                let selected_count = game
                    .your_hand
                    .iter()
                    .filter(|card| ui.selected.contains(card))
                    .count();
                let label = format!("归还 {selected_count}/5");
                add_shengji_button(
                    commands,
                    actions,
                    &label,
                    (selected_count == 5)
                        .then_some(UiAction::Shengji(ShengjiUiAction::SubmitCards)),
                    assets,
                    142.0,
                    48.0,
                    PaperCardButtonTone::Indigo,
                );
            }
        }
        ShengjiPhaseView::Playing if game.current_player == Some(game.you) => {
            let required = game
                .trick
                .as_ref()
                .and_then(|trick| trick.plays.first())
                .map(|play| play.play.cards.len());
            let selection_ready = required.map_or_else(
                || !ui.selected.is_empty(),
                |required| ui.selected.len() == required,
            );
            add_shengji_button(
                commands,
                actions,
                "出牌",
                selection_ready.then_some(UiAction::Shengji(ShengjiUiAction::SubmitCards)),
                assets,
                142.0,
                48.0,
                PaperCardButtonTone::Indigo,
            );
            add_shengji_button(
                commands,
                actions,
                "提示",
                Some(UiAction::Shengji(ShengjiUiAction::Hint)),
                assets,
                142.0,
                48.0,
                PaperCardButtonTone::Jade,
            );
        }
        ShengjiPhaseView::Playing => {}
        ShengjiPhaseView::Finished { .. } => {}
        ShengjiPhaseView::BottomFlipping { .. } => {}
        ShengjiPhaseView::Redealing => {}
        ShengjiPhaseView::Dealing { .. } | ShengjiPhaseView::BiddingGrace { .. } => {}
    }
}

pub(crate) fn select_forced_shengji_follow_cards(game: &ShengjiSnapshot, ui: &mut ShengjiUiState) {
    if !matches!(game.phase, ShengjiPhaseView::Playing) || game.current_player != Some(game.you) {
        return;
    }
    let Some(trump) = game.trump else {
        return;
    };
    let Some(lead) = game
        .trick
        .as_ref()
        .and_then(|trick| trick.plays.first())
        .map(|play| &play.play)
    else {
        return;
    };
    ui.selected
        .extend(forced_follow_cards(&game.your_hand, lead, trump));
}
