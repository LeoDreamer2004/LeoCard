use super::super::TexasTableLayout;
use super::ChipZoneLayout;

pub(super) fn texas_pot_chip_zone(squid_enabled: bool) -> ChipZoneLayout {
    if cfg!(target_os = "android") {
        let left = if squid_enabled { 600.0 } else { 349.0 };
        return ChipZoneLayout {
            left,
            top: TexasTableLayout::POT_TOP,
            width: 931.0 - left,
            height: 40.0,
        };
    }
    // Reserve the left column below the street label for squid coins.
    let left = if squid_enabled { 560.0 } else { 440.0 };
    ChipZoneLayout {
        left,
        top: TexasTableLayout::POT_TOP,
        width: 840.0 - left,
        height: 110.0,
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
