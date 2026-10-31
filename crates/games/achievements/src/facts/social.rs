use leocard_game_common::PlayerId;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InteractionKind {
    Flower,
    Egg,
    Wine,
    Shoe,
}

impl InteractionKind {
    pub const fn flower_count(self) -> u32 {
        match self {
            Self::Flower => 1,
            Self::Wine => 10,
            _ => 0,
        }
    }

    pub const fn egg_count(self) -> u32 {
        match self {
            Self::Egg => 1,
            Self::Shoe => 10,
            _ => 0,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct InteractionFacts {
    pub source: PlayerId,
    pub target: PlayerId,
    pub kind: InteractionKind,
    /// Source-assigned session time, independent of delivery latency.
    pub elapsed_millis: u64,
}

#[derive(Clone, Debug)]
pub enum ChatActivity {
    Text(String),
    QuickVoice,
    Other,
}

#[derive(Clone, Debug)]
pub struct ChatFacts {
    pub source: PlayerId,
    pub content: ChatActivity,
}
