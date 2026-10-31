use crate::McGuessEntryKind;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum McGuessPool {
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

impl McGuessPool {
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

    pub const fn contains(self, kind: McGuessEntryKind) -> bool {
        matches!(
            (self, kind),
            (Self::Blocks | Self::Mixed, McGuessEntryKind::Block)
                | (Self::Items | Self::Mixed, McGuessEntryKind::Item)
                | (Self::Mobs, McGuessEntryKind::Mob)
                | (Self::Enchantments, McGuessEntryKind::Enchantment)
                | (Self::Effects, McGuessEntryKind::Effect)
                | (
                    Self::VillagerProfessions,
                    McGuessEntryKind::VillagerProfession
                )
                | (Self::Landscapes, McGuessEntryKind::Landscape)
                | (Self::Biomes, McGuessEntryKind::Biome)
        )
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct McGuessOptions {
    pub pool: McGuessPool,
    pub candidate_hints: bool,
    pub allow_unknown_guesses: bool,
}

impl Default for McGuessOptions {
    fn default() -> Self {
        Self {
            pool: McGuessPool::default(),
            candidate_hints: true,
            allow_unknown_guesses: false,
        }
    }
}

impl McGuessOptions {
    pub(super) fn is_valid(self) -> bool {
        !(self.candidate_hints && self.allow_unknown_guesses)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct McGuessTurnBasedRules {
    pub options: McGuessOptions,
    pub rounds: u8,
    /// Zero disables the individual turn clock.
    pub guess_seconds: u16,
}

impl Default for McGuessTurnBasedRules {
    fn default() -> Self {
        Self {
            options: McGuessOptions::default(),
            rounds: 1,
            guess_seconds: 60,
        }
    }
}

impl McGuessTurnBasedRules {
    pub const GUESS_SECONDS_OPTIONS: [u16; 5] = [15, 30, 60, 120, 0];

    pub(super) fn is_valid(self) -> bool {
        self.options.is_valid()
            && (1..=5).contains(&self.rounds)
            && Self::GUESS_SECONDS_OPTIONS.contains(&self.guess_seconds)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct McGuessRaceRules {
    pub options: McGuessOptions,
    pub games: u8,
    pub seconds: u16,
}

impl Default for McGuessRaceRules {
    fn default() -> Self {
        Self {
            options: McGuessOptions::default(),
            games: 5,
            seconds: 60,
        }
    }
}

impl McGuessRaceRules {
    pub const SECONDS_OPTIONS: [u16; 5] = [40, 60, 90, 120, 180];
    pub const GAMES_OPTIONS: [u8; 3] = [3, 5, 10];

    pub(super) fn is_valid(self) -> bool {
        self.options.is_valid()
            && Self::GAMES_OPTIONS.contains(&self.games)
            && Self::SECONDS_OPTIONS.contains(&self.seconds)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct McGuessGraphRules {
    pub options: McGuessOptions,
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

impl Default for McGuessGraphRules {
    fn default() -> Self {
        Self {
            options: McGuessOptions::default(),
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

impl McGuessGraphRules {
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
