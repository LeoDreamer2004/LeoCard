use super::super::{TexasUiAction, TexasUiState};
use super::TexasSpectatorUiState;
use crate::app::presentation::{ACCENT, SeatSide, add_switch_drawer, add_text};
use crate::app::runtime::UiAssets;
use crate::app::shell::UiAction;
use bevy::picking::Pickable;
use leocard_game_common::PlayerId;
use leocard_protocol::TexasSnapshot;
use leocard_ui::{DrawerSwitch, SwitchDrawer, UiComposer, spawn_node};

use bevy::prelude::*;

#[derive(Component)]
pub(in super::super) struct TexasWinRateLabel(pub PlayerId);

pub(in super::super) fn render_texas_spectator_drawer(
    commands: &mut UiComposer,
    viewport: Entity,
    game: &TexasSnapshot,
    ui: &TexasUiState,
    assets: &UiAssets,
) {
    if !game.spectator_available || !ui.spectator.item_active {
        return;
    }
    let switches = [DrawerSwitch {
        short: "%",
        label: "胜率显示",
        enabled: ui.spectator.preferences.show_win_rates,
        action: UiAction::Texas(TexasUiAction::ToggleWinRates),
    }];
    add_switch_drawer(
        commands,
        viewport,
        SwitchDrawer {
            expanded: ui.spectator.drawer_open,
            toggle_action: UiAction::Texas(TexasUiAction::ToggleSpectatorDrawer),
            switches: &switches,
        },
        assets,
    );
}

pub(in super::super) fn add_texas_win_rate(
    commands: &mut UiComposer,
    avatar: Entity,
    player: PlayerId,
    side: SeatSide,
    game: &TexasSnapshot,
    state: &TexasSpectatorUiState,
    assets: &UiAssets,
) {
    if !state.item_active
        || !state.preferences.show_win_rates
        || game.spectator_equities.is_none()
        || !game
            .players
            .iter()
            .any(|seat| seat.player.id == player && !seat.folded)
    {
        return;
    }
    // The mobile side seats sit close to the edge, so keep their equity labels toward the table.
    let left_of_avatar = if cfg!(target_os = "android") {
        matches!(side, SeatSide::Right)
    } else {
        matches!(side, SeatSide::Left)
    };
    let holder = spawn_node(
        commands,
        avatar,
        Node {
            position_type: PositionType::Absolute,
            left: if left_of_avatar { Val::Auto } else { px(70) },
            right: if left_of_avatar { px(70) } else { Val::Auto },
            top: px(13),
            width: px(120),
            justify_content: if left_of_avatar {
                JustifyContent::FlexEnd
            } else {
                JustifyContent::FlexStart
            },
            ..default()
        },
        None,
    );
    // Escape the portrait's local stack so table-wide squid and chip layers stay below the rate.
    commands
        .entity(holder)
        .insert((Pickable::IGNORE, GlobalZIndex(30)));
    let label = add_text(
        commands,
        holder,
        state
            .displayed(player)
            .map_or_else(|| "—%".to_owned(), |rate| format!("{rate:.0}%")),
        28.0,
        ACCENT,
        assets,
    );
    commands.entity(label).insert((
        TexasWinRateLabel(player),
        Pickable::IGNORE,
        TextShadow {
            offset: Vec2::new(1.0, 2.0),
            color: Color::BLACK.with_alpha(0.7),
        },
    ));
}
