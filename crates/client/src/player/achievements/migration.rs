//! Migration from the previous v3 archive to the independent format version.

use super::storage::AchievementArchive;

const V3_HEADER: &[u8] = b"LEOCARD-ACHIEVEMENTS/0.5.0/v3\n";

pub(super) fn decode_v3(bytes: &[u8]) -> Result<AchievementArchive, String> {
    let payload = bytes
        .strip_prefix(V3_HEADER)
        .ok_or_else(|| "无法识别成就档案格式，原存档已保留".to_owned())?;
    // v3 and format 1 share the complete payload layout: profile identity,
    // criterion counters and scopes, unlock timestamps, and event receipts.
    AchievementArchive::decode_payload(payload)
}
