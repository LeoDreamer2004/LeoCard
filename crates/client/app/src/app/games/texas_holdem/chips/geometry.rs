use super::ChipZoneLayout;

pub(super) fn texas_pot_chip_zone(squid_enabled: bool) -> ChipZoneLayout {
    // Reserve the left column below the street label for squid coins.
    let left = if squid_enabled { 560.0 } else { 440.0 };
    ChipZoneLayout {
        left,
        // Keep pot chips below the board cards, but reclaim the space that used
        // to be reserved for the "底池" title.
        top: 260.0,
        width: 840.0 - left,
        height: 128.0,
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
