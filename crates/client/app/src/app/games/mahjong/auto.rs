//! 麻将局内的快捷开关与本地自动操作。
use super::{MahjongAutomaticActionKey, MahjongUiAction, MahjongUiState};
use crate::app::presentation::add_switch_drawer;
use crate::app::runtime::{ClientResource, UiAssets};
use crate::app::shell::{UiAction, game_command};
use bevy::prelude::*;
use leocard_mahjong::{MahjongClaim, MahjongClaimOption};
use leocard_protocol::{GameSnapshot, MahjongCommand, MahjongPhaseView, MahjongSnapshot};
use leocard_ui::{DrawerSwitch, SwitchDrawer, UiComposer};

// The 0.42s draw animation finishes before the remaining time is spent showing the upright tile.
const READY_DRAW_DISCARD_DELAY: f64 = 0.9;

pub(super) fn render_mahjong_auto_drawer(
    commands: &mut UiComposer,
    table: Entity,
    ui: &MahjongUiState,
    game: &MahjongSnapshot,
    assets: &UiAssets,
) {
    let switches = [
        DrawerSwitch {
            short: "和",
            label: "自动和牌",
            enabled: ui.auto_win,
            action: UiAction::Mahjong(MahjongUiAction::ToggleAutoWin),
        },
        DrawerSwitch {
            short: "鸣",
            label: "不再鸣牌",
            enabled: ui.no_claim,
            action: UiAction::Mahjong(MahjongUiAction::ToggleNoClaim),
        },
        DrawerSwitch {
            short: "打",
            label: "自动摸打",
            enabled: ui.auto_draw_discard || game.your_declared_ready(),
            action: UiAction::Mahjong(MahjongUiAction::ToggleAutoDrawDiscard),
        },
        DrawerSwitch {
            short: "花",
            label: "自动补花",
            enabled: game.your_auto_flower,
            action: UiAction::Mahjong(MahjongUiAction::SetAutoFlower(!game.your_auto_flower)),
        },
    ];
    let switches = switches
        .into_iter()
        .filter(|switch| {
            !(game.rules.false_win
                && matches!(
                    &switch.action,
                    UiAction::Mahjong(MahjongUiAction::ToggleAutoWin)
                ))
                && !(game.your_declared_ready()
                    && matches!(
                        &switch.action,
                        UiAction::Mahjong(MahjongUiAction::ToggleAutoDrawDiscard)
                    ))
        })
        .collect::<Vec<_>>();
    add_switch_drawer(
        commands,
        table,
        SwitchDrawer {
            expanded: ui.auto_drawer_open,
            toggle_action: UiAction::Mahjong(MahjongUiAction::ToggleAutoDrawer),
            switches: &switches,
        },
        assets,
    );
}

pub(super) fn apply_automatic_mahjong_action(
    mut client: Option<ResMut<ClientResource>>,
    mut ui: ResMut<MahjongUiState>,
    time: Res<Time>,
) {
    let Some(client) = client.as_deref_mut() else {
        ui.last_automatic_action = None;
        ui.ready_draw.clear();
        return;
    };
    let game = client
        .0
        .model()
        .game_snapshot()
        .and_then(GameSnapshot::mahjong);
    if let Some(game) = game {
        ui.begin_hand(game.match_id, game.sequence_index);
    }
    let now = time.elapsed_secs_f64();
    let ready_draw = game
        .filter(|game| {
            game.your_declared_ready()
                && matches!(game.phase, MahjongPhaseView::Playing)
                && game.current_player == game.you
        })
        .and_then(|game| {
            game.your_drawn_tile.map(|tile| {
                MahjongAutomaticActionKey::Discard(game.match_id, game.sequence_index, tile)
            })
        });
    if let Some(key) = ready_draw {
        if ui.ready_draw.observe(key) {
            ui.ready_draw.state = now;
        }
    } else {
        ui.ready_draw.clear();
    }
    // Observe the draw even when a win/kong prompt pauses automation, so UI changes
    // cannot restart the timer. Only a discard with this exact key waits below.
    let decision = game.and_then(|game| choose_automatic_action(game, &ui));
    let Some((key, command)) = decision else {
        ui.last_automatic_action = None;
        return;
    };
    if ui.last_automatic_action == Some(key) {
        return;
    }
    if ui.ready_draw.key == Some(key) && now - ui.ready_draw.state < READY_DRAW_DISCARD_DELAY {
        return;
    }
    if client.0.send(game_command(command)) {
        ui.last_automatic_action = Some(key);
    }
}

pub(super) fn choose_automatic_action(
    game: &MahjongSnapshot,
    ui: &MahjongUiState,
) -> Option<(MahjongAutomaticActionKey, MahjongCommand)> {
    if game
        .players
        .iter()
        .find(|player| player.player.id == game.you)
        .is_some_and(|player| player.auto_play)
    {
        return None;
    }
    if let Some(pending) = &game.pending_claim {
        if pending.your_response.is_some() {
            return None;
        }
        let can_win = pending.your_options.contains(&MahjongClaimOption::Win);
        let claim = if !game.rules.false_win && pending.can_legal_win && ui.auto_win {
            MahjongClaim::Win
        } else if ui.no_claim && !can_win && !pending.your_options.is_empty() {
            MahjongClaim::Pass
        } else {
            return None;
        };
        return Some((
            MahjongAutomaticActionKey::Claim(
                game.match_id,
                game.sequence_index,
                pending.tile,
                claim,
            ),
            MahjongCommand::RespondToClaim { claim },
        ));
    }
    if !matches!(game.phase, MahjongPhaseView::Playing) || game.current_player != game.you {
        return None;
    }
    let drawn = game.your_drawn_tile?;
    if !game.rules.false_win && ui.auto_win && game.can_legal_self_draw {
        return Some((
            MahjongAutomaticActionKey::SelfDraw(game.match_id, game.sequence_index, drawn),
            MahjongCommand::DeclareSelfDraw,
        ));
    }
    // 自动和牌已优先处理，剩余的手动吃碰杠和和牌机会仍须暂停自动摸打。
    if ui.declaring_ready
        || game.can_legal_self_draw
        || game.can_self_draw
        || !game.concealed_kong_options.is_empty()
        || (!ui.no_claim && !game.added_kong_options.is_empty())
    {
        return None;
    }
    (ui.auto_draw_discard || game.your_declared_ready()).then_some((
        MahjongAutomaticActionKey::Discard(game.match_id, game.sequence_index, drawn),
        MahjongCommand::Discard { tile: drawn },
    ))
}
