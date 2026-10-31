use serde::{Deserialize, Serialize};

#[derive(
    Clone, Copy, Debug, Default, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize,
)]
pub struct PlayerId(pub u8);

impl PlayerId {
    pub const fn from_index(index: usize) -> Self {
        assert!(index <= u8::MAX as usize, "player index must fit in u8");
        Self(index as u8)
    }

    pub const fn index(self) -> usize {
        self.0 as usize
    }
}

impl From<PlayerId> for usize {
    fn from(player: PlayerId) -> Self {
        player.index()
    }
}
