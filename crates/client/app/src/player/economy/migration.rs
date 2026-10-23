use super::storage::{EconomyArchive, FORMAT_VERSION};
use leocard_protocol::ProfileId;

pub(super) fn decode(
    version: u16,
    payload: &[u8],
    profile_id: ProfileId,
) -> Result<EconomyArchive, String> {
    match version {
        // 用户选择旧钱包重新开始，不保留旧余额和道具。
        1 | 2 => Ok(EconomyArchive::new(profile_id)),
        FORMAT_VERSION => {
            let archive: EconomyArchive =
                postcard::from_bytes(payload).map_err(|error| format!("无法解析钱包：{error}"))?;
            if archive.profile_id != profile_id {
                return Err("钱包与当前玩家身份不匹配，原存档已保留".to_owned());
            }
            Ok(archive)
        }
        _ => Err("不支持此钱包版本，原存档已保留".to_owned()),
    }
}
