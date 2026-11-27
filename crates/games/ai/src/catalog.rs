use crate::{ModelId, TensorArchive, TensorMap};
use serde::Serialize;
use sha2::{Digest, Sha256};

#[derive(Clone, Copy, Debug, Serialize)]
pub enum SourceFormat {
    Pytorch,
    Numpy,
    Json,
}

#[derive(Serialize)]
pub struct ModelFile {
    pub name: &'static str,
    pub url: &'static str,
    pub bytes: u64,
    pub sha256: &'static str,
    pub format: SourceFormat,
    pub group: Option<&'static str>,
}

impl ModelFile {
    pub fn convert(&self, bytes: &[u8]) -> Result<(Vec<u8>, TensorMap), String> {
        self.verify(bytes)?;
        let tensors = match self.format {
            SourceFormat::Pytorch => TensorArchive::from_pytorch_group(bytes, self.group)?,
            SourceFormat::Numpy => TensorArchive::from_npz(bytes)?,
            SourceFormat::Json => TensorArchive::from_json(bytes)?,
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
        checkpoint!($name, $url, $bytes, $sha, $format, None)
    };
    ($name:literal, $url:expr, $bytes:literal, $sha:literal, $format:ident, $group:expr) => {
        ModelFile {
            name: $name,
            url: $url,
            bytes: $bytes,
            sha256: $sha,
            format: SourceFormat::$format,
            group: $group,
        }
    };
}

pub const fn model_spec(id: ModelId) -> &'static ModelSpec {
    match id {
        ModelId::BlackjackTransformer => &ModelSpec {
            id: ModelId::BlackjackTransformer,
            key: "blackjack-transformer",
            name: "Blackjack v6",
            version: "6cbe94b-f16-v1",
            license: "上游未声明许可证",
            source: "https://github.com/itsPat/blackjack-ai",
            files: &[checkpoint!(
                "transformer",
                "https://raw.githubusercontent.com/itsPat/blackjack-ai/6cbe94be3611b5fa85e59c1256b105f95ed09964/models/blackjack-v6-full/model.pt",
                3343042,
                "e49cd4db6a327842cfc0185021ee0b92a722878c3ede930c603fbdb8836dc091",
                Pytorch
            )],
        },
        ModelId::TexasPpo => &ModelSpec {
            id: ModelId::TexasPpo,
            key: "texas-ppo",
            name: "六人 PPO",
            version: "d58223f-f16-v1",
            license: "上游未声明许可证",
            source: "https://github.com/kailiu0712/Multi-Agent-Reinforcement-Learning-6-player-Texas-Poker",
            files: &[
                checkpoint!(
                    "policy",
                    "https://raw.githubusercontent.com/kailiu0712/Multi-Agent-Reinforcement-Learning-6-player-Texas-Poker/d58223f0e054dd60f04c98e1a64e71578cf94e2d/outputs/best_hero.pt",
                    234011,
                    "103d98411b324df13941613dd25454d232ed93211153dd61cac14a9a9b867565",
                    Pytorch,
                    Some("policy_state_dict")
                ),
                checkpoint!(
                    "strength",
                    "https://raw.githubusercontent.com/kailiu0712/Multi-Agent-Reinforcement-Learning-6-player-Texas-Poker/d58223f0e054dd60f04c98e1a64e71578cf94e2d/strength_chart.json",
                    1012423,
                    "b1e0cab1670cf56fab48af6736fc44d1dd031f0112fc6d07e6b865a088057a3d",
                    Json
                ),
            ],
        },
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
