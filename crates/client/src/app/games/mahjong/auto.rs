//! 麻将局内的快捷开关与本地自动操作。

use super::{MahjongAutomaticActionKey, MahjongUiAction, MahjongUiState};
use crate::app::presentation::{DrawerSwitch, SwitchDrawer, add_switch_drawer};
use crate::app::runtime::{ClientResource, UiAssets};
use crate::app::shell::{UiAction, game_command};
use bevy::prelude::*;
use leocard_mahjong::{MahjongClaim, MahjongClaimOption};
use leocard_protocol::{MahjongCommand, MahjongPhaseView, MahjongSnapshot};

pub(super) fn render_mahjong_auto_drawer(
    commands: &mut Commands,
    table: Entity,
    ui: &MahjongUiState,
    false_win_allowed: bool,
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
            enabled: ui.auto_draw_discard,
            action: UiAction::Mahjong(MahjongUiAction::ToggleAutoDrawDiscard),
        },
    ];
    add_switch_drawer(
        commands,
        table,
        SwitchDrawer {
            expanded: ui.auto_drawer_open,
            toggle_action: UiAction::Mahjong(MahjongUiAction::ToggleAutoDrawer),
            switches: &switches[usize::from(false_win_allowed)..],
        },
        assets,
    );
}

pub(super) fn apply_automatic_mahjong_action(
    mut client: Option<ResMut<ClientResource>>,
    mut ui: ResMut<MahjongUiState>,
) {
    let Some(client) = client.as_deref_mut() else {
        ui.last_automatic_action = None;
        return;
    };
    if let Some(game) = client.0.model().mahjong_game() {
        ui.begin_hand(game.match_id, game.sequence_index);
    }
    let decision = client
        .0
        .model()
        .mahjong_game()
        .and_then(|game| choose_automatic_action(game, &ui));
    let Some((key, command)) = decision else {
        ui.last_automatic_action = None;
        return;
    };
    if ui.last_automatic_action == Some(key) {
        return;
    }
    if client.0.send(game_command(command)) {
        ui.last_automatic_action = Some(key);
    }
}

fn choose_automatic_action(
    game: &MahjongSnapshot,
    ui: &MahjongUiState,
) -> Option<(MahjongAutomaticActionKey, MahjongCommand)> {
    if game
        .players
        .iter()
        .find(|player| player.id == game.you)
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
    ui.auto_draw_discard.then_some((
        MahjongAutomaticActionKey::Discard(game.match_id, game.sequence_index, drawn),
        MahjongCommand::Discard { tile: drawn },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use leocard_mahjong::{MahjongRuleSet, MahjongSuit, MahjongTile, MahjongTileKind, MahjongWind};
    use leocard_protocol::{MahjongPendingClaimView, MatchId, PlayerId};

    fn snapshot() -> MahjongSnapshot {
        let you = PlayerId(0);
        let drawn = MahjongTile::new(MahjongTileKind::suited(MahjongSuit::Characters, 5), 0);
        MahjongSnapshot {
            match_id: MatchId([1; 16]),
            host_port: 52300,
            you,
            host: you,
            rules: MahjongRuleSet::default(),
            players: Vec::new(),
            your_hand: vec![drawn],
            your_drawn_tile: Some(drawn),
            discards: Vec::new(),
            dealer: you,
            prevalent_wind: MahjongWind::East,
            sequence_index: 0,
            current_player: you,
            wall_len: 60,
            match_scores: [0; 4],
            pending_claim: None,
            can_self_draw: false,
            can_legal_self_draw: false,
            concealed_kong_options: Vec::new(),
            added_kong_options: Vec::new(),
            phase: MahjongPhaseView::Playing,
        }
    }

    #[test]
    fn auto_options_reset_only_when_a_new_hand_begins() {
        let mut ui = MahjongUiState::default();
        let match_id = MatchId([1; 16]);
        ui.begin_hand(match_id, 0);
        ui.auto_drawer_open = true;
        ui.auto_win = true;
        ui.no_claim = true;
        ui.auto_draw_discard = true;
        ui.begin_hand(match_id, 0);
        assert!(ui.auto_win && ui.no_claim && ui.auto_draw_discard && ui.auto_drawer_open);

        ui.begin_hand(match_id, 1);
        assert!(!ui.auto_win && !ui.no_claim && !ui.auto_draw_discard && !ui.auto_drawer_open);
        ui.auto_win = true;
        ui.begin_hand(MatchId([2; 16]), 0);
        assert!(!ui.auto_win);
    }

    #[test]
    fn auto_win_uses_only_legal_win_and_takes_priority_over_draw_discard() {
        let mut game = snapshot();
        let ui = MahjongUiState {
            auto_win: true,
            auto_draw_discard: true,
            ..default()
        };
        game.can_self_draw = true;
        assert_eq!(
            choose_automatic_action(&game, &ui).map(|(_, command)| command),
            Some(MahjongCommand::Discard {
                tile: game.your_drawn_tile.unwrap(),
            })
        );
        game.can_legal_self_draw = true;
        assert_eq!(
            choose_automatic_action(&game, &ui).map(|(_, command)| command),
            Some(MahjongCommand::DeclareSelfDraw)
        );
    }

    #[test]
    fn no_claim_passes_calls_but_keeps_manual_false_win_available() {
        let mut game = snapshot();
        let ui = MahjongUiState {
            auto_win: true,
            no_claim: true,
            ..default()
        };
        let tile = MahjongTile::new(MahjongTileKind::suited(MahjongSuit::Dots, 3), 0);
        game.phase = MahjongPhaseView::WaitingForClaims;
        game.pending_claim = Some(MahjongPendingClaimView {
            source: PlayerId(1),
            tile,
            robbing_kong: false,
            your_options: vec![MahjongClaimOption::Pung],
            can_legal_win: false,
            your_response: None,
            waiting_for: vec![game.you],
        });
        assert_eq!(
            choose_automatic_action(&game, &ui).map(|(_, command)| command),
            Some(MahjongCommand::RespondToClaim {
                claim: MahjongClaim::Pass,
            })
        );
        let pending = game.pending_claim.as_mut().unwrap();
        pending.your_options.push(MahjongClaimOption::Win);
        assert!(choose_automatic_action(&game, &ui).is_none());
        game.pending_claim.as_mut().unwrap().can_legal_win = true;
        assert_eq!(
            choose_automatic_action(&game, &ui).map(|(_, command)| command),
            Some(MahjongCommand::RespondToClaim {
                claim: MahjongClaim::Win,
            })
        );
    }

    #[test]
    fn false_win_rule_disables_auto_win_even_if_previously_enabled() {
        let mut game = snapshot();
        let ui = MahjongUiState {
            auto_win: true,
            auto_draw_discard: true,
            ..default()
        };
        game.rules.false_win = true;
        game.can_legal_self_draw = true;
        assert_eq!(
            choose_automatic_action(&game, &ui).map(|(_, command)| command),
            Some(MahjongCommand::Discard {
                tile: game.your_drawn_tile.unwrap(),
            })
        );

        game.phase = MahjongPhaseView::WaitingForClaims;
        game.pending_claim = Some(MahjongPendingClaimView {
            source: PlayerId(1),
            tile: game.your_drawn_tile.unwrap(),
            robbing_kong: false,
            your_options: vec![MahjongClaimOption::Win],
            can_legal_win: true,
            your_response: None,
            waiting_for: vec![game.you],
        });
        assert!(choose_automatic_action(&game, &ui).is_none());
    }
}
