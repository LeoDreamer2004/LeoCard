use super::assets::DrawGuessAssets;
use super::feedback::DrawingFeedback;
use crate::app::presentation::{AnswerRevealMotion, AnswerRevealSpec, add_answer_reveal};
use crate::app::runtime::UiAssets;
use bevy::prelude::*;
use leocard_draw_guess::{DrawGuessPhase, DrawGuessRuleSet};
use leocard_protocol::DrawGuessSnapshot;
use leocard_ui::{TransitionVisuals, UiComposer, fade_panel};

#[derive(Component)]
pub(super) struct DrawingAnswerReveal;

pub(super) fn render(
    commands: &mut UiComposer,
    parent: Entity,
    game: &DrawGuessSnapshot,
    art: &DrawGuessAssets,
    assets: &UiAssets,
) {
    if game.state.phase != DrawGuessPhase::Reveal {
        return;
    }
    let Some(answer) = &game.state.word else {
        return;
    };
    let guessed = game
        .state
        .players
        .iter()
        .filter(|player| player.guessed)
        .count();
    let total = game.state.players.len().saturating_sub(1);
    let caption = format!("答案揭晓  {guessed}/{total} 人猜中");
    let banner = add_answer_reveal(
        commands,
        parent,
        AnswerRevealSpec {
            answer,
            caption: &caption,
            icon: None,
        },
        art.skin.popup_frame(),
        assets,
    );
    commands.entity(banner).insert(DrawingAnswerReveal);
}

pub(super) fn animate(
    feedback: Res<DrawingFeedback>,
    mut panels: Query<(Entity, &mut UiTransform), With<DrawingAnswerReveal>>,
    children: Query<&Children>,
    mut visuals: TransitionVisuals,
    mut commands: UiComposer,
) {
    let motion = AnswerRevealMotion::new(
        feedback.answer_elapsed(),
        f32::from(DrawGuessRuleSet::REVEAL_SECONDS) - 0.5,
    );
    for (entity, mut transform) in &mut panels {
        *transform = motion.transform;
        fade_panel(
            entity,
            motion.opacity,
            &children,
            &mut visuals,
            &mut commands,
        );
    }
}
