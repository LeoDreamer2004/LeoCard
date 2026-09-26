//! 麻将局内的快捷开关与本地自动操作。

use super::{MahjongAutomaticActionKey, MahjongUiAction, MahjongUiState};
use crate::app::presentation::{ACCENT, BORDER, MUTED, PANEL, TEXT, add_text, spawn_node};
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
    developer_hand_input_visible: bool,
    assets: &UiAssets,
) {
    let expanded = ui.auto_drawer_open;
    let drawer = spawn_node(
        commands,
        table,
        Node {
            position_type: PositionType::Absolute,
            left: px(8),
            bottom: px(if developer_hand_input_visible {
                128
            } else {
                82
            }),
            width: px(if expanded { 142 } else { 38 }),
            padding: UiRect::axes(px(if expanded { 9 } else { 3 }), px(6)),
            flex_direction: FlexDirection::Column,
            row_gap: px(3),
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(8)),
            ..default()
        },
        Some(PANEL.with_alpha(0.94)),
    );
    commands
        .entity(drawer)
        .insert((BorderColor::all(BORDER), GlobalZIndex(1200)));
    for (short, label, enabled, action) in [
        (
            "和",
            "自动和牌",
            ui.auto_win,
            MahjongUiAction::ToggleAutoWin,
        ),
        (
            "鸣",
            "不再鸣牌",
            ui.no_claim,
            MahjongUiAction::ToggleNoClaim,
        ),
        (
            "打",
            "自动摸打",
            ui.auto_draw_discard,
            MahjongUiAction::ToggleAutoDrawDiscard,
        ),
    ]
    .into_iter()
    .skip(usize::from(false_win_allowed))
    {
        let row = commands
            .spawn((
                Button,
                UiAction::Mahjong(action),
                Node {
                    width: percent(100),
                    height: px(31),
                    align_items: AlignItems::Center,
                    justify_content: if expanded {
                        JustifyContent::SpaceBetween
                    } else {
                        JustifyContent::Center
                    },
                    ..default()
                },
                BackgroundColor(Color::NONE),
            ))
            .id();
        commands.entity(drawer).add_child(row);
        add_text(
            commands,
            row,
            if expanded { label } else { short },
            if expanded { 14.0 } else { 17.0 },
            if enabled { ACCENT } else { MUTED },
            assets,
        );
        if expanded {
            let circle = spawn_node(
                commands,
                row,
                Node {
                    width: px(16),
                    height: px(16),
                    border: UiRect::all(px(1.5)),
                    border_radius: BorderRadius::all(px(8)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                None,
            );
            commands
                .entity(circle)
                .insert(BorderColor::all(if enabled { ACCENT } else { MUTED }));
            if enabled {
                spawn_node(
                    commands,
                    circle,
                    Node {
                        width: px(8),
                        height: px(8),
                        border_radius: BorderRadius::all(px(4)),
                        ..default()
                    },
                    Some(ACCENT),
                );
            }
        }
    }
    let arrow = commands
        .spawn((
            Button,
            UiAction::Mahjong(MahjongUiAction::ToggleAutoDrawer),
            Node {
                width: percent(100),
                height: px(24),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::NONE),
        ))
        .id();
    commands.entity(drawer).add_child(arrow);
    add_text(
        commands,
        arrow,
        if expanded { "<" } else { ">" },
        17.0,
        TEXT,
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
