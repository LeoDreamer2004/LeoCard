use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GraphRelation {
    Crafting,
    Cooking,
    Brewing,
    Smithing,
    Stonecutting,
    Loot,
    MiningDrop,
    Gifts,
    GrantsEffect,
    Equippable,
    SpawnWeapon,
    ConvertsTo,
    Enchantable,
    ConflictsWith,
    Generates,
    MemberOf,
    TradeIngredient,
    TradeOutput,
    Workstation,
    SpawnsMob,
    LandscapeSpawns,
    GeneratesLandscape,
    MajorBlock,
    Locates,
    SpawnsEntity,
    Hatches,
    LosesLife,
    Infests,
    Potting,
    Strips,
    Harvest,
    Oxidizes,
    Scrapes,
    Waxes,
    Unwaxes,
    Processing,
    Natural,
    Trading,
    Other,
}

impl GraphRelation {
    /// Concrete relations before unrevealed edges are grouped by `coarse`.
    pub const DETAILED: [Self; 35] = [
        Self::Crafting,
        Self::Cooking,
        Self::Brewing,
        Self::Smithing,
        Self::Stonecutting,
        Self::Loot,
        Self::MiningDrop,
        Self::Gifts,
        Self::GrantsEffect,
        Self::Equippable,
        Self::SpawnWeapon,
        Self::ConvertsTo,
        Self::Enchantable,
        Self::ConflictsWith,
        Self::Generates,
        Self::MemberOf,
        Self::TradeIngredient,
        Self::TradeOutput,
        Self::Workstation,
        Self::SpawnsMob,
        Self::LandscapeSpawns,
        Self::GeneratesLandscape,
        Self::MajorBlock,
        Self::Locates,
        Self::SpawnsEntity,
        Self::Hatches,
        Self::LosesLife,
        Self::Infests,
        Self::Potting,
        Self::Strips,
        Self::Harvest,
        Self::Oxidizes,
        Self::Scrapes,
        Self::Waxes,
        Self::Unwaxes,
    ];

    pub fn coarse(self) -> Self {
        match self {
            Self::Processing | Self::Natural | Self::Trading | Self::Other => self,
            Self::Crafting
            | Self::Enchantable
            | Self::ConflictsWith
            | Self::GrantsEffect
            | Self::MemberOf => self,
            Self::Cooking | Self::Brewing | Self::Smithing | Self::Stonecutting => Self::Processing,
            Self::Loot | Self::MiningDrop | Self::Gifts => Self::Loot,
            Self::Generates
            | Self::SpawnsMob
            | Self::LandscapeSpawns
            | Self::GeneratesLandscape
            | Self::MajorBlock => Self::Natural,
            Self::TradeIngredient | Self::TradeOutput | Self::Workstation => Self::Trading,
            _ => Self::Other,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Crafting => "合成原料",
            Self::Cooking => "烧制成",
            Self::Brewing => "酿造成",
            Self::Smithing => "锻造原料",
            Self::Stonecutting => "切石成",
            Self::Loot => "战利品",
            Self::MiningDrop => "挖掘掉落",
            Self::Gifts => "赠予",
            Self::GrantsEffect => "给予效果",
            Self::Equippable => "可装备",
            Self::SpawnWeapon => "配备武器",
            Self::ConvertsTo => "生物转化",
            Self::Enchantable => "可附魔于",
            Self::ConflictsWith => "附魔冲突",
            Self::Generates => "自然生成",
            Self::MemberOf => "属于标签",
            Self::TradeIngredient => "交易原料",
            Self::TradeOutput => "交易产出",
            Self::Workstation => "职业工作方块",
            Self::SpawnsMob => "群系生成生物",
            Self::LandscapeSpawns => "景观生成生物",
            Self::GeneratesLandscape => "群系生成景观",
            Self::MajorBlock => "群系主要方块",
            Self::Locates => "定位",
            Self::SpawnsEntity => "生成生物",
            Self::Hatches => "孵化",
            Self::LosesLife => "失活",
            Self::Infests => "虫蚀",
            Self::Potting => "种植盆",
            Self::Strips => "去皮",
            Self::Harvest => "收获",
            Self::Oxidizes => "氧化",
            Self::Scrapes => "除锈",
            Self::Waxes => "涂蜡",
            Self::Unwaxes => "脱蜡",
            Self::Processing => "加工成",
            Self::Natural => "自然生成",
            Self::Trading => "交易",
            Self::Other => "其他",
        }
    }
}
