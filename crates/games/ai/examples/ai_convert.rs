//! Convert a downloaded PyTorch/NPZ checkpoint without a Python environment.
//! cargo run -p leocard-ai --example ai_convert -- checkpoint.npz model.lcai
use leocard_ai::{ModelId, TensorArchive, model_spec};
use std::env;
use std::fs;

fn main() -> Result<(), String> {
    let arguments: Vec<_> = env::args().skip(1).collect();
    if arguments.as_slice() == ["--catalog"] {
        let catalog: Vec<_> = ModelId::ALL.into_iter().map(model_spec).collect();
        println!(
            "{}",
            serde_json::to_string_pretty(&catalog).map_err(|e| e.to_string())?
        );
        return Ok(());
    }
    let [input, output] = arguments.as_slice() else {
        return Err("usage: convert <checkpoint.npz|checkpoint.pt> <model.lcai>".into());
    };
    let bytes = fs::read(input).map_err(|e| e.to_string())?;
    let tensors = if input.ends_with(".npz") {
        TensorArchive::from_npz(&bytes)?
    } else {
        TensorArchive::from_pytorch(&bytes)?
    };
    let bytes = TensorArchive::encode(&tensors)?;
    let checked = TensorArchive::decode(&bytes)?;
    fs::write(output, &bytes).map_err(|e| e.to_string())?;
    println!("{} tensors, {} bytes", checked.len(), bytes.len());
    for (name, tensor) in checked.iter().take(8) {
        println!("{name}: {:?}", tensor.shape);
    }
    Ok(())
}
