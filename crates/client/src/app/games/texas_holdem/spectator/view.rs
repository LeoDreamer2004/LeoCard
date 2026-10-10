use super::super::{TexasHoldemUiAction, TexasHoldemUiState};
use super::TexasSpectatorUiState;
use crate::app::presentation::{
    ACCENT, DrawerSwitch, SwitchDrawer, add_switch_drawer, add_text, spawn_node,
};
use crate::app::runtime::UiAssets;
use crate::app::shell::{SeatSide, UiAction};
use bevy::picking::Pickable;
use bevy::prelude::*;
use leocard_protocol::{PlayerId, TexasHoldemSnapshot};

#[derive(Component)]
pub(in super::super) struct TexasWinRateLabel(pub PlayerId);

pub(in super::super) fn render_texas_spectator_drawer(
    commands: &mut Commands,
    table: Entity,
    game: &TexasHoldemSnapshot,
    ui: &TexasHoldemUiState,
    assets: &UiAssets,
) {
    if !game.spectator_available || !ui.spectator.item_active {
        return;
    }
    let switches = [DrawerSwitch {
        short: "%",
        label: "胜率显示",
        enabled: ui.spectator.preferences.show_win_rates,
        action: UiAction::TexasHoldem(TexasHoldemUiAction::ToggleWinRates),
    }];
    add_switch_drawer(
        commands,
        table,
        SwitchDrawer {
            expanded: ui.spectator.drawer_open,
            toggle_action: UiAction::TexasHoldem(TexasHoldemUiAction::ToggleSpectatorDrawer),
            switches: &switches,
        },
        assets,
    );
}

pub(in super::super) fn add_texas_win_rate(
    commands: &mut Commands,
    avatar: Entity,
    player: PlayerId,
    side: SeatSide,
    game: &TexasHoldemSnapshot,
    state: &TexasSpectatorUiState,
    assets: &UiAssets,
) {
    if !state.item_active
        || !state.preferences.show_win_rates
        || game.spectator_equities.is_none()
        || !game
            .players
            .iter()
            .any(|seat| seat.id == player && !seat.folded)
    {
        return;
    }
    let holder = spawn_node(
        commands,
        avatar,
        Node {
            position_type: PositionType::Absolute,
            left: if matches!(side, SeatSide::Left) {
                Val::Auto
            } else {
                px(70)
            },
            right: if matches!(side, SeatSide::Left) {
                px(70)
            } else {
                Val::Auto
            },
            top: px(13),
            width: px(120),
            justify_content: if matches!(side, SeatSide::Left) {
                JustifyContent::FlexEnd
            } else {
                JustifyContent::FlexStart
            },
            ..default()
        },
        None,
    );
    commands
        .entity(holder)
        .insert((Pickable::IGNORE, ZIndex(30)));
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
