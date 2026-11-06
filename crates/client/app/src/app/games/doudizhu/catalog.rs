use crate::app::games::{GameDescriptor, GameLobbyRules, GameMetadata};
use leocard_doudizhu::DoudizhuRuleSet;
use leocard_protocol::GameKind;

impl GameMetadata for DoudizhuRuleSet {
    const DESCRIPTOR: GameDescriptor = GameDescriptor {
        kind: GameKind::Doudizhu,
        title: "斗地主",
        description: "17张牌你能秒我?",
    };
}

impl GameLobbyRules for DoudizhuRuleSet {
    fn seat_count(&self) -> u8 {
        self.mode.players()
    }

    fn accepts_player_count(&self, count: u8) -> bool {
        count == self.mode.players()
    }

    fn lobby_rule_labels(&self) -> [String; 3] {
        [
            self.mode.label().to_owned(),
            format!("{} 副牌", self.mode.deck_count()),
            format!("{} 轮", self.rounds),
        ]
    }
}
