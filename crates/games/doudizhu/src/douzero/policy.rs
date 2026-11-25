use super::DouzeroObservation;
use super::model::{Model, build_head};
use super::observation::encode_cards;
use crate::DoudizhuAction;
use leocard_ai::{DevicePreference, LoadedModel, ModelId, TensorMap};
use leocard_game_common::SearchBudget;

pub struct DouzeroBot;

impl DouzeroBot {
    pub fn validate(tensors: &[TensorMap]) -> Result<(), String> {
        if tensors.len() != 3 {
            return Err("DouZero 需要三个角色模型".into());
        }
        for (role, weights) in tensors.iter().enumerate() {
            Model::read(weights)?;
            build_head(
                weights,
                if role == 0 { 373 } else { 484 },
                DevicePreference::Cpu,
            )?;
        }
        Ok(())
    }

    pub fn choose_with_progress(
        input: &DouzeroObservation,
        loaded: &LoadedModel,
        budget: &SearchBudget,
        report: &mut dyn FnMut(DoudizhuAction),
    ) -> Result<DoudizhuAction, String> {
        if loaded.id != ModelId::Douzero {
            return Err("斗地主模型类型错误".into());
        }
        let weights = loaded.tensors(input.role)?;
        let model = Model::read(weights)?;
        let network = loaded.network(&format!("role{}", input.role), || {
            build_head(
                weights,
                if input.role == 0 { 373 } else { 484 },
                loaded.device,
            )
        })?;
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
            let batch: Vec<_> = features
                .iter()
                .flat_map(|row| hidden.iter().chain(row).copied())
                .collect();
            for (action, score) in actions.iter().zip(network.run(&batch, features.len())?) {
                if score.is_finite() && score > best_score {
                    best_score = score;
                    best = action.clone();
                }
            }
            report(best.clone());
        }
        Ok(best)
    }
}
