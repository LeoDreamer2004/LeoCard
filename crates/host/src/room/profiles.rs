use super::{Participant, RoomSession};
use leocard_protocol::{PlayerId, PlayerReferenceChange};

impl RoomSession {
    /// 机器人占未离开玩家的三分之一及以上时，不记录游戏档案。
    pub(crate) fn records_game_profiles(&self) -> bool {
        let (players, bots) = self
            .players
            .iter()
            .filter(|player| !player.left)
            .fold((0, 0), |(players, bots), player| {
                (players + 1, bots + usize::from(player.is_bot))
            });
        bots * 3 < players
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
