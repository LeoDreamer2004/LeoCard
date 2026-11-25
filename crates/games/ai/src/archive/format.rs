use crate::{Tensor, TensorMap};
use half::f16;
use serde::{Deserialize, Serialize};

const MAGIC: &[u8; 8] = b"LCAI001\0";
pub(super) const MAX_MODEL_BYTES: usize = 512 * 1024 * 1024;

#[derive(Deserialize, Serialize)]
struct Entry {
    name: String,
    shape: Vec<usize>,
    elements: usize,
}

/// Validated, versioned tensor data. Loading never executes Python code.
pub struct TensorArchive;

impl TensorArchive {
    pub fn encode(tensors: &TensorMap) -> Result<Vec<u8>, String> {
        let entries: Vec<_> = tensors
            .iter()
            .map(|(name, tensor)| Entry {
                name: name.clone(),
                shape: tensor.shape.clone(),
                elements: tensor.values.len(),
            })
            .collect();
        let header = serde_json::to_vec(&entries).map_err(|e| e.to_string())?;
        let mut out = MAGIC.to_vec();
        out.extend_from_slice(&(header.len() as u32).to_le_bytes());
        out.extend_from_slice(&header);
        for tensor in tensors.values() {
            for value in tensor.values.iter() {
                let value = f16::from_f32(*value);
                if !value.is_finite() {
                    return Err("模型权重超出 f16 范围".into());
                }
                out.extend_from_slice(&value.to_le_bytes());
            }
        }
        if out.len() > MAX_MODEL_BYTES {
            return Err("模型超过最大支持大小".into());
        }
        Ok(out)
    }

    pub fn decode(bytes: &[u8]) -> Result<TensorMap, String> {
        if bytes.len() < 12 || bytes.len() > MAX_MODEL_BYTES || &bytes[..8] != MAGIC {
            return Err("模型缓存格式或版本错误，请重新下载".into());
        }
        let header_len = u32::from_le_bytes(bytes[8..12].try_into().unwrap()) as usize;
        let end = 12usize
            .checked_add(header_len)
            .filter(|n| *n <= bytes.len())
            .ok_or("模型头部被截断")?;
        let entries: Vec<Entry> =
            serde_json::from_slice(&bytes[12..end]).map_err(|e| e.to_string())?;
        let mut offset = end;
        let mut tensors = TensorMap::new();
        for entry in entries {
            let size = entry.elements.checked_mul(2).ok_or("模型尺寸溢出")?;
            let end = offset
                .checked_add(size)
                .filter(|n| *n <= bytes.len())
                .ok_or("模型权重被截断")?;
            let values = bytes[offset..end]
                .as_chunks::<2>()
                .0
                .iter()
                .map(|v| f16::from_le_bytes(*v).to_f32())
                .collect();
            let tensor = Tensor::new(entry.shape, values)?;
            if tensors.insert(entry.name, tensor).is_some() {
                return Err("模型含重复张量".into());
            }
            offset = end;
        }
        if offset != bytes.len() || tensors.is_empty() {
            return Err("模型缓存含多余数据或不含权重".into());
        }
        Ok(tensors)
    }
}
