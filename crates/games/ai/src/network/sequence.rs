use super::NetworkBuilder;
use super::operation::{CausalAttention, Embedding, Gelu, LayerNorm};
use crate::TensorMap;

impl NetworkBuilder {
    pub fn layer_norm(
        &mut self,
        input: usize,
        tensors: &TensorMap,
        name: &str,
        relu: bool,
    ) -> Result<usize, String> {
        let shape = self.shape(input)?.to_vec();
        let width = shape.iter().product::<usize>();
        let weight = tensors
            .get(&format!("{name}.weight"))
            .ok_or_else(|| format!("模型缺少 {name}.weight"))?
            .clone();
        let bias = tensors
            .get(&format!("{name}.bias"))
            .ok_or_else(|| format!("模型缺少 {name}.bias"))?
            .clone();
        weight.expect_shape(&[width])?;
        bias.expect_shape(&[width])?;
        self.push(input, shape, LayerNorm { weight, bias, relu })
    }

    /// One unpadded sequence. Rows are tokens, with learned query-relative positions.
    pub fn token_embedding(
        &mut self,
        tensors: &TensorMap,
        tokens: &str,
        positions: &str,
    ) -> Result<usize, String> {
        if self.input_shape != [1] || !self.steps.is_empty() {
            return Err("词元嵌入必须是序列网络的第一层".into());
        }
        let weight = tensors.get(tokens).ok_or("模型缺少词元嵌入")?.clone();
        let positions = tensors.get(positions).ok_or("模型缺少位置嵌入")?.clone();
        let [vocabulary, width] = weight.shape.as_slice() else {
            return Err("词元嵌入尺寸错误".into());
        };
        if *vocabulary == 0
            || *width == 0
            || positions.shape.len() != 2
            || positions.shape[1] != *width
        {
            return Err("位置嵌入尺寸错误".into());
        }
        self.push(0, vec![*width], Embedding { weight, positions })
    }

    pub fn gelu(&mut self, input: usize) -> Result<usize, String> {
        self.push(input, self.shape(input)?.to_vec(), Gelu)
    }

    pub fn causal_attention(&mut self, input: usize, heads: usize) -> Result<usize, String> {
        let shape = self.shape(input)?;
        if shape.len() != 1 || heads == 0 || !shape[0].is_multiple_of(3 * heads) {
            return Err("多头注意力尺寸错误".into());
        }
        self.push(input, vec![shape[0] / 3], CausalAttention { heads })
    }
}
