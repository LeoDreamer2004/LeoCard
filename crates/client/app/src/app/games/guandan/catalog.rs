use crate::app::games::{GameDescriptor, GameLobbyRules, GameMetadata};
use leocard_guandan::{GuandanRuleSet, rank_label};
use leocard_protocol::GameKind;

impl GameMetadata for GuandanRuleSet {
    const DESCRIPTOR: GameDescriptor = GameDescriptor {
        kind: GameKind::Guandan,
        title: "掼蛋",
        description: "丐版升级",
    };
}

impl GameLobbyRules for GuandanRuleSet {
    fn seat_count(&self) -> u8 {
        GuandanRuleSet::PLAYER_COUNT
    }

    fn accepts_player_count(&self, count: u8) -> bool {
        count == GuandanRuleSet::PLAYER_COUNT
    }

    fn lobby_rule_labels(&self) -> [String; 3] {
        [
            "四人两队".to_owned(),
            "两副牌".to_owned(),
            format!("从 {} 开始", rank_label(self.starting_level)),
        ]
    }
}
