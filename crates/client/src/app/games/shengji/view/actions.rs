use super::*;
use leocard_protocol::{
    PlayerId, ShengjiFiveTrumpCrossingStage, ShengjiPhaseView, ShengjiSnapshot,
};
use leocard_shengji::ShengjiSuit;
use leocard_shengji::forced_follow_cards;

pub fn add_shengji_actions(
    commands: &mut Commands,
    hand_area: Entity,
    game: &ShengjiSnapshot,
    ui: &UiState,
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
            height: px(48),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            column_gap: px(10),
            ..default()
        },
        None,
    );
    match &game.phase {
        ShengjiPhaseView::Burying if game.dealer == Some(game.you) => {
            let count = ui.shengji.selected.len();
            let kitty_size = game.rules.kitty_size();
            if count == kitty_size {
                add_action_button(
                    commands,
                    actions,
                    "埋下底牌",
                    UiAction::Shengji(ShengjiUiAction::SubmitCards),
                    ButtonKind::Primary,
                    assets,
                );
            } else {
                add_disabled_action_button(
                    commands,
                    actions,
                    &format!("请选择 {kitty_size} 张底牌（{count}/{kitty_size}）"),
                    assets,
                );
            }
        }
        ShengjiPhaseView::Burying => {
            add_text(commands, actions, "等待庄家埋底…", 15.0, MUTED, assets);
        }
        ShengjiPhaseView::BottomCopying { player, .. } if *player == game.you => {
            for (label, target, color) in [
                ("♦", Some(ShengjiSuit::Diamond), DANGER),
                ("♣", Some(ShengjiSuit::Club), TEXT),
                ("♥", Some(ShengjiSuit::Heart), DANGER),
                ("♠", Some(ShengjiSuit::Spade), TEXT),
                ("无主", None, ACCENT),
            ] {
                let cards = shengji_declaration_candidate(game, target);
                add_shengji_bid_button(commands, actions, label, cards, color, true, assets);
            }
            add_action_button(
                commands,
                actions,
                "不抄底",
                UiAction::Shengji(ShengjiUiAction::DeclineBottomCopy),
                ButtonKind::Secondary,
                assets,
            );
        }
        ShengjiPhaseView::BottomCopying { .. } => {
            add_text(commands, actions, "等待抄底…", 15.0, MUTED, assets);
        }
        ShengjiPhaseView::BottomCopyBurying { player } if *player == game.you => {
            let count = ui.shengji.selected.len();
            let kitty_size = game.rules.kitty_size();
            if count == kitty_size {
                add_action_button(
                    commands,
                    actions,
                    "重新埋底",
                    UiAction::Shengji(ShengjiUiAction::SubmitCards),
                    ButtonKind::Primary,
                    assets,
                );
            } else {
                add_disabled_action_button(
                    commands,
                    actions,
                    &format!("选择 {kitty_size} 张重新埋底（{count}/{kitty_size}）"),
                    assets,
                );
            }
        }
        ShengjiPhaseView::BottomCopyBurying { .. } => {
            add_text(commands, actions, "等待抄底…", 15.0, MUTED, assets);
        }
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
                    .filter(|card| ui.shengji.selected.contains(card))
                    .copied()
                    .collect::<Vec<_>>();
                let includes_all_trumps = game.trump.is_some_and(|trump| {
                    game.your_hand
                        .iter()
                        .filter(|card| trump.is_trump(**card))
                        .all(|card| selected.contains(card))
                });
                if selected.len() == 5 && includes_all_trumps {
                    add_action_button(
                        commands,
                        actions,
                        "五主过江",
                        UiAction::Shengji(ShengjiUiAction::SubmitCards),
                        ButtonKind::Primary,
                        assets,
                    );
                } else {
                    add_disabled_action_button(
                        commands,
                        actions,
                        &format!("选择全部主牌并补足五张（{}/5）", selected.len()),
                        assets,
                    );
                }
                add_action_button(
                    commands,
                    actions,
                    "不过江",
                    UiAction::Shengji(ShengjiUiAction::DeclineFiveTrumpCrossing),
                    ButtonKind::Secondary,
                    assets,
                );
            } else if decided.contains(&game.you) {
                add_text(
                    commands,
                    actions,
                    "已选择，等待其他玩家…",
                    15.0,
                    MUTED,
                    assets,
                );
            } else {
                add_text(
                    commands,
                    actions,
                    "等待可过江玩家选择…",
                    15.0,
                    MUTED,
                    assets,
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
                    .filter(|card| ui.shengji.selected.contains(card))
                    .count();
                if selected_count == 5 {
                    add_action_button(
                        commands,
                        actions,
                        "归还五张",
                        UiAction::Shengji(ShengjiUiAction::SubmitCards),
                        ButtonKind::Primary,
                        assets,
                    );
                } else {
                    add_disabled_action_button(
                        commands,
                        actions,
                        &format!("选择五张归还牌（{selected_count}/5）"),
                        assets,
                    );
                }
            } else if returned.contains(&game.you) {
                add_text(
                    commands,
                    actions,
                    "已归还，等待其他玩家…",
                    15.0,
                    MUTED,
                    assets,
                );
            } else {
                add_text(commands, actions, "等待对家完成过江…", 15.0, MUTED, assets);
            }
        }
        ShengjiPhaseView::Playing if game.current_player == Some(game.you) => {
            let required = game
                .trick
                .as_ref()
                .and_then(|trick| trick.plays.first())
                .map(|play| play.play.cards.len());
            let selection_ready = required.map_or_else(
                || !ui.shengji.selected.is_empty(),
                |required| ui.shengji.selected.len() == required,
            );
            if !selection_ready {
                add_disabled_action_button(commands, actions, "出牌", assets);
            } else {
                add_action_button(
                    commands,
                    actions,
                    "出牌",
                    UiAction::Shengji(ShengjiUiAction::SubmitCards),
                    ButtonKind::Primary,
                    assets,
                );
            }
            add_action_button(
                commands,
                actions,
                "提示",
                UiAction::Shengji(ShengjiUiAction::Hint),
                ButtonKind::Secondary,
                assets,
            );
        }
        ShengjiPhaseView::Playing => {
            add_text(commands, actions, "等待其他玩家出牌…", 15.0, MUTED, assets);
        }
        ShengjiPhaseView::Finished { .. } => {}
        ShengjiPhaseView::BottomFlipping { .. } => {}
        ShengjiPhaseView::Redealing => {
            add_text(
                commands,
                actions,
                "无人亮主，正在重新发牌…",
                15.0,
                ACCENT,
                assets,
            );
        }
        ShengjiPhaseView::Dealing { .. } | ShengjiPhaseView::BiddingGrace { .. } => {}
    }
}

pub fn select_forced_shengji_follow_cards(game: &ShengjiSnapshot, ui: &mut UiState) {
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
    ui.shengji
        .selected
        .extend(forced_follow_cards(&game.your_hand, lead, trump));
}
