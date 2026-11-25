use super::state::{Network, Operation, Step};
use crate::{DevicePreference, TensorMap};

pub struct NetworkBuilder {
    input_shape: Vec<usize>,
    steps: Vec<Step>,
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
        let weight = tensors
            .get(&format!("{name}.weight"))
            .ok_or_else(|| format!("模型缺少 {name}.weight"))?
            .clone();
        let bias = tensors
            .get(&format!("{name}.bias"))
            .ok_or_else(|| format!("模型缺少 {name}.bias"))?
            .clone();
        let inputs = self.shape(input)?.iter().product::<usize>();
        let [outputs, width] = weight.shape.as_slice() else {
            return Err("全连接权重尺寸错误".into());
        };
        if inputs != *width {
            return Err(format!("全连接 {name} 的输入尺寸错误"));
        }
        bias.expect_shape(&[*outputs])?;
        self.push(
            input,
            vec![*outputs],
            Operation::Linear { weight, bias, relu },
        )
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
            Operation::Conv3x3 { weight, bias, relu },
        )
    }

    pub fn add_relu(&mut self, input: usize, other: usize) -> Result<usize, String> {
        let shape = self.shape(input)?.to_vec();
        if shape != self.shape(other)? {
            return Err("残差分支尺寸不一致".into());
        }
        self.push(input, shape, Operation::AddRelu { other })
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

    fn shape(&self, slot: usize) -> Result<&[usize], String> {
        if slot == 0 {
            Ok(&self.input_shape)
        } else {
            self.steps
                .get(slot - 1)
                .map(|s| s.shape.as_slice())
                .ok_or_else(|| "模型引用了不存在的张量".into())
        }
    }

    fn push(
        &mut self,
        input: usize,
        shape: Vec<usize>,
        operation: Operation,
    ) -> Result<usize, String> {
        self.steps.push(Step {
            input,
            shape,
            operation,
        });
        Ok(self.steps.len())
    }
}
