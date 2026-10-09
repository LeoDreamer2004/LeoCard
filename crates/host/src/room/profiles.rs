use super::{Participant, RoomSession};
use leocard_protocol::{PlayerId, PlayerReferenceChange};

impl RoomSession {
    /// 机器人只能在准备大厅增删；本局有机器人时，所有玩家均不记录游戏档案。
    pub(crate) fn records_game_profiles(&self) -> bool {
        !self
            .players
            .iter()
            .any(|player| player.is_bot && !player.left)
    }

    pub(crate) fn settle_completed_match_profiles_once<I, F>(
        &mut self,
        finished: &mut Option<Vec<PlayerReferenceChange>>,
        settlements: I,
        mut record_game_profile: F,
    ) -> bool
    where
        I: IntoIterator<Item = (PlayerId, i16)>,
        F: FnMut(&mut Participant, i16),
    {
        if finished.is_some() {
            return false;
        }
        // 空结算同样标记本局已结束，客户端不会累计积分、局数或结算游戏金币。
        if !self.records_game_profiles() {
            *finished = Some(Vec::new());
            return true;
        }
        let settlements = settlements.into_iter();
        let mut changes = Vec::with_capacity(settlements.size_hint().0);
        for (player, delta) in settlements {
            let participant = self
                .players
                .iter_mut()
                .find(|participant| participant.id == player)
                .expect("a finished game participant belongs to its room");
            participant.reference_points = participant
                .reference_points
                .saturating_add(i32::from(delta));
            participant.completed_games = participant.completed_games.saturating_add(1);
            record_game_profile(participant, delta);
            changes.push(PlayerReferenceChange {
                player,
                profile_id: participant.profile_id,
                delta,
            });
        }
        *finished = Some(changes);
        true
    }
}
