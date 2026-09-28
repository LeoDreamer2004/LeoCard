use super::{ChipZoneLayout, PLAYER_CHIP_ZONE_HEIGHT, PLAYER_CHIP_ZONE_WIDTH};

pub(crate) fn texas_player_chip_zone(relative: u8) -> ChipZoneLayout {
    let (center_x, top, height) = match relative {
        0 => {
            // 自己的下注区位于底牌正上方，操作按钮紧接在其下方。
            (640.0, 405.0, PLAYER_CHIP_ZONE_HEIGHT)
        }
        1 => (282.0, 346.0, PLAYER_CHIP_ZONE_HEIGHT),
        2 => (282.0, 123.0, PLAYER_CHIP_ZONE_HEIGHT),
        3 => (640.0, 37.0, 116.0),
        4 => (998.0, 123.0, PLAYER_CHIP_ZONE_HEIGHT),
        5 => (998.0, 346.0, PLAYER_CHIP_ZONE_HEIGHT),
        _ => return texas_pot_chip_zone(),
    };
    ChipZoneLayout {
        left: center_x - PLAYER_CHIP_ZONE_WIDTH * 0.5,
        top,
        width: PLAYER_CHIP_ZONE_WIDTH,
        height,
    }
}

pub(super) fn texas_pot_chip_zone() -> ChipZoneLayout {
    ChipZoneLayout {
        left: 440.0,
        // Keep pot chips below the board cards, but reclaim the space that used
        // to be reserved for the "底池" title.
        top: 260.0,
        width: 400.0,
        height: 128.0,
    }
}

pub(super) fn texas_pot_partition_zone(index: usize, count: usize) -> ChipZoneLayout {
    let whole = texas_pot_chip_zone();
    let count = count.max(1);
    let gap = if count > 1 { 8.0 } else { 0.0 };
    let width = (whole.width - gap * count.saturating_sub(1) as f32) / count as f32;
    ChipZoneLayout {
        left: whole.left + index.min(count - 1) as f32 * (width + gap),
        top: whole.top,
        width,
        height: whole.height,
    }
}

/// The visual centre zone contains both the board cards and the physical pot.
/// Chip scattering still uses `texas_pot_chip_zone`, so chips cannot cover cards.
pub(super) fn texas_center_zone_panel() -> ChipZoneLayout {
    ChipZoneLayout {
        // Keep the board and pot in one frame while leaving a gap to the
        // player chip zones above, below, and on both sides.
        left: 420.0,
        top: 158.0,
        width: 440.0,
        height: 240.0,
    }
}

pub(super) fn random_unit(id: u64, salt: u64) -> f32 {
    let mut value = id
        .wrapping_mul(0x9E37_79B9_7F4A_7C15)
        .wrapping_add(salt.wrapping_mul(0xBF58_476D_1CE4_E5B9));
    value ^= value >> 30;
    value = value.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    value ^= value >> 27;
    value = value.wrapping_mul(0x94D0_49BB_1331_11EB);
    value ^= value >> 31;
    (value as u32) as f32 / u32::MAX as f32
}

pub(super) fn random_signed(id: u64, salt: u64) -> f32 {
    random_unit(id, salt) * 2.0 - 1.0
}
