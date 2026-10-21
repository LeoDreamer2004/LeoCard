use crate::EntryKind;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum GuessPool {
    Blocks,
    Items,
    #[default]
    Mixed,
    Mobs,
    Enchantments,
    Effects,
    VillagerProfessions,
    Landscapes,
    Biomes,
}

impl GuessPool {
    pub const STANDARD: [Self; 3] = [Self::Blocks, Self::Items, Self::Mixed];
    pub const ALL: [Self; 9] = [
        Self::Blocks,
        Self::Items,
        Self::Mixed,
        Self::Mobs,
        Self::Enchantments,
        Self::Effects,
        Self::VillagerProfessions,
        Self::Landscapes,
        Self::Biomes,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Blocks => "方块",
            Self::Items => "物品",
            Self::Mixed => "全部",
            Self::Mobs => "生物",
            Self::Enchantments => "附魔",
            Self::Effects => "状态效果",
            Self::VillagerProfessions => "村民职业",
            Self::Landscapes => "世界景观",
            Self::Biomes => "生物群系",
        }
    }

    pub const fn contains(self, kind: EntryKind) -> bool {
        matches!(
            (self, kind),
            (Self::Blocks | Self::Mixed, EntryKind::Block)
                | (Self::Items | Self::Mixed, EntryKind::Item)
                | (Self::Mobs, EntryKind::Mob)
                | (Self::Enchantments, EntryKind::Enchantment)
                | (Self::Effects, EntryKind::Effect)
                | (Self::VillagerProfessions, EntryKind::VillagerProfession)
                | (Self::Landscapes, EntryKind::Landscape)
                | (Self::Biomes, EntryKind::Biome)
        )
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GuessOptions {
    pub pool: GuessPool,
    pub candidate_hints: bool,
    pub allow_unknown_guesses: bool,
}

impl Default for GuessOptions {
    fn default() -> Self {
        Self {
            pool: GuessPool::default(),
            candidate_hints: true,
            allow_unknown_guesses: false,
        }
    }
}

impl GuessOptions {
    pub(super) fn is_valid(self) -> bool {
        !(self.candidate_hints && self.allow_unknown_guesses)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TurnBasedRules {
    pub options: GuessOptions,
    pub rounds: u8,
    /// Zero disables the individual turn clock.
    pub guess_seconds: u16,
}

impl Default for TurnBasedRules {
    fn default() -> Self {
        Self {
            options: GuessOptions::default(),
            rounds: 1,
            guess_seconds: 60,
        }
    }
}

impl TurnBasedRules {
    pub const GUESS_SECONDS_OPTIONS: [u16; 5] = [15, 30, 60, 120, 0];

    pub(super) fn is_valid(self) -> bool {
        self.options.is_valid()
            && (1..=5).contains(&self.rounds)
            && Self::GUESS_SECONDS_OPTIONS.contains(&self.guess_seconds)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RaceRules {
    pub options: GuessOptions,
    pub games: u8,
    pub seconds: u16,
}

impl Default for RaceRules {
    fn default() -> Self {
        Self {
            options: GuessOptions::default(),
            games: 5,
            seconds: 60,
        }
    }
}

impl RaceRules {
    pub const SECONDS_OPTIONS: [u16; 5] = [40, 60, 90, 120, 180];
    pub const GAMES_OPTIONS: [u8; 3] = [3, 5, 10];

    pub(super) fn is_valid(self) -> bool {
        self.options.is_valid()
            && Self::GAMES_OPTIONS.contains(&self.games)
            && Self::SECONDS_OPTIONS.contains(&self.seconds)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GraphRules {
    pub options: GuessOptions,
    /// Every participant opens one graph; otherwise play one graph with a shuffled order.
    pub round_robin: bool,
    pub nodes: u8,
    /// Fixed-point tenths: 0 is regular, 10 is highly irregular.
    pub temperature: u8,
    /// Zero disables the individual turn clock.
    pub guess_seconds: u16,
    pub show_character_count: bool,
    pub show_directions: bool,
    pub include_tags: bool,
}

impl Default for GraphRules {
    fn default() -> Self {
        Self {
            options: GuessOptions::default(),
            round_robin: true,
            nodes: 16,
            temperature: 5,
            guess_seconds: 120,
            show_character_count: true,
            show_directions: false,
            include_tags: true,
        }
    }
}

impl GraphRules {
    pub const GUESS_SECONDS_OPTIONS: [u16; 5] = [30, 60, 120, 240, 0];
    pub const NODE_OPTIONS: &'static [u8] = if cfg!(debug_assertions) || cfg!(feature = "developer")
    {
        &[2, 16, 24, 32, 40]
    } else {
        &[16, 24, 32, 40]
    };
    pub const TEMPERATURE_OPTIONS: [u8; 11] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

    pub(super) fn is_valid(self) -> bool {
        self.options.is_valid()
            && Self::NODE_OPTIONS.contains(&self.nodes)
            && Self::TEMPERATURE_OPTIONS.contains(&self.temperature)
            && Self::GUESS_SECONDS_OPTIONS.contains(&self.guess_seconds)
    }
}
