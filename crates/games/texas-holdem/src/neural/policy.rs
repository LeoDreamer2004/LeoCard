use super::TexasPpoObservation;
use crate::{TexasAction, TexasGameState, TexasRuleSet};
use leocard_ai::{DevicePreference, LoadedModel, ModelId, Network, NetworkBuilder, TensorMap};

pub struct TexasPpoBot;

impl TexasPpoBot {
    pub const fn supports_rules(rules: &TexasRuleSet) -> bool {
        !rules.short_deck
            && !rules.ignore_kickers
            && !rules.omaha
            && !rules.fixed_limit
            && rules.squid_markers == 0
            && rules.side_pots
            && !rules.run_it_twice
    }

    pub fn supports_game(game: &TexasGameState) -> bool {
        Self::supports_rules(game.rules())
            && game.players().len() == 6
            && game
                .players()
                .iter()
                .all(|player| player.stack() + player.committed_total() > 0)
    }

    pub fn validate(tensors: &[TensorMap]) -> Result<(), String> {
        let [weights, chart] = tensors else {
            return Err("德州模型权重组错误".into());
        };
        build(weights, DevicePreference::Cpu)?;
        for key in [
            "/preflop/6/values/AA/strength",
            "/postflop/6/river/values/river|c0/strength",
        ] {
            chart
                .get(key)
                .ok_or("德州模型缺少强度表")?
                .expect_shape(&[1])?;
        }
        Ok(())
    }

    pub fn choose(input: &TexasPpoObservation, model: &LoadedModel) -> Result<TexasAction, String> {
        if model.id != ModelId::TexasPpo {
            return Err("德州模型类型错误".into());
        }
        let network = model.network("policy", || build(model.tensors(0)?, model.device))?;
        let logits = network.run(&input.features, 1)?;
        input
            .actions
            .iter()
            .max_by(|(a, _), (b, _)| logits[*a].total_cmp(&logits[*b]))
            .map(|(_, action)| *action)
            .ok_or_else(|| "德州没有合法模型动作".into())
    }
}

pub(super) fn build(tensors: &TensorMap, device: DevicePreference) -> Result<Network, String> {
    tensors
        .get("head.weight")
        .ok_or("德州模型缺少策略头")?
        .expect_shape(&[5, 128])?;
    let mut builder = NetworkBuilder::new(vec![84]);
    let x = builder.linear(0, tensors, "backbone.0", false)?;
    let x = builder.layer_norm(x, tensors, "backbone.1", true)?;
    let x = builder.linear(x, tensors, "backbone.3", false)?;
    let x = builder.layer_norm(x, tensors, "backbone.4", true)?;
    let x = builder.linear(x, tensors, "head", false)?;
    builder.finish(x, device)
}
