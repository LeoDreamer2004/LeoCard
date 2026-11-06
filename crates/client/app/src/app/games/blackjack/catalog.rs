use crate::app::games::{GameDescriptor, GameLobbyRules, GameMetadata};
use leocard_blackjack::BlackjackRuleSet;
use leocard_protocol::GameKind;

impl GameMetadata for BlackjackRuleSet {
    const DESCRIPTOR: GameDescriptor = GameDescriptor {
        kind: GameKind::Blackjack,
        title: "21 点",
        description: "再来一张，还是见好就收?",
    };
}

impl GameLobbyRules for BlackjackRuleSet {
    fn seat_count(&self) -> u8 {
        BlackjackRuleSet::MAX_PLAYERS
    }

    fn accepts_player_count(&self, count: u8) -> bool {
        (BlackjackRuleSet::MIN_PLAYERS..=BlackjackRuleSet::MAX_PLAYERS).contains(&count)
    }

    fn lobby_rule_labels(&self) -> [String; 3] {
        [
            format!("{} 筹码", self.initial_chips),
            format!("{} 局", self.rounds),
            "系统庄家".to_owned(),
        ]
    }
}
