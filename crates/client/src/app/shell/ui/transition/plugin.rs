use super::lobby_game::animate_lobby_game_motion;
use super::page::{animate_page_motion, page_content_can_animate};
use super::{LobbyGameMotion, PageMotion};
use crate::app::presentation::{
    SummaryAnimationSet, animate_button_arrows, update_button_highlights,
};
use crate::app::runtime::ClientUpdateSet;
use crate::app::shell::update_achievement_category_hover;
use bevy::prelude::*;

pub(crate) struct PageTransitionPlugin;

impl Plugin for PageTransitionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PageMotion>()
            .init_resource::<LobbyGameMotion>()
            .configure_sets(Update, SummaryAnimationSet.run_if(page_content_can_animate))
            .add_systems(
                Update,
                (
                    animate_page_motion
                        .after(update_achievement_category_hover)
                        .after(update_button_highlights)
                        .after(animate_button_arrows),
                    animate_lobby_game_motion,
                )
                    .in_set(ClientUpdateSet::Animate),
            );
    }
}
