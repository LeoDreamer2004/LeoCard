use super::operation::CpuInput;
use super::state::Network;

pub(super) fn run(network: &Network, input: &[f32], batch: usize) -> Result<Vec<f32>, String> {
    let mut slots = vec![input.to_vec()];
    for step in &network.steps {
        let input = CpuInput {
            values: &slots[step.input],
            other: &slots[step.operation.other_input().unwrap_or(step.input)],
            shape: network.shape(step.input),
            batch,
        };
        slots.push(step.operation.cpu(input)?);
    }
    let output = slots.swap_remove(network.output);
    if output.iter().any(|n| !n.is_finite()) {
        return Err("模型输出包含非有限数值".into());
    }
    Ok(output)
}
