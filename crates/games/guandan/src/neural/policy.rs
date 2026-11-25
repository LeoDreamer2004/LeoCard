use super::GuandanModelObservation;
use crate::GuandanAction;
use leocard_ai::{DevicePreference, LoadedModel, ModelId, Network, NetworkBuilder, TensorMap};

pub struct GuandanActorBot;

impl GuandanActorBot {
    pub fn validate(tensors: &[TensorMap]) -> Result<(), String> {
        if tensors.len() != 1 {
            return Err("掼蛋模型权重组错误".into());
        }
        build(&tensors[0], DevicePreference::Cpu)?;
        Ok(())
    }

    pub fn choose(
        input: &GuandanModelObservation,
        model: &LoadedModel,
    ) -> Result<GuandanAction, String> {
        if model.id != ModelId::GuandanActor {
            return Err("掼蛋模型类型错误".into());
        }
        let network = model.network("actor", || build(model.tensors(0)?, model.device))?;
        let logits = network.run(&input.features, 1)?;
        input
            .actions
            .iter()
            .max_by(|(left, _), (right, _)| logits[*left].total_cmp(&logits[*right]))
            .map(|(_, action)| action.clone())
            .ok_or_else(|| "没有模型可用的合法动作".into())
    }
}

pub(super) fn build(tensors: &TensorMap, device: DevicePreference) -> Result<Network, String> {
    tensors
        .get("net.4.weight")
        .ok_or("掼蛋模型缺少策略头")?
        .expect_shape(&[375, 512])?;
    let mut builder = NetworkBuilder::new(vec![3049]);
    let x = builder.linear(0, tensors, "net.0", true)?;
    let x = builder.linear(x, tensors, "net.2", true)?;
    let x = builder.linear(x, tensors, "net.4", false)?;
    builder.finish(x, device)
}
