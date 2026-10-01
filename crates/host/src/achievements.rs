//! Room-side projection and transient achievement announcements.
//! The host validates publications; it never evaluates local achievement criteria.

use crate::{ConnectionId, Delivery, RoomSession};
use leocard_achievements::{ACHIEVEMENT_REGISTRY, achievement_by_id, achievement_counts};
use leocard_protocol::{AchievementAnnouncement, AchievementCounts};
use leocard_protocol::{PlayerViolation, ProfileId, RejectReason, RequestId, ServerEvent};
use std::collections::HashSet;

#[derive(Clone, Debug, Default)]
pub(super) struct RoomAchievements {
    announced: HashSet<(ProfileId, String)>,
}

impl RoomSession {
    pub(super) fn publish_achievements(
        &mut self,
        connection: ConnectionId,
        request_id: RequestId,
        counts: AchievementCounts,
        unlocked: Vec<String>,
    ) -> Result<Vec<Delivery>, RejectReason> {
        let player = self
            .players
            .iter_mut()
            .find(|player| player.connection == connection && !player.left)
            .ok_or(RejectReason::Player(PlayerViolation::NotJoined))?;
        let old = player.game_profiles.achievements.by_tier();
        let new = counts.by_tier();
        let maximum =
            achievement_counts(ACHIEVEMENT_REGISTRY.iter().map(|definition| definition.id))
                .by_tier();
        if new.iter().zip(old).any(|(new, old)| *new < old)
            || new
                .iter()
                .zip(maximum)
                .any(|(count, maximum)| *count > maximum)
            || unlocked.len() > ACHIEVEMENT_REGISTRY.len()
            || unlocked.iter().any(|id| achievement_by_id(id).is_none())
        {
            return Err(RejectReason::Player(PlayerViolation::InvalidAchievements));
        }
        let profile_id = player.profile_id;
        let fresh = unlocked
            .iter()
            .filter(|id| {
                !self
                    .achievements
                    .announced
                    .contains(&(profile_id, (*id).clone()))
            })
            .map(String::as_str)
            .collect::<HashSet<_>>();
        let fresh_counts = achievement_counts(fresh).by_tier();
        if fresh_counts
            .iter()
            .zip(new.into_iter().zip(old))
            .any(|(fresh, (new, old))| *fresh > new - old)
        {
            return Err(RejectReason::Player(PlayerViolation::InvalidAchievements));
        }
        let name = player.name.clone();
        let changed = player.game_profiles.achievements != counts;
        player.game_profiles.achievements = counts;
        let mut announcements = Vec::new();
        for id in unlocked {
            if self.achievements.announced.insert((profile_id, id.clone())) {
                announcements.push(AchievementAnnouncement {
                    profile_id,
                    name: name.clone(),
                    achievement_id: id,
                });
            }
        }
        if changed || !announcements.is_empty() {
            self.bump_revision();
        }
        Ok(announcements
            .into_iter()
            .flat_map(|announcement| {
                self.broadcast_event(
                    Some((connection, request_id)),
                    ServerEvent::AchievementUnlocked(announcement),
                )
                .into_iter()
                .filter(|delivery| delivery.recipient != connection)
            })
            .collect())
    }
}
