use crate::app::games::{GameDescriptor, GameLobbyRules, GameMetadata};
use leocard_protocol::GameKind;
use leocard_shengji::ShengjiRuleSet;

impl GameMetadata for ShengjiRuleSet {
    const DESCRIPTOR: GameDescriptor = GameDescriptor {
        kind: GameKind::Shengji,
        title: "升级",
        description: "神对手 or 猪队友",
    };
}

impl GameLobbyRules for ShengjiRuleSet {
    fn seat_count(&self) -> u8 {
        self.player_count()
    }

    fn accepts_player_count(&self, count: u8) -> bool {
        count == self.player_count()
    }

    fn lobby_rule_labels(&self) -> [String; 3] {
        [
            format!("{}副牌", self.deck_count),
            if self.bottom_copy {
                "允许抄底".to_owned()
            } else {
                "不抄底".to_owned()
            },
            if self.player_count() == 6 {
                "3v3".to_owned()
            } else if self.five_trump_crossing {
                "五主过江".to_owned()
            } else {
                "不过江".to_owned()
            },
        ]
    }
}
