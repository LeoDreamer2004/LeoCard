use super::TensorArchive;
use super::format::MAX_MODEL_BYTES;
use crate::{Tensor, TensorMap};
use half::{bf16, f16};
use nom::error::Error;
use repugnant_pickle::ops::PickleOp;
use repugnant_pickle::{Value, evaluate, parse_ops};
use std::io::{Cursor, Read};
use zip::ZipArchive;

impl TensorArchive {
    /// Reads tensor storage from torch.save ZIP files without a Python interpreter.
    /// Pickle globals are inspected as data and are never called.
    pub fn from_pytorch(bytes: &[u8]) -> Result<TensorMap, String> {
        let mut archive = ZipArchive::new(Cursor::new(bytes)).map_err(|e| e.to_string())?;
        let path = archive
            .file_names()
            .filter_map(Result::ok)
            .find(|p| p.ends_with("/data.pkl"))
            .ok_or("PyTorch 模型缺少 data.pkl")?
            .into_owned();
        let prefix = path.rsplit_once('/').ok_or("模型目录错误")?.0.to_owned();
        let pickle = read_entry(&mut archive, &path, 4 * 1024 * 1024)?;
        let (_, ops) =
            parse_ops::<Error<&[u8]>>(&pickle).map_err(|e| format!("模型元数据无效：{e}"))?;
        let (values, _) = evaluate(&ops, true).map_err(|e| e.to_string())?;
        let mut tensors = TensorMap::new();
        for value in &values {
            read_values(value, &prefix, &mut archive, &mut tensors, 0)?;
        }
        if tensors.is_empty() {
            return Err("模型不含支持的 PyTorch 权重".into());
        }
        Ok(tensors)
    }
}

fn read_values(
    value: &Value<'_>,
    prefix: &str,
    archive: &mut ZipArchive<Cursor<&[u8]>>,
    tensors: &mut TensorMap,
    depth: usize,
) -> Result<(), String> {
    if depth > 64 {
        return Err("模型元数据嵌套过深".into());
    }
    match value {
        Value::Seq(_, items) => {
            if let [Value::String(name), tensor] = items.as_slice()
                && let Some(tensor) = read_tensor(tensor, prefix, archive)?
            {
                tensors.insert((*name).to_owned(), tensor);
                return Ok(());
            }
            for item in items {
                read_values(item, prefix, archive, tensors, depth + 1)?;
            }
        }
        Value::Global(_, items) | Value::App(_, items) | Value::Object(_, items) => {
            for item in items {
                read_values(item, prefix, archive, tensors, depth + 1)?;
            }
        }
        Value::Build(object, state) => {
            read_values(object, prefix, archive, tensors, depth + 1)?;
            read_values(state, prefix, archive, tensors, depth + 1)?;
        }
        _ => {}
    }
    Ok(())
}

fn read_tensor(
    value: &Value<'_>,
    prefix: &str,
    archive: &mut ZipArchive<Cursor<&[u8]>>,
) -> Result<Option<Tensor>, String> {
    let Value::Global(function, arguments) = value else {
        return Ok(None);
    };
    let Value::Raw(function) = function.as_ref() else {
        return Ok(None);
    };
    if !matches!(
        function.as_ref(),
        PickleOp::GLOBAL("torch._utils", "_rebuild_tensor_v2" | "_rebuild_tensor")
    ) {
        return Ok(None);
    }
    let [Value::Seq(_, fields)] = arguments.as_slice() else {
        return Err("模型张量参数错误".into());
    };
    let [
        Value::PersId(storage),
        Value::Int(offset),
        Value::Seq(_, shape),
        Value::Seq(_, stride),
        ..,
    ] = fields.as_slice()
    else {
        return Err("模型张量布局错误".into());
    };
    let Value::Seq(_, storage) = storage.as_ref() else {
        return Err("模型存储标识错误".into());
    };
    let [
        Value::String("storage"),
        Value::Raw(dtype),
        Value::String(key),
        ..,
    ] = storage.as_slice()
    else {
        return Err("模型存储字段错误".into());
    };
    let PickleOp::GLOBAL("torch", dtype) = dtype.as_ref() else {
        return Err("不支持的张量类型".into());
    };
    let dimensions = |values: &[Value<'_>]| {
        values
            .iter()
            .map(|v| match v {
                Value::Int(n) => usize::try_from(*n).map_err(|_| "模型尺寸为负数".to_owned()),
                _ => Err("模型尺寸不是整数".into()),
            })
            .collect::<Result<Vec<_>, String>>()
    };
    let shape = dimensions(shape)?;
    let stride = dimensions(stride)?;
    let mut elements = 1usize;
    for (&dimension, &step) in shape.iter().zip(&stride).rev() {
        if step != elements && dimension > 1 {
            return Err("模型使用不支持的非连续张量".into());
        }
        elements = elements.checked_mul(dimension).ok_or("模型尺寸溢出")?;
    }
    let width = match *dtype {
        "HalfStorage" | "BFloat16Storage" => 2,
        "FloatStorage" => 4,
        "DoubleStorage" | "LongStorage" => 8,
        _ => return Err(format!("不支持的模型存储类型：{dtype}")),
    };
    let data = read_entry(archive, &format!("{prefix}/data/{key}"), MAX_MODEL_BYTES)?;
    let start = usize::try_from(*offset)
        .ok()
        .and_then(|n| n.checked_mul(width))
        .ok_or("张量偏移错误")?;
    let end = elements
        .checked_mul(width)
        .and_then(|n| start.checked_add(n))
        .ok_or("张量尺寸溢出")?;
    let data = data.get(start..end).ok_or("模型存储被截断")?;
    let values = match *dtype {
        "HalfStorage" => data
            .as_chunks::<2>()
            .0
            .iter()
            .map(|v| f16::from_le_bytes(*v).to_f32())
            .collect(),
        "BFloat16Storage" => data
            .as_chunks::<2>()
            .0
            .iter()
            .map(|v| bf16::from_le_bytes(*v).to_f32())
            .collect(),
        "FloatStorage" => data
            .as_chunks::<4>()
            .0
            .iter()
            .map(|v| f32::from_le_bytes(*v))
            .collect(),
        "DoubleStorage" => data
            .as_chunks::<8>()
            .0
            .iter()
            .map(|v| f64::from_le_bytes(*v) as f32)
            .collect(),
        _ => data
            .as_chunks::<8>()
            .0
            .iter()
            .map(|v| i64::from_le_bytes(*v) as f32)
            .collect(),
    };
    Tensor::new(shape, values).map(Some)
}

fn read_entry(
    archive: &mut ZipArchive<Cursor<&[u8]>>,
    path: &str,
    limit: usize,
) -> Result<Vec<u8>, String> {
    let mut file = archive.by_name(path).map_err(|e| e.to_string())?;
    if file.size() > limit as u64 {
        return Err("模型条目过大".into());
    }
    let mut bytes = Vec::with_capacity(file.size() as usize);
    file.read_to_end(&mut bytes).map_err(|e| e.to_string())?;
    Ok(bytes)
}
