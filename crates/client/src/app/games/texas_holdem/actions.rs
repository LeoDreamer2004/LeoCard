//! 德州扑克按钮动作到本地状态或网络命令的转换。

use super::{TexasHandGuideState, TexasHoldemUiState};
use crate::app::runtime::{ClientResource, PageErrorState};
use crate::app::shell::{
    DomainUiAction, PressedUiAction, UiAction, UiActionHandler, dispatch_domain_actions,
    send_game_command,
};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use leocard_protocol::TexasHoldemCommand;
use leocard_texas_holdem::{TexasHoldemAction, TexasHoldemRuleSet};

#[derive(Clone)]
pub(crate) enum TexasHoldemUiAction {
    UpdateRules(TexasHoldemRuleSet),
    SetRaiseTo(u32),
    Act(TexasHoldemAction),
    ToggleHandGuide,
    CloseHandGuide,
    ToggleSpectatorDrawer,
    ToggleWinRates,
}

impl DomainUiAction for TexasHoldemUiAction {
    fn rebuilds_ui(&self) -> bool {
        !matches!(self, Self::ToggleHandGuide | Self::CloseHandGuide)
    }

    fn extract(action: &UiAction) -> Option<&Self> {
        let UiAction::TexasHoldem(action) = action else {
            return None;
        };
        Some(action)
    }
}

#[derive(SystemParam)]
pub(crate) struct TexasHoldemActionContext<'w> {
    client: Option<ResMut<'w, ClientResource>>,
    ui: ResMut<'w, TexasHoldemUiState>,
    guide: ResMut<'w, TexasHandGuideState>,
    page_error: ResMut<'w, PageErrorState>,
}

pub(super) fn dispatch_texas_holdem_actions(
    mut actions: MessageReader<PressedUiAction>,
    mut context: TexasHoldemActionContext,
) {
    dispatch_domain_actions::<TexasHoldemUiAction, _>(&mut actions, &mut context);
}

impl UiActionHandler<TexasHoldemActionContext<'_>> for TexasHoldemUiAction {
    fn handle(&self, context: &mut TexasHoldemActionContext<'_>) {
        match self {
            Self::ToggleSpectatorDrawer => {
                context.ui.spectator.drawer_open = !context.ui.spectator.drawer_open
            }
            Self::ToggleWinRates => {
                let preferences = &mut context.ui.spectator.preferences;
                preferences.show_win_rates = !preferences.show_win_rates;
                if let Err(error) = preferences.save() {
                    preferences.show_win_rates = !preferences.show_win_rates;
                    context.page_error.error = Some(error);
                }
            }
            Self::ToggleHandGuide => context.guide.toggle(),
            Self::CloseHandGuide => context.guide.close(),
            TexasHoldemUiAction::UpdateRules(rules) => {
                send_game_command(
                    &mut context.client,
                    TexasHoldemCommand::UpdateRules { rules: *rules },
                );
            }
            TexasHoldemUiAction::SetRaiseTo(target) => context.ui.raise_to = *target,
            TexasHoldemUiAction::Act(action) => {
                send_game_command(
                    &mut context.client,
                    TexasHoldemCommand::Act { action: *action },
                );
            }
        }
    }
}
