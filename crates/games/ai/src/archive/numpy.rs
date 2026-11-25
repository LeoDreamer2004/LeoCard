use super::TensorArchive;
use super::format::MAX_MODEL_BYTES;
use crate::{Tensor, TensorMap};
use half::f16;
use std::io::{Cursor, Read};
use std::str;
use zip::ZipArchive;

impl TensorArchive {
    pub fn from_npz(bytes: &[u8]) -> Result<TensorMap, String> {
        let mut archive = ZipArchive::new(Cursor::new(bytes)).map_err(|e| e.to_string())?;
        let mut tensors = TensorMap::new();
        let mut total = 0u64;
        for i in 0..archive.len() {
            let mut file = archive.by_index(i).map_err(|e| e.to_string())?;
            total = total.checked_add(file.size()).ok_or("模型尺寸溢出")?;
            if total > MAX_MODEL_BYTES as u64 {
                return Err("解压后的模型过大".into());
            }
            let name = file
                .name()
                .map_err(|e| e.to_string())?
                .strip_suffix(".npy")
                .ok_or("NPZ 含非张量条目")?
                .to_owned();
            let mut data = Vec::with_capacity(file.size() as usize);
            file.read_to_end(&mut data).map_err(|e| e.to_string())?;
            if tensors.insert(name, read_npy(&data)?).is_some() {
                return Err("NPZ 含重复张量".into());
            }
        }
        if tensors.is_empty() {
            return Err("NPZ 不含权重".into());
        }
        Ok(tensors)
    }
}

fn read_npy(bytes: &[u8]) -> Result<Tensor, String> {
    if bytes.len() < 12 || &bytes[..6] != b"\x93NUMPY" {
        return Err("无效的 NPY 张量".into());
    }
    let (start, length) = match bytes[6] {
        1 => (
            10,
            u16::from_le_bytes(bytes[8..10].try_into().unwrap()) as usize,
        ),
        2 | 3 => (
            12,
            u32::from_le_bytes(bytes[8..12].try_into().unwrap()) as usize,
        ),
        _ => return Err("不支持的 NPY 版本".into()),
    };
    let end = start + length;
    let header = bytes.get(start..end).ok_or("NPY 头部被截断")?;
    let header = str::from_utf8(header).map_err(|e| e.to_string())?;
    if !header.contains("'fortran_order': False") {
        return Err("模型必须使用连续行优先张量".into());
    }
    let shape = header
        .split("'shape':")
        .nth(1)
        .and_then(|s| s.split('(').nth(1))
        .and_then(|s| s.split(')').next())
        .ok_or("NPY 缺少尺寸")?;
    let shape = shape
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.parse::<usize>().map_err(|e| e.to_string()))
        .collect::<Result<Vec<_>, _>>()?;
    let dtype = header
        .split("'descr':")
        .nth(1)
        .and_then(|s| s.split('\'').nth(1))
        .ok_or("NPY 缺少数据类型")?;
    let data = &bytes[end..];
    let (width, values): (usize, Vec<f32>) = match dtype {
        "<f2" | "=f2" => (
            2,
            data.as_chunks::<2>()
                .0
                .iter()
                .map(|v| f16::from_le_bytes(*v).to_f32())
                .collect(),
        ),
        "<f4" | "=f4" => (
            4,
            data.as_chunks::<4>()
                .0
                .iter()
                .map(|v| f32::from_le_bytes(*v))
                .collect(),
        ),
        "<i8" | "=i8" => (
            8,
            data.as_chunks::<8>()
                .0
                .iter()
                .map(|v| i64::from_le_bytes(*v) as f32)
                .collect(),
        ),
        _ => return Err(format!("不支持的模型张量类型：{dtype}")),
    };
    if !data.len().is_multiple_of(width) {
        return Err("NPY 数据被截断".into());
    }
    Tensor::new(shape, values)
}
