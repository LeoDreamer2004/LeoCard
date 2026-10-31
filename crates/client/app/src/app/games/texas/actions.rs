//! 德州扑克按钮动作到本地状态或网络命令的转换。

use super::{TexasHandGuideState, TexasUiState};
use crate::app::runtime::{ClientResource, PageErrorState};
use crate::app::shell::{
    DomainUiAction, PressedUiAction, UiAction, UiActionHandler, dispatch_domain_actions,
    send_game_command,
};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use leocard_client::{ItemId, PlayerEconomy};
use leocard_protocol::TexasCommand;
use leocard_texas_holdem::{TexasAction, TexasRuleSet};

#[derive(Clone)]
pub(crate) enum TexasUiAction {
    UpdateRules(TexasRuleSet),
    SetRaiseTo(u32),
    Act(TexasAction),
    ToggleHandGuide,
    CloseHandGuide,
    ToggleSpectatorDrawer,
    ToggleWinRates,
}

impl DomainUiAction for TexasUiAction {
    fn rebuilds_ui(&self) -> bool {
        !matches!(self, Self::ToggleHandGuide | Self::CloseHandGuide)
    }

    fn extract(action: &UiAction) -> Option<&Self> {
        let UiAction::Texas(action) = action else {
            return None;
        };
        Some(action)
    }
}

#[derive(SystemParam)]
pub(crate) struct TexasActionContext<'w> {
    client: Option<ResMut<'w, ClientResource>>,
    ui: ResMut<'w, TexasUiState>,
    guide: ResMut<'w, TexasHandGuideState>,
    page_error: ResMut<'w, PageErrorState>,
    economy: Res<'w, PlayerEconomy>,
}

pub(super) fn dispatch_texas_actions(
    mut actions: MessageReader<PressedUiAction>,
    mut context: TexasActionContext,
) {
    dispatch_domain_actions::<TexasUiAction, _>(&mut actions, &mut context);
}

impl UiActionHandler<TexasActionContext<'_>> for TexasUiAction {
    fn handle(&self, context: &mut TexasActionContext<'_>) {
        match self {
            Self::ToggleSpectatorDrawer => {
                context.ui.spectator.drawer_open = !context.ui.spectator.drawer_open
            }
            Self::ToggleWinRates => {
                if !context.economy.active(ItemId::ObservationLens) {
                    return;
                }
                let preferences = &mut context.ui.spectator.preferences;
                preferences.show_win_rates = !preferences.show_win_rates;
                if let Err(error) = preferences.save() {
                    preferences.show_win_rates = !preferences.show_win_rates;
                    context.page_error.error = Some(error);
                }
            }
            Self::ToggleHandGuide => context.guide.toggle(),
            Self::CloseHandGuide => context.guide.close(),
            TexasUiAction::UpdateRules(rules) => {
                send_game_command(
                    &mut context.client,
                    TexasCommand::UpdateRules { rules: *rules },
                );
            }
            TexasUiAction::SetRaiseTo(target) => context.ui.raise_to = *target,
            TexasUiAction::Act(action) => {
                send_game_command(&mut context.client, TexasCommand::Act { action: *action });
            }
        }
    }
}
