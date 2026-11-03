use super::DouzeroObservation;
use super::model::Model;
use super::observation::encode_cards;
use crate::DoudizhuAction;
use leocard_game_common::SearchBudget;

pub struct DouzeroBot;

impl DouzeroBot {
    pub fn choose_with_progress(
        input: &DouzeroObservation,
        budget: &SearchBudget,
        report: &mut dyn FnMut(DoudizhuAction),
    ) -> DoudizhuAction {
        let model = Model::for_role(input.role);
        let hidden = model.encode_history(&input.history);
        let mut best = input
            .actions
            .first()
            .expect("a running hand has a legal move")
            .clone();
        let mut best_score = f32::NEG_INFINITY;
        for actions in input.actions.chunks(32) {
            if budget.exhausted() {
                break;
            }
            let features: Vec<_> = actions
                .iter()
                .map(|action| {
                    let mut row = input.features.clone();
                    let cards = match action {
                        DoudizhuAction::Play(cards) => cards.as_slice(),
                        _ => &[],
                    };
                    row.extend(encode_cards(cards));
                    row
                })
                .collect();
            for (action, score) in actions.iter().zip(model.scores(&hidden, &features)) {
                if score.is_finite() && score > best_score {
                    best_score = score;
                    best = action.clone();
                }
            }
            report(best.clone());
        }
        best
    }
}
