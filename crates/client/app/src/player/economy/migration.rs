use super::storage::{EconomyArchive, EconomyLedger, FORMAT_VERSION};
use leocard_protocol::{AvatarFrameId, ProfileId};
use serde::Deserialize;
use std::collections::BTreeSet;

#[derive(Deserialize)]
struct V4EconomyArchive {
    ledger: EconomyLedger,
    owned_frames: BTreeSet<AvatarFrameId>,
    equipped_frame: AvatarFrameId,
}

pub(super) fn decode(
    version: u16,
    payload: &[u8],
    profile_id: ProfileId,
    exclusive: Option<AvatarFrameId>,
) -> Result<EconomyArchive, String> {
    let mut archive: EconomyArchive = match version {
        1 | 2 => Ok(EconomyArchive::new(profile_id)),
        3 => postcard::from_bytes(payload).map(EconomyArchive::with_ledger),
        4 => postcard::from_bytes::<V4EconomyArchive>(payload).map(|previous| EconomyArchive {
            ledger: previous.ledger,
            owned_frames: previous.owned_frames,
            equipped_frame: previous.equipped_frame,
            exclusive_frame: exclusive,
        }),
        FORMAT_VERSION => postcard::from_bytes(payload),
        _ => return Err("不支持此钱包版本，原存档已保留".to_owned()),
    }
    .map_err(|error| format!("无法解析钱包：{error}"))?;
    if archive.ledger.profile_id != profile_id {
        return Err("钱包与当前玩家身份不匹配，原存档已保留".to_owned());
    }
    if version < 4 {
        archive.update_exclusive_frame(exclusive);
    }
    Ok(archive)
}
