use super::super::TexasTableLayout;
use super::ChipZoneLayout;

pub(super) fn texas_pot_chip_zone(squid_enabled: bool) -> ChipZoneLayout {
    // Reserve the left column for squid coins and use the gap beneath the board.
    let (left, right) = if cfg!(target_os = "android") {
        (if squid_enabled { 600.0 } else { 349.0 }, 931.0)
    } else {
        (if squid_enabled { 560.0 } else { 440.0 }, 840.0)
    };
    let top = TexasTableLayout::board_card_origin(0).y + TexasTableLayout::BOARD_CARD_SIZE.y + 3.0;
    ChipZoneLayout {
        left,
        top,
        width: right - left,
        height: TexasTableLayout::LOCAL_BET_TOP - top - 7.0,
    }
}

pub(super) fn texas_pot_partition_zone(
    index: usize,
    count: usize,
    squid_enabled: bool,
) -> ChipZoneLayout {
    let whole = texas_pot_chip_zone(squid_enabled);
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
