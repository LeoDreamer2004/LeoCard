use super::super::MahjongBotAction;
use super::super::search::{Position, Search};
use super::MahjongModelObservation;
use super::features::action_id;
use super::model::build;
use leocard_ai::{DevicePreference, LoadedModel, ModelId, TensorMap};
use leocard_game_common::SearchBudget;
use std::collections::BTreeMap;

pub struct KdensBot;

impl KdensBot {
    pub fn validate(tensors: &[TensorMap]) -> Result<(), String> {
        if tensors.len() != 3 {
            return Err("kdens3 需要三个学生模型".into());
        }
        for weights in tensors {
            build(weights, DevicePreference::Cpu)?;
        }
        Ok(())
    }

    pub fn choose_with_progress(
        observation: &MahjongModelObservation,
        model: &LoadedModel,
        budget: &SearchBudget,
        report: &mut dyn FnMut(MahjongBotAction),
    ) -> Result<MahjongBotAction, String> {
        if model.id != ModelId::Kdens3 {
            return Err("麻将模型类型错误".into());
        }
        let input = &observation.position;
        if let Some(action) = input.immediate_action() {
            return Ok(action);
        }
        let actions: BTreeMap<_, _> = input
            .options
            .iter()
            .copied()
            .filter_map(|action| action_id(action, input).map(|id| (id, action)))
            .collect();
        if actions.is_empty() {
            return Err("当前动作不在麻将模型的动作空间中".into());
        }
        let features = &observation.features;
        let mut probabilities = [0.0f32; 235];
        let mut best = input.fallback();
        for student in 0..3 {
            if student > 0 && budget.exhausted() {
                break;
            }
            let network = model.network(&format!("student{student}"), || {
                build(model.tensors(student)?, model.device)
            })?;
            let logits = network.run(features, 1)?;
            let maximum = actions
                .keys()
                .map(|i| logits[*i])
                .fold(f32::NEG_INFINITY, f32::max);
            let total: f32 = actions.keys().map(|i| (logits[*i] - maximum).exp()).sum();
            for &i in actions.keys() {
                probabilities[i] += (logits[i] - maximum).exp() / total;
            }
            let mut score = f32::NEG_INFINITY;
            for (&i, &action) in &actions {
                if probabilities[i] > score {
                    score = probabilities[i];
                    best = action;
                }
            }
            report(best);
        }
        if input.may_declare_ready
            && let MahjongBotAction::Discard(tile) = best
        {
            let mut position = Position {
                counts: [0; 34],
                melds: input.melds.clone(),
                replacement: false,
                declared_ready: true,
            };
            for index in input.hand.iter().filter_map(|t| t.kind().index34()) {
                position.counts[index] += 1;
            }
            if let Some(index) = tile.kind().index34() {
                position.counts[index] -= 1;
                if Search::new(input, budget).has_live_win(&position, &input.unseen) {
                    best = MahjongBotAction::DeclareReady(tile);
                }
            }
        }
        Ok(best)
    }
}
