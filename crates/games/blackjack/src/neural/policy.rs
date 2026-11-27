use crate::{BlackjackAction, BlackjackBot, BlackjackObservation};
use leocard_ai::{DevicePreference, LoadedModel, ModelId, Network, NetworkBuilder, TensorMap};

pub struct BlackjackTransformerBot;

impl BlackjackTransformerBot {
    pub fn validate(tensors: &[TensorMap]) -> Result<(), String> {
        let [weights] = tensors else {
            return Err("21 点模型权重组错误".into());
        };
        build(weights, DevicePreference::Cpu)?;
        Ok(())
    }

    pub fn choose(
        input: &BlackjackObservation,
        model: &LoadedModel,
    ) -> Result<BlackjackAction, String> {
        if model.id != ModelId::BlackjackTransformer {
            return Err("21 点模型类型错误".into());
        }
        // Every hand starts with a fresh shoe; retain the minimum stake policy.
        if input.available.minimum_bet > 0 {
            return Ok(BlackjackBot::choose(input));
        }
        let tokens = encode(input)?;
        let network = model.network("transformer", || build(model.tensors(0)?, model.device))?;
        let output = network.run(&tokens, tokens.len())?;
        let logits = &output[output.len() - 29..];
        let allowed = input.available;
        [
            (22, allowed.hit, BlackjackAction::Hit),
            (23, allowed.stand, BlackjackAction::Stand),
            (24, allowed.double_down, BlackjackAction::DoubleDown),
            (25, allowed.split, BlackjackAction::Split),
        ]
        .into_iter()
        .filter(|(_, legal, _)| *legal)
        .max_by(|(a, _, _), (b, _, _)| logits[*a].total_cmp(&logits[*b]))
        .map(|(_, _, action)| action)
        .ok_or_else(|| "21 点没有合法模型动作".into())
    }
}

pub(super) fn encode(input: &BlackjackObservation) -> Result<Vec<f32>, String> {
    let mut tokens = vec![11.0]; // HISTORY; no cards survive the next shuffle.
    tokens.extend(
        input
            .exposed
            .iter()
            .map(|card| f32::from(card.rank.points())),
    );
    tokens.extend([12.0, 13.0]); // CURRENT_HAND, PLAYER
    tokens.extend(input.cards.iter().map(|card| f32::from(card.rank.points())));
    tokens.push(14.0); // DEALER
    tokens.push(f32::from(
        input.dealer.ok_or("模型缺少庄家明牌")?.rank.points(),
    ));
    tokens.push(16.0); // PLAY_QUERY
    if tokens.len() > 256 {
        return Err("21 点公开牌超过模型上下文".into());
    }
    Ok(tokens)
}

pub(super) fn build(tensors: &TensorMap, device: DevicePreference) -> Result<Network, String> {
    tensors
        .get("token_embedding.weight")
        .ok_or("缺少词元嵌入")?
        .expect_shape(&[29, 128])?;
    tensors
        .get("position_embedding.weight")
        .ok_or("缺少位置嵌入")?
        .expect_shape(&[256, 128])?;
    tensors
        .get("language_model_head.weight")
        .ok_or("缺少决策头")?
        .expect_shape(&[29, 128])?;
    let mut builder = NetworkBuilder::new(vec![1]);
    let mut x = builder.token_embedding(
        tensors,
        "token_embedding.weight",
        "position_embedding.weight",
    )?;
    for index in 0..4 {
        let prefix = format!("blocks.{index}");
        let y = builder.layer_norm(x, tensors, &format!("{prefix}.attention_norm"), false)?;
        let y =
            builder.linear_no_bias(y, tensors, &format!("{prefix}.attention.query_key_value"))?;
        if tensors[&format!("{prefix}.attention.query_key_value.weight")].shape != [384, 128] {
            return Err("21 点注意力权重尺寸错误".into());
        }
        let y = builder.causal_attention(y, 4)?;
        let y = builder.linear_no_bias(y, tensors, &format!("{prefix}.attention.output"))?;
        x = builder.add(x, y, false)?;
        let y = builder.layer_norm(x, tensors, &format!("{prefix}.feed_forward_norm"), false)?;
        let y = builder.linear(y, tensors, &format!("{prefix}.feed_forward.input"), false)?;
        let y = builder.gelu(y)?;
        let y = builder.linear(y, tensors, &format!("{prefix}.feed_forward.output"), false)?;
        x = builder.add(x, y, false)?;
    }
    let x = builder.layer_norm(x, tensors, "final_norm", false)?;
    let x = builder.linear_no_bias(x, tensors, "language_model_head")?;
    builder.finish(x, device)
}
