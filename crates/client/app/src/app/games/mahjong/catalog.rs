use crate::app::games::{GameDescriptor, GameLobbyRules, GameMetadata};
use leocard_mahjong::{MahjongMatchLength, MahjongRuleSet};
use leocard_protocol::GameKind;

impl GameMetadata for MahjongRuleSet {
    const DESCRIPTOR: GameDescriptor = GameDescriptor {
        kind: GameKind::Mahjong,
        title: "麻将合集",
        description: "八番起和，方城之战",
    };
}

impl GameLobbyRules for MahjongRuleSet {
    fn seat_count(&self) -> u8 {
        MahjongRuleSet::PLAYER_COUNT
    }

    fn accepts_player_count(&self, count: u8) -> bool {
        count == MahjongRuleSet::PLAYER_COUNT
    }

    fn lobby_rule_labels(&self) -> [String; 3] {
        [
            match self.match_length {
                MahjongMatchLength::SingleHand => "单局结算".to_owned(),
                MahjongMatchLength::EastRound => "东风场".to_owned(),
                MahjongMatchLength::HalfGame => "半庄场".to_owned(),
                MahjongMatchLength::FullGame => "全庄场".to_owned(),
            },
            if self.minimum_eight_points {
                "8 番起和".to_owned()
            } else {
                "不限起和番数".to_owned()
            },
            if self.multiple_winners {
                "允许一炮多响".to_owned()
            } else {
                "截和".to_owned()
            },
        ]
    }
}
