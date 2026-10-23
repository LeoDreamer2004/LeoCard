use super::adapter::AchievementInput;
use super::{
    AchievementRecipient, AchievementSession, AchievementUnlocked, LocalAchievementTrigger,
};
use crate::app::runtime::ServerNotification;
use bevy::prelude::*;
use leocard_achievements::{AchievementTrigger, PersonalActivity};
use leocard_client::{LocalPlayerProfile, PlayerAchievements};
use leocard_protocol::{ACHIEVEMENT_REGISTRY, ServerEvent};

#[derive(Resource, Default)]
pub(super) struct ActivityObserver {
    epoch: u64,
    activity: PersonalActivity,
}

#[derive(bevy::ecs::system::SystemParam)]
pub(super) struct TriggerResources<'w> {
    profile: Res<'w, LocalPlayerProfile>,
    book: ResMut<'w, PlayerAchievements>,
    session: ResMut<'w, AchievementSession>,
    observer: ResMut<'w, ActivityObserver>,
}

pub(super) fn process_triggers(
    mut notifications: MessageReader<ServerNotification>,
    mut local: MessageReader<LocalAchievementTrigger>,
    mut state: TriggerResources,
    mut unlocked: MessageWriter<AchievementUnlocked>,
) {
    let state = &mut state;
    if state.observer.epoch != state.session.connection_epoch {
        state.observer.epoch = state.session.connection_epoch;
        state.observer.activity = PersonalActivity::default();
    }
    for notification in notifications.read() {
        // A game that cannot settle profiles must not accumulate achievement progress either.
        if notification
            .game_context
            .is_some_and(|context| !context.records_progress)
        {
            continue;
        }
        if state.session.enabled
            && let Some(AchievementInput { trigger, context }) =
                AchievementInput::from_notification(notification)
        {
            let source = state
                .session
                .connected
                .then_some(state.session.connection_epoch);
            state
                .session
                .record(state.book.trigger_with_context(&trigger, context), source);
            if let AchievementTrigger::Interaction { player, event } = trigger
                && let Some(event) = state.observer.activity.observe_interaction(player, &event)
            {
                state.session.record(
                    state.book.trigger(&AchievementTrigger::Personal(event)),
                    source,
                );
            }
        }
        if let ServerEvent::AchievementUnlocked(announcement) = &notification.event
            && announcement.profile_id != state.profile.identity.profile_id()
            && let Some(definition) = ACHIEVEMENT_REGISTRY.by_id(&announcement.achievement_id)
        {
            unlocked.write(AchievementUnlocked {
                definition,
                recipient: AchievementRecipient::TablePlayer {
                    name: announcement.name.clone(),
                },
            });
        }
    }
    for event in local.read() {
        if state.session.enabled {
            state.session.record(state.book.trigger(&event.0), None);
        }
    }
}
