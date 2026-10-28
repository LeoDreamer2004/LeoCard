use leocard_doudizhu::{Mode as DoudizhuMode, RuleSet as DoudizhuRuleSet};
use leocard_mahjong::MahjongRuleSet;
use leocard_minecraft_guess::MinecraftGuessRuleSet;
use leocard_protocol::TABLE_SEAT_COUNT;
use leocard_qigui523::QiGuiRuleSet;
use leocard_shengji::{ShengjiMode, ShengjiRuleSet};
use leocard_texas_holdem::TexasHoldemRuleSet;
use leocard_uno::UnoRuleSet;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct GamePreferences {
    pub qigui523: QiGui523Preferences,
    pub texas_holdem: TexasHoldemPreferences,
    pub shengji: ShengjiPreferences,
    pub uno: UnoPreferences,
    pub mahjong: MahjongPreferences,
    pub minecraft_guess: MinecraftGuessPreferences,
    pub doudizhu: DoudizhuPreferences,
}

#[derive(Deserialize, Serialize)]
pub struct QiGui523Preferences {
    pub host_rules: QiGuiRuleSet,
}

#[derive(Deserialize, Serialize)]
pub struct TexasHoldemPreferences {
    pub host_rules: TexasHoldemRuleSet,
}

#[derive(Deserialize, Serialize)]
pub struct ShengjiPreferences {
    pub host_rules: ShengjiRuleSet,
    pub mode_rules: [ShengjiRuleSet; 2],
}

impl ShengjiPreferences {
    fn default_modes() -> [ShengjiRuleSet; 2] {
        [
            ShengjiRuleSet::default(),
            ShengjiRuleSet::for_mode(ShengjiMode::SixPlayer),
        ]
    }

    pub fn remembered_modes(&self) -> [ShengjiRuleSet; 2] {
        let mut modes = self.mode_rules;
        modes[self.host_rules.mode as usize] = self.host_rules;
        modes
    }
}

impl Default for ShengjiPreferences {
    fn default() -> Self {
        Self {
            host_rules: ShengjiRuleSet::default(),
            mode_rules: Self::default_modes(),
        }
    }
}

#[derive(Default, Deserialize, Serialize)]
pub struct UnoPreferences {
    pub host_rules: UnoRuleSet,
}

#[derive(Default, Deserialize, Serialize)]
pub struct MahjongPreferences {
    pub host_rules: MahjongRuleSet,
}

#[derive(Default, Deserialize, Serialize)]
pub struct MinecraftGuessPreferences {
    pub host_rules: MinecraftGuessRuleSet,
}

#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct DoudizhuPreferences {
    pub host_rules: DoudizhuRuleSet,
    pub mode_rules: [DoudizhuRuleSet; 2],
}

impl DoudizhuPreferences {
    pub fn remembered_modes(&self) -> [DoudizhuRuleSet; 2] {
        let mut modes = self.mode_rules;
        modes[self.host_rules.mode as usize] = self.host_rules;
        modes
    }
}

impl Default for DoudizhuPreferences {
    fn default() -> Self {
        Self {
            host_rules: DoudizhuRuleSet::default(),
            mode_rules: [DoudizhuMode::Three, DoudizhuMode::Four].map(|mode| DoudizhuRuleSet {
                mode,
                ..DoudizhuRuleSet::default()
            }),
        }
    }
}

impl Default for GamePreferences {
    fn default() -> Self {
        Self {
            qigui523: QiGui523Preferences {
                host_rules: QiGuiRuleSet {
                    player_count: TABLE_SEAT_COUNT,
                    ..QiGuiRuleSet::default()
                },
            },
            texas_holdem: TexasHoldemPreferences {
                host_rules: TexasHoldemRuleSet {
                    player_count: TexasHoldemRuleSet::MAX_PLAYERS,
                    ..TexasHoldemRuleSet::default()
                },
            },
            shengji: ShengjiPreferences::default(),
            uno: UnoPreferences::default(),
            mahjong: MahjongPreferences::default(),
            minecraft_guess: MinecraftGuessPreferences::default(),
            doudizhu: DoudizhuPreferences::default(),
        }
    }
}
