use bevy::prelude::*;
use leocard_protocol::{
    TexasPhaseView, TexasPlayerView, TexasPotAwardView, TexasRevealedHand, TexasSnapshot,
};
use leocard_texas_holdem::{TexasCard, TexasEvaluatedHand};

#[derive(Component)]
pub(in crate::app::games::texas) struct TexasShowdownRevealRoot;

#[derive(Component)]
pub(in crate::app::games::texas) struct TexasShowdownBackdrop;

#[derive(Component)]
pub(in crate::app::games::texas) struct TexasShowdownBestCard {
    pub source: Vec2,
    pub target: Vec2,
    pub delay: f32,
    pub start_scale: Vec2,
}

#[derive(Component)]
pub(in crate::app::games::texas) struct TexasShowdownTitle;

#[derive(Component)]
pub(in crate::app::games::texas) struct TexasShowdownTitleText;

#[derive(Component)]
pub(in crate::app::games::texas) struct TexasShowdownUnderline;

pub(in crate::app::games::texas) struct TexasShowdownHand<'a> {
    pub award: &'a TexasPotAwardView,
    pub winner: &'a TexasPlayerView,
    pub revealed: &'a TexasRevealedHand,
    pub best: TexasEvaluatedHand,
    pub board: &'a [TexasCard],
}

impl<'a> TexasShowdownHand<'a> {
    pub(super) fn for_run(game: &'a TexasSnapshot, run: u8) -> Option<Self> {
        let TexasPhaseView::HandComplete {
            showdown: true,
            awards,
            ..
        } = &game.phase
        else {
            return None;
        };
        let award = awards.iter().find(|award| award.run == run)?;
        let winner_id = *award.winners.first()?;
        let winner = game
            .players
            .iter()
            .find(|player| player.player.id == winner_id)?;
        let revealed = game
            .revealed_hands
            .iter()
            .find(|hand| hand.player == winner_id)?;
        let (board, best) = match run {
            1 => (&game.community, revealed.best?),
            2 => (&game.second_community, revealed.second_best?),
            _ => return None,
        };
        Some(Self {
            award,
            winner,
            revealed,
            best,
            board,
        })
    }
}
