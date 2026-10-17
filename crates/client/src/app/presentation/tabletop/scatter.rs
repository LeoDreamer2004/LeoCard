use super::PokerChipVisual;
use bevy::prelude::*;

pub(crate) struct PokerChipScatter {
    pub seed: u64,
    pub size: f32,
    pub bounds: Vec2,
}

impl PokerChipScatter {
    pub(crate) fn chip(&self, index: usize, denomination: u16) -> PokerChipVisual {
        let salt = index as u64 * 3;
        PokerChipVisual {
            denomination,
            position: Vec2::new(
                table_random_unit(self.seed, salt) * (self.bounds.x - self.size).max(0.0),
                table_random_unit(self.seed, salt + 1) * (self.bounds.y - self.size).max(0.0),
            ),
            size: self.size,
            rotation: table_random_signed(self.seed, salt + 2) * 0.22,
            layer: index as i32,
        }
    }
}

pub(crate) fn table_random_unit(id: u64, salt: u64) -> f32 {
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

pub(crate) fn table_random_signed(id: u64, salt: u64) -> f32 {
    table_random_unit(id, salt) * 2.0 - 1.0
}
