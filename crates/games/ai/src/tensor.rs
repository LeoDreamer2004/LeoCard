use std::collections::BTreeMap;
use std::sync::Arc;

pub type TensorMap = BTreeMap<String, Tensor>;

#[derive(Clone, Debug)]
pub struct Tensor {
    pub shape: Vec<usize>,
    pub values: Arc<[f32]>,
}

impl Tensor {
    pub fn new(shape: Vec<usize>, values: Vec<f32>) -> Result<Self, String> {
        let elements = shape.iter().try_fold(1usize, |n, d| n.checked_mul(*d));
        if elements != Some(values.len()) || values.iter().any(|v| !v.is_finite()) {
            return Err("模型张量尺寸错误或包含非有限数值".into());
        }
        Ok(Self {
            shape,
            values: values.into(),
        })
    }

    pub fn expect_shape(&self, shape: &[usize]) -> Result<&[f32], String> {
        if self.shape != shape {
            return Err(format!(
                "模型张量尺寸错误：应为 {shape:?}，实际为 {:?}",
                self.shape
            ));
        }
        Ok(&self.values)
    }
}
