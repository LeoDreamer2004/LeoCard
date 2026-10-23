use crate::player::preferences::{
    GamePreferences, GlobalPreferences, MahjongPreferences, QiGui523Preferences, SavedPreferences,
    ShengjiPreferences, TexasHoldemPreferences, UnoPreferences,
};
use leocard_texas_holdem::TexasHoldemRuleSet;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub(super) struct Archive<G, M = MahjongPreferences, T = Rules<TexasRules>> {
    pub global: G,
    pub games: Games<M, T>,
}

impl<G, M, T> From<Archive<G, M, T>> for SavedPreferences
where
    G: Into<GlobalPreferences>,
    M: Into<MahjongPreferences>,
    T: Into<TexasHoldemPreferences>,
{
    fn from(archive: Archive<G, M, T>) -> Self {
        Self {
            global: archive.global.into(),
            games: archive.games.into(),
        }
    }
}

#[derive(Deserialize, Serialize)]
pub(super) struct Games<M = MahjongPreferences, T = Rules<TexasRules>> {
    pub qigui523: QiGui523Preferences,
    pub texas_holdem: T,
    pub shengji: ShengjiPreferences,
    pub uno: UnoPreferences,
    pub mahjong: M,
}

impl<M, T> From<Games<M, T>> for GamePreferences
where
    M: Into<MahjongPreferences>,
    T: Into<TexasHoldemPreferences>,
{
    fn from(games: Games<M, T>) -> Self {
        Self {
            qigui523: games.qigui523,
            texas_holdem: games.texas_holdem.into(),
            shengji: games.shengji,
            uno: games.uno,
            mahjong: games.mahjong.into(),
            minecraft_guess: Default::default(),
        }
    }
}

#[derive(Deserialize, Serialize)]
pub(super) struct Rules<T> {
    pub host_rules: T,
}

impl From<Rules<TexasRules>> for TexasHoldemPreferences {
    fn from(rules: Rules<TexasRules>) -> Self {
        Self {
            host_rules: rules.host_rules.into(),
        }
    }
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
