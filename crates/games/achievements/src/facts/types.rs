use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct TrophyCounts {
    pub gold: u32,
    pub silver: u32,
    pub bronze: u32,
}

impl TrophyCounts {
    pub const fn by_tier(self) -> [u32; 3] {
        [self.gold, self.silver, self.bronze]
    }
}
