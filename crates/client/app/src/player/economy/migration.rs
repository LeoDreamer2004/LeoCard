use super::PropId;
use super::storage::{EconomyArchive, EconomyLedger, FORMAT_VERSION};
use leocard_protocol::{AvatarFrameId, ProfileId};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Deserialize)]
struct V4EconomyArchive {
    ledger: EconomyLedger,
    owned_frames: BTreeSet<AvatarFrameId>,
    equipped_frame: AvatarFrameId,
}

/// Released v5 and v6 share this payload; v6 already uses the shared card counter.
#[derive(Deserialize)]
struct V6EconomyArchive {
    ledger: EconomyLedger,
    owned_frames: BTreeSet<AvatarFrameId>,
    equipped_frame: AvatarFrameId,
    exclusive_frame: Option<AvatarFrameId>,
}

pub(super) fn decode(
    version: u16,
    payload: &[u8],
    profile_id: ProfileId,
    exclusive: Option<AvatarFrameId>,
    now: u64,
) -> Result<EconomyArchive, String> {
    let mut archive: EconomyArchive = match version {
        1 | 2 => Ok(EconomyArchive::new(profile_id)),
        3 => postcard::from_bytes(payload).map(EconomyArchive::with_ledger),
        4 => postcard::from_bytes::<V4EconomyArchive>(payload).map(|previous| EconomyArchive {
            ledger: previous.ledger,
            owned_frames: previous.owned_frames,
            equipped_frame: previous.equipped_frame,
            exclusive_frame: exclusive,
            consumables: BTreeMap::new(),
        }),
        5 | 6 => postcard::from_bytes::<V6EconomyArchive>(payload).map(|previous| EconomyArchive {
            ledger: previous.ledger,
            owned_frames: previous.owned_frames,
            equipped_frame: previous.equipped_frame,
            exclusive_frame: previous.exclusive_frame,
            consumables: BTreeMap::new(),
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
    if (3..6).contains(&version) {
        merge_card_counter_time(&mut archive.ledger, now);
    }
    Ok(archive)
}

fn merge_card_counter_time(ledger: &mut EconomyLedger, now: u64) {
    let remaining = [
        PropId::ShengjiCardCounter,
        PropId::QiGui523CardCounter,
        PropId::DoudizhuCardCounter,
    ]
    .into_iter()
    .filter_map(|id| ledger.active_props.remove(&id))
    .fold(0_u64, |total, expires| {
        total.saturating_add(expires.saturating_sub(now))
    });
    if remaining > 0 {
        ledger
            .active_props
            .insert(PropId::CardCounter, now.saturating_add(remaining));
    }
}
