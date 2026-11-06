use crate::app::games::{GameDescriptor, GameLobbyRules, GameMetadata};
use leocard_protocol::GameKind;
use leocard_texas_holdem::TexasRuleSet;

impl GameMetadata for TexasRuleSet {
    const DESCRIPTOR: GameDescriptor = GameDescriptor {
        kind: GameKind::Texas,
        title: "德州扑克",
        description: "窝要验牌!",
    };
}

impl GameLobbyRules for TexasRuleSet {
    fn seat_count(&self) -> u8 {
        TexasRuleSet::MAX_PLAYERS
    }

    fn accepts_player_count(&self, count: u8) -> bool {
        self.supports_player_count(count)
    }

    fn lobby_rule_labels(&self) -> [String; 3] {
        [
            format!("{}筹码", self.starting_chips),
            match (self.omaha, self.short_deck) {
                (true, true) => "短牌奥马哈".to_owned(),
                (true, false) => "奥马哈".to_owned(),
                (false, true) => "短牌德州".to_owned(),
                (false, false) => "标准德州".to_owned(),
            },
            if self.ignore_kickers {
                "只比较最大牌型".to_owned()
            } else {
                "标准比牌".to_owned()
            },
        ]
    }
}
