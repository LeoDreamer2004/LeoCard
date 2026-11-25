use crate::{ModelId, TensorArchive, TensorMap};
use serde::Serialize;
use sha2::{Digest, Sha256};

#[derive(Clone, Copy, Debug, Serialize)]
pub enum SourceFormat {
    Pytorch,
    Numpy,
}

#[derive(Serialize)]
pub struct ModelFile {
    pub name: &'static str,
    pub url: &'static str,
    pub bytes: u64,
    pub sha256: &'static str,
    pub format: SourceFormat,
}

impl ModelFile {
    pub fn convert(&self, bytes: &[u8]) -> Result<(Vec<u8>, TensorMap), String> {
        self.verify(bytes)?;
        let tensors = match self.format {
            SourceFormat::Pytorch => TensorArchive::from_pytorch(bytes)?,
            SourceFormat::Numpy => TensorArchive::from_npz(bytes)?,
        };
        let encoded = TensorArchive::encode(&tensors)?;
        let tensors = TensorArchive::decode(&encoded)?;
        Ok((encoded, tensors))
    }

    pub fn verify(&self, bytes: &[u8]) -> Result<(), String> {
        let digest = Sha256::digest(bytes)
            .iter()
            .map(|n| format!("{n:02x}"))
            .collect::<String>();
        if bytes.len() as u64 != self.bytes || digest != self.sha256 {
            return Err(format!("{} 下载校验失败，请重试", self.name));
        }
        Ok(())
    }
}

#[derive(Serialize)]
pub struct ModelSpec {
    pub id: ModelId,
    pub key: &'static str,
    pub name: &'static str,
    pub version: &'static str,
    pub license: &'static str,
    pub source: &'static str,
    pub files: &'static [ModelFile],
}

macro_rules! checkpoint {
    ($name:literal, $url:expr, $bytes:literal, $sha:literal, $format:ident) => {
        ModelFile {
            name: $name,
            url: $url,
            bytes: $bytes,
            sha256: $sha,
            format: SourceFormat::$format,
        }
    };
}

pub const fn model_spec(id: ModelId) -> &'static ModelSpec {
    match id {
        ModelId::Douzero => &ModelSpec {
            id: ModelId::Douzero,
            key: "douzero",
            name: "DouZero-WP",
            version: "57b3914-f16-v1",
            license: "Apache-2.0",
            source: "https://github.com/kwai/DouZero",
            files: &[
                checkpoint!(
                    "landlord",
                    "https://huggingface.co/palemoky/douzero-baselines/resolve/57b3914046c2a0877016b8b8830fd07cf5b0ba08/checkpoints/douzero_WP/landlord.ckpt",
                    5835434,
                    "132e26479fcb69f457ddbf1ad32a7bdc3aa73cde80a28cc91f29a62c6075955a",
                    Pytorch
                ),
                checkpoint!(
                    "landlord_down",
                    "https://huggingface.co/palemoky/douzero-baselines/resolve/57b3914046c2a0877016b8b8830fd07cf5b0ba08/checkpoints/douzero_WP/landlord_down.ckpt",
                    6062768,
                    "964720faf583f4905662c94aaf73b1a97352b76ae3686fcc949beabbacdd95d1",
                    Pytorch
                ),
                checkpoint!(
                    "landlord_up",
                    "https://huggingface.co/palemoky/douzero-baselines/resolve/57b3914046c2a0877016b8b8830fd07cf5b0ba08/checkpoints/douzero_WP/landlord_up.ckpt",
                    6062768,
                    "b849ea518e66f088d635d3a29956da880f7b5ca89fabc5f70fc356c106ca6a15",
                    Pytorch
                ),
            ],
        },
        ModelId::Kdens3 => &ModelSpec {
            id: ModelId::Kdens3,
            key: "kdens3",
            name: "kdens3",
            version: "6964d80-f16-v1",
            license: "MIT",
            source: "https://huggingface.co/Dannibal/ijcai-mahjong-ckpts-2026",
            files: &[
                checkpoint!(
                    "student0",
                    "https://huggingface.co/Dannibal/ijcai-mahjong-ckpts-2026/resolve/6964d80b9f8a6670dda44f14b664c61112f61009/champion/kdens_s0_fp16.npz",
                    26564407,
                    "b2d8bc289b0822e1d3e9d04eba3b6b3c435cd37a941875fc3af638cf84ef6e82",
                    Numpy
                ),
                checkpoint!(
                    "student1",
                    "https://huggingface.co/Dannibal/ijcai-mahjong-ckpts-2026/resolve/6964d80b9f8a6670dda44f14b664c61112f61009/champion/kdens_s1_fp16.npz",
                    26563317,
                    "132e110197f31c05b9d5b39120aa3fe73ac491a55b480314d98abc9fa68319e9",
                    Numpy
                ),
                checkpoint!(
                    "student2",
                    "https://huggingface.co/Dannibal/ijcai-mahjong-ckpts-2026/resolve/6964d80b9f8a6670dda44f14b664c61112f61009/champion/kdens_s2_fp16.npz",
                    26566330,
                    "73483924cedc86fe6bba2fe7d299127899299bddf429f4d0373364daf5565460",
                    Numpy
                ),
            ],
        },
        ModelId::GuandanActor => &ModelSpec {
            id: ModelId::GuandanActor,
            key: "guandan-actor",
            name: "Guandan Actor",
            version: "7d37216-master-v1",
            license: "MIT",
            source: "https://github.com/746505972/guandan",
            files: &[checkpoint!(
                "actor",
                "https://raw.githubusercontent.com/746505972/guandan/7d37216369d8b003e94a2779c82f497a621f6f7b/models/master.pth",
                8069242,
                "b37081152f9db7674779f1ecc9b7cbea7f2db6106c6e5bdf09d36d8c88a9aa37",
                Pytorch
            )],
        },
    }
}
