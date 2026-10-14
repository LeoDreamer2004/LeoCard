use crate::player::preferences::{
    GamePreferences, GlobalPreferences, MahjongPreferences, QiGui523Preferences, SavedPreferences,
    ShengjiPreferences, TexasHoldemPreferences, UnoPreferences,
};
use leocard_texas_holdem::TexasHoldemRuleSet;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub(super) struct Archive<G, M = MahjongPreferences> {
    pub global: G,
    pub games: Games<M>,
}

impl<G: Into<GlobalPreferences>, M: Into<MahjongPreferences>> From<Archive<G, M>>
    for SavedPreferences
{
    fn from(archive: Archive<G, M>) -> Self {
        Self {
            global: archive.global.into(),
            games: archive.games.into(),
        }
    }
}

#[derive(Deserialize, Serialize)]
pub(super) struct Games<M = MahjongPreferences> {
    pub qigui523: QiGui523Preferences,
    pub texas_holdem: Rules<TexasRules>,
    pub shengji: ShengjiPreferences,
    pub uno: UnoPreferences,
    pub mahjong: M,
}

impl<M: Into<MahjongPreferences>> From<Games<M>> for GamePreferences {
    fn from(games: Games<M>) -> Self {
        Self {
            qigui523: games.qigui523,
            texas_holdem: TexasHoldemPreferences {
                host_rules: games.texas_holdem.host_rules.into(),
            },
            shengji: games.shengji,
            uno: games.uno,
            mahjong: games.mahjong.into(),
        }
    }
}

#[derive(Deserialize, Serialize)]
pub(super) struct Rules<T> {
    pub host_rules: T,
}

/// v0.8.4 及以前的德州规则布局；固定限注为新增设置，迁移时默认关闭。
#[derive(Deserialize, Serialize)]
pub(super) struct TexasRules {
    pub player_count: u8,
    pub starting_chips: u16,
    pub short_deck: bool,
    pub ignore_kickers: bool,
    pub omaha: bool,
}

impl From<TexasRules> for TexasHoldemRuleSet {
    fn from(rules: TexasRules) -> Self {
        Self {
            player_count: rules.player_count,
            starting_chips: rules.starting_chips,
            short_deck: rules.short_deck,
            ignore_kickers: rules.ignore_kickers,
            omaha: rules.omaha,
            fixed_limit: false,
        }
    }
}
