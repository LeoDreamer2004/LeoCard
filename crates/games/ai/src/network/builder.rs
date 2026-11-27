use super::operation::{Add, Conv3x3, Linear, Operation};
use super::state::{Network, Step};
use crate::{DevicePreference, Tensor, TensorMap};
use std::sync::Arc;

pub struct NetworkBuilder {
    pub(super) input_shape: Vec<usize>,
    pub(super) steps: Vec<Step>,
}

impl NetworkBuilder {
    pub fn new(input_shape: Vec<usize>) -> Self {
        Self {
            input_shape,
            steps: Vec::new(),
        }
    }

    pub fn linear(
        &mut self,
        input: usize,
        tensors: &TensorMap,
        name: &str,
        relu: bool,
    ) -> Result<usize, String> {
        self.linear_layer(input, tensors, name, relu, true)
    }

    pub fn linear_no_bias(
        &mut self,
        input: usize,
        tensors: &TensorMap,
        name: &str,
    ) -> Result<usize, String> {
        self.linear_layer(input, tensors, name, false, false)
    }

    fn linear_layer(
        &mut self,
        input: usize,
        tensors: &TensorMap,
        name: &str,
        relu: bool,
        with_bias: bool,
    ) -> Result<usize, String> {
        let weight = tensors
            .get(&format!("{name}.weight"))
            .ok_or_else(|| format!("模型缺少 {name}.weight"))?
            .clone();
        let inputs = self.shape(input)?.iter().product::<usize>();
        let [outputs, width] = weight.shape.as_slice() else {
            return Err("全连接权重尺寸错误".into());
        };
        if inputs != *width {
            return Err(format!("全连接 {name} 的输入尺寸错误"));
        }
        let bias = if with_bias {
            tensors
                .get(&format!("{name}.bias"))
                .ok_or_else(|| format!("模型缺少 {name}.bias"))?
                .clone()
        } else {
            Tensor::new(vec![*outputs], vec![0.0; *outputs])?
        };
        bias.expect_shape(&[*outputs])?;
        self.push(input, vec![*outputs], Linear { weight, bias, relu })
    }

    pub fn conv3x3(
        &mut self,
        input: usize,
        tensors: &TensorMap,
        name: &str,
        relu: bool,
    ) -> Result<usize, String> {
        let weight = tensors
            .get(&format!("{name}.weight"))
            .ok_or_else(|| format!("模型缺少 {name}.weight"))?
            .clone();
        let bias = tensors
            .get(&format!("{name}.bias"))
            .ok_or_else(|| format!("模型缺少 {name}.bias"))?
            .clone();
        let [channels, height, width] = self.shape(input)? else {
            return Err("卷积输入必须为 CHW 张量".into());
        };
        let [outputs, inputs, 3, 3] = weight.shape.as_slice() else {
            return Err("卷积核必须为 3×3".into());
        };
        if channels != inputs {
            return Err(format!("卷积 {name} 的输入通道错误"));
        }
        bias.expect_shape(&[*outputs])?;
        self.push(
            input,
            vec![*outputs, *height, *width],
            Conv3x3 { weight, bias, relu },
        )
    }

    pub fn add_relu(&mut self, input: usize, other: usize) -> Result<usize, String> {
        self.add(input, other, true)
    }

    pub fn add(&mut self, input: usize, other: usize, relu: bool) -> Result<usize, String> {
        let shape = self.shape(input)?.to_vec();
        if shape != self.shape(other)? {
            return Err("残差分支尺寸不一致".into());
        }
        self.push(input, shape, Add { other, relu })
    }

    pub fn finish(self, output: usize, preference: DevicePreference) -> Result<Network, String> {
        self.shape(output)?;
        if self.steps.is_empty() {
            return Err("模型网络为空".into());
        }
        Ok(Network::new(
            self.input_shape,
            self.steps,
            output,
            preference,
        ))
    }

    pub(super) fn shape(&self, slot: usize) -> Result<&[usize], String> {
        if slot == 0 {
            Ok(&self.input_shape)
        } else {
            self.steps
                .get(slot - 1)
                .map(|s| s.shape.as_slice())
                .ok_or_else(|| "模型引用了不存在的张量".into())
        }
    }

    pub(super) fn push(
        &mut self,
        input: usize,
        shape: Vec<usize>,
        operation: impl Operation + 'static,
    ) -> Result<usize, String> {
        self.steps.push(Step {
            input,
            shape,
            operation: Arc::new(operation),
        });
        Ok(self.steps.len())
    }
}
