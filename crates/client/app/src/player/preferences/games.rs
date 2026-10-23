use leocard_mahjong::MahjongRuleSet;
use leocard_minecraft_guess::MinecraftGuessRuleSet;
use leocard_protocol::TABLE_SEAT_COUNT;
use leocard_qigui523::QiGuiRuleSet;
use leocard_shengji::ShengjiRuleSet;
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
}

#[derive(Deserialize, Serialize)]
pub struct QiGui523Preferences {
    pub host_rules: QiGuiRuleSet,
}

#[derive(Deserialize, Serialize)]
pub struct TexasHoldemPreferences {
    pub host_rules: TexasHoldemRuleSet,
}

#[derive(Default, Deserialize, Serialize)]
pub struct ShengjiPreferences {
    pub host_rules: ShengjiRuleSet,
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
        }
    }
}
