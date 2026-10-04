use super::{
    AchievementPublication, AchievementRecipient, AchievementSession, AchievementUnlocked,
};
use crate::app::{runtime::ClientResource, shell::UiState};
use bevy::prelude::*;
use leocard_achievements::AchievementTrigger;
use leocard_client::{LocalPlayerProfile, PlayerAchievements};
use leocard_protocol::ClientCommand;

pub(super) fn setup_book(
    mut commands: Commands,
    mut profile: ResMut<LocalPlayerProfile>,
    mut session: ResMut<AchievementSession>,
) {
    let mut book = match PlayerAchievements::load(&profile) {
        Ok(book) => {
            session.enabled = true;
            book
        }
        Err(error) => {
            // Keep the existing file intact. An empty fallback may be displayed,
            // but must never overwrite or publish over an unreadable archive.
            warn!("{error}；本次会话暂停成就记录");
            PlayerAchievements::new(profile.identity.profile_id())
        }
    };
    if session.enabled {
        let counts = book.counts();
        session.record(
            book.trigger(&AchievementTrigger::TrophyTotals(counts)),
            None,
        );
    }
    profile.game_profiles.achievements = book.counts();
    commands.insert_resource(book);
}

pub(super) fn reconcile_connection(
    client: Option<Res<ClientResource>>,
    mut session: ResMut<AchievementSession>,
    mut publication: ResMut<AchievementPublication>,
) {
    if (client.is_none() && session.connected)
        || client.as_ref().is_some_and(|client| client.is_added())
    {
        publication.unlocked.clear();
        session.connection_epoch = session.connection_epoch.wrapping_add(1);
    }
    session.connected = client.is_some();
}

pub(super) fn commit_progress(
    book: Res<PlayerAchievements>,
    mut session: ResMut<AchievementSession>,
    mut publication: ResMut<AchievementPublication>,
    mut profile: ResMut<LocalPlayerProfile>,
    mut ui: ResMut<UiState>,
    mut unlocked: MessageWriter<AchievementUnlocked>,
    time: Res<Time>,
) {
    session.since_save += time.delta_secs();
    if !session.dirty || (session.save_failed && session.since_save < 0.5) {
        return;
    }
    session.since_save = 0.0;
    if let Err(error) = book.save() {
        if !session.save_failed {
            warn!("{error}；稍后重试");
        }
        session.save_failed = true;
        return;
    }
    session.dirty = false;
    session.save_failed = false;
    let epoch = session.connected.then_some(session.connection_epoch);
    for (definition, source) in session.pending.drain(..) {
        if source.is_some() && source == epoch {
            publication.unlocked.push(definition.id.to_owned());
            publication.since_send = 0.5;
        }
        unlocked.write(AchievementUnlocked {
            definition,
            recipient: AchievementRecipient::Local,
        });
    }
    let counts = book.counts();
    if profile.game_profiles.achievements != counts {
        profile.game_profiles.achievements = counts;
        ui.dirty = true;
    }
}

pub(super) fn publish_progress(
    mut client: Option<ResMut<ClientResource>>,
    session: Res<AchievementSession>,
    profile: Res<LocalPlayerProfile>,
    mut publication: ResMut<AchievementPublication>,
    time: Res<Time>,
) {
    if !session.enabled {
        return;
    }
    publication.since_send += time.delta_secs();
    let Some(client) = client.as_deref_mut() else {
        return;
    };
    let Some(profiles) = client.0.model().local_game_profiles() else {
        return;
    };
    let counts = profile.game_profiles.achievements;
    if profiles.achievements == counts {
        publication.unlocked.clear();
        return;
    }
    if publication.since_send >= 0.5
        && client.0.send(ClientCommand::PublishAchievements {
            counts,
            unlocked: publication.unlocked.clone(),
        })
    {
        publication.since_send = 0.0;
    }
}
