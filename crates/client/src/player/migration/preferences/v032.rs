use super::schema::Rules;
use crate::player::preferences::MahjongPreferences;
use leocard_mahjong::{MahjongMatchLength, MahjongRuleSet, MahjongUmaStyle};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub(super) struct MahjongRules {
    pub match_length: MahjongMatchLength,
    pub minimum_eight_points: bool,
    pub multiple_winners: bool,
    pub false_win: bool,
}

impl From<Rules<MahjongRules>> for MahjongPreferences {
    fn from(value: Rules<MahjongRules>) -> Self {
        let rules = value.host_rules;
        Self {
            host_rules: MahjongRuleSet {
                match_length: rules.match_length,
                uma_style: MahjongUmaStyle::Balanced,
                minimum_eight_points: rules.minimum_eight_points,
                multiple_winners: rules.multiple_winners,
                false_win: rules.false_win,
            },
        }
    }
}
