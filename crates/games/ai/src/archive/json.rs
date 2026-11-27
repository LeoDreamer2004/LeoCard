use super::TensorArchive;
use crate::{Tensor, TensorMap};
use serde_json::Value;

impl TensorArchive {
    /// Numeric auxiliary tables use the same checked, compact cache as weights.
    pub fn from_json(bytes: &[u8]) -> Result<TensorMap, String> {
        let value: Value = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
        let mut tensors = TensorMap::new();
        flatten(&value, String::new(), &mut tensors)?;
        if tensors.is_empty() {
            return Err("模型 JSON 不含数值".into());
        }
        Ok(tensors)
    }
}

fn flatten(value: &Value, name: String, tensors: &mut TensorMap) -> Result<(), String> {
    match value {
        Value::Object(values) => {
            for (key, value) in values {
                flatten(value, format!("{name}/{key}"), tensors)?;
            }
        }
        Value::Array(values) => {
            for (index, value) in values.iter().enumerate() {
                flatten(value, format!("{name}/{index}"), tensors)?;
            }
        }
        Value::Number(value) => {
            let value = value.as_f64().ok_or("模型 JSON 数值错误")? as f32;
            tensors.insert(name, Tensor::new(vec![1], vec![value])?);
        }
        _ => {}
    }
    Ok(())
}
