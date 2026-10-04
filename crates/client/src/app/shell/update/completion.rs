use crate::{app::achievements::LocalAchievementTrigger, updater::completed_update};
use bevy::prelude::*;
use leocard_achievements::{AchievementTrigger, PersonalEvent};
use leocard_client::LocalPlayerProfile;

pub(super) fn emit_update_completion(
    profile: Res<LocalPlayerProfile>,
    mut events: MessageWriter<LocalAchievementTrigger>,
) {
    match completed_update(profile.identity.profile_id()) {
        Ok(true) => {
            events.write(LocalAchievementTrigger(AchievementTrigger::Personal(
                PersonalEvent::ClientUpdated,
            )));
        }
        Ok(false) => {}
        Err(error) => warn!("{error}"),
    }
}
