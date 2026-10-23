//! Initial denominations and exact-value change rules.

use super::DENOMINATIONS;

pub(super) fn initial_chip_denominations(total: u32) -> Vec<u16> {
    let counts = match total {
        5 => Some((0, 0, 5)),
        10 => Some((0, 1, 5)),
        20 => Some((0, 3, 5)),
        30 => Some((1, 3, 5)),
        40 => Some((2, 3, 5)),
        50 => Some((3, 3, 5)),
        _ => None,
    };
    if let Some((tens, fives, ones)) = counts {
        return std::iter::repeat_n(10, tens)
            .chain(std::iter::repeat_n(5, fives))
            .chain(std::iter::repeat_n(1, ones))
            .collect();
    }
    canonical_chip_denominations(total)
}

pub(super) fn canonical_chip_denominations(mut total: u32) -> Vec<u16> {
    let mut result = Vec::new();
    for denomination in DENOMINATIONS {
        while total >= u32::from(denomination) {
            result.push(denomination);
            total -= u32::from(denomination);
        }
    }
    result
}

pub(super) fn change_for(denomination: u16) -> Vec<u16> {
    match denomination {
        100 => vec![25; 4],
        25 => vec![10, 10, 5],
        10 => vec![5, 5],
        5 => vec![1; 5],
        _ => Vec::new(),
    }
}
