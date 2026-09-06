use super::*;
use leocard_mahjong::MahjongRuleSet;
use leocard_protocol::RoomId;
use leocard_protocol::{
    AvatarId, ChatMessage, GameRules, GameSnapshot, LobbySnapshot, MahjongSnapshot, MatchId,
    PlayerId, PlayerInteraction, PlayerReferenceChange, QiGui523Snapshot, RejectReason, Revision,
    ShengjiSnapshot, TexasHoldemSnapshot, UnoSnapshot,
};
use leocard_qigui523::QiGuiRuleSet;
use leocard_shengji::ShengjiRuleSet;
use leocard_texas_holdem::TexasHoldemRuleSet;
use leocard_uno::UnoRuleSet;
use std::collections::HashMap;

impl ClientModel {
    pub fn room_id(&self) -> RoomId {
        self.room_id
    }

    pub fn host_port(&self) -> Option<u16> {
        self.host_port
    }

    pub fn latest_revision(&self) -> Revision {
        self.latest_revision
    }

    pub fn you(&self) -> Option<PlayerId> {
        self.you
    }

    pub fn lobby(&self) -> Option<&LobbySnapshot> {
        self.lobby.as_ref()
    }

    pub fn game_snapshot(&self) -> Option<&GameSnapshot> {
        self.game.as_ref()
    }

    pub fn qigui523_game(&self) -> Option<&QiGui523Snapshot> {
        self.game.as_ref().and_then(GameSnapshot::qigui523)
    }

    pub fn texas_holdem_game(&self) -> Option<&TexasHoldemSnapshot> {
        self.game.as_ref().and_then(GameSnapshot::texas_holdem)
    }

    pub fn shengji_game(&self) -> Option<&ShengjiSnapshot> {
        self.game.as_ref().and_then(GameSnapshot::shengji)
    }

    pub fn uno_game(&self) -> Option<&UnoSnapshot> {
        self.game.as_ref().and_then(GameSnapshot::uno)
    }

    pub fn mahjong_game(&self) -> Option<&MahjongSnapshot> {
        self.game.as_ref().and_then(GameSnapshot::mahjong)
    }

    pub fn game_rules(&self) -> Option<&GameRules> {
        self.rules.as_ref()
    }

    pub fn qigui523_rules(&self) -> Option<&QiGuiRuleSet> {
        self.rules.as_ref().and_then(GameRules::qigui523)
    }

    pub fn texas_holdem_rules(&self) -> Option<&TexasHoldemRuleSet> {
        self.rules.as_ref().and_then(GameRules::texas_holdem)
    }

    pub fn shengji_rules(&self) -> Option<&ShengjiRuleSet> {
        self.rules.as_ref().and_then(GameRules::shengji)
    }

    pub fn uno_rules(&self) -> Option<&UnoRuleSet> {
        self.rules.as_ref().and_then(GameRules::uno)
    }

    pub fn mahjong_rules(&self) -> Option<&MahjongRuleSet> {
        self.rules.as_ref().and_then(GameRules::mahjong)
    }

    pub fn avatars(&self) -> &HashMap<AvatarId, Vec<u8>> {
        &self.avatars
    }

    pub fn room_closed(&self) -> bool {
        self.room_closed
    }

    pub fn left_room(&self) -> bool {
        self.left_room
    }

    pub fn last_notice(&self) -> Option<&str> {
        self.notice.value.as_deref()
    }

    pub fn notice_serial(&self) -> u64 {
        self.notice.serial
    }

    pub fn take_player_interactions(&mut self) -> Vec<PlayerInteraction> {
        self.pending.player_interactions.drain(..).collect()
    }

    pub fn take_chat_messages(&mut self) -> Vec<ChatMessage> {
        self.pending.chat_messages.drain(..).collect()
    }

    pub fn last_finished_match(&self) -> Option<(MatchId, &[PlayerReferenceChange])> {
        self.last_finished_match
            .as_ref()
            .map(|(match_id, changes)| (*match_id, changes.as_slice()))
    }

    pub fn last_rejection(&self) -> Option<&RejectReason> {
        self.rejection.value.as_ref()
    }

    /// 每收到一次拒绝消息便递增，即使相邻两次拒绝的内容完全相同。
    pub fn rejection_serial(&self) -> u64 {
        self.rejection.serial
    }
}
