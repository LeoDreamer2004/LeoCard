use super::super::{ShengjiUiAction, ShengjiUiState};
use super::add_shengji_bid_strip;
use crate::app::presentation::spawn_node;
use crate::app::runtime::UiAssets;
use crate::app::shell::{
    CozyButtonVariant, UiAction, add_cozy_button_variant, add_cozy_disabled_button,
};
use bevy::prelude::*;
use leocard_protocol::{
    PlayerId, ShengjiFiveTrumpCrossingStage, ShengjiPhaseView, ShengjiSnapshot,
};
use leocard_shengji::forced_follow_cards;

pub(super) fn add_shengji_actions(
    commands: &mut Commands,
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
            left: percent(25),
            right: percent(25),
            top: px(0),
            height: px(54),
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
            let label = format!("埋底 ({count}/{kitty_size})");
            if count == kitty_size {
                add_cozy_button_variant(
                    commands,
                    actions,
                    &label,
                    UiAction::Shengji(ShengjiUiAction::SubmitCards),
                    assets,
                    px(142),
                    48.0,
                    CozyButtonVariant::Cool,
                );
            } else {
                add_cozy_disabled_button(commands, actions, &label, assets, px(142), 48.0);
            }
        }
        ShengjiPhaseView::Burying => {}
        ShengjiPhaseView::BottomCopying { player, .. } if *player == game.you => {
            add_shengji_bid_strip(commands, actions, game, true, assets);
            add_cozy_button_variant(
                commands,
                actions,
                "不抄底",
                UiAction::Shengji(ShengjiUiAction::DeclineBottomCopy),
                assets,
                px(120),
                48.0,
                CozyButtonVariant::Neutral,
            );
        }
        ShengjiPhaseView::BottomCopying { .. } => {}
        ShengjiPhaseView::BottomCopyBurying { player } if *player == game.you => {
            let count = ui.selected.len();
            let kitty_size = game.rules.kitty_size();
            let label = format!("再埋底 ({count}/{kitty_size})");
            if count == kitty_size {
                add_cozy_button_variant(
                    commands,
                    actions,
                    &label,
                    UiAction::Shengji(ShengjiUiAction::SubmitCards),
                    assets,
                    px(142),
                    48.0,
                    CozyButtonVariant::Cool,
                );
            } else {
                add_cozy_disabled_button(commands, actions, &label, assets, px(142), 48.0);
            }
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
                let label = format!("过江 ({}/5)", selected.len());
                if selected.len() == 5 && includes_all_trumps {
                    add_cozy_button_variant(
                        commands,
                        actions,
                        &label,
                        UiAction::Shengji(ShengjiUiAction::SubmitCards),
                        assets,
                        px(142),
                        48.0,
                        CozyButtonVariant::Cool,
                    );
                } else {
                    add_cozy_disabled_button(commands, actions, &label, assets, px(142), 48.0);
                }
                add_cozy_button_variant(
                    commands,
                    actions,
                    "不过江",
                    UiAction::Shengji(ShengjiUiAction::DeclineFiveTrumpCrossing),
                    assets,
                    px(120),
                    48.0,
                    CozyButtonVariant::Neutral,
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
                let label = format!("归还 ({selected_count}/5)");
                if selected_count == 5 {
                    add_cozy_button_variant(
                        commands,
                        actions,
                        &label,
                        UiAction::Shengji(ShengjiUiAction::SubmitCards),
                        assets,
                        px(142),
                        48.0,
                        CozyButtonVariant::Cool,
                    );
                } else {
                    add_cozy_disabled_button(commands, actions, &label, assets, px(142), 48.0);
                }
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
            if !selection_ready {
                add_cozy_disabled_button(commands, actions, "出牌", assets, px(142), 48.0);
            } else {
                add_cozy_button_variant(
                    commands,
                    actions,
                    "出牌",
                    UiAction::Shengji(ShengjiUiAction::SubmitCards),
                    assets,
                    px(142),
                    48.0,
                    CozyButtonVariant::Cool,
                );
            }
            add_cozy_button_variant(
                commands,
                actions,
                "提示",
                UiAction::Shengji(ShengjiUiAction::Hint),
                assets,
                px(142),
                48.0,
                CozyButtonVariant::Neutral,
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
