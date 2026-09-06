//! 跨游戏共用的发牌音效调度。

use crate::app::UiAssets;
use bevy::prelude::*;

#[derive(Component)]
pub struct PendingDealSound {
    pub remaining: f32,
    pub variant: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CardPlaySoundKind {
    Place,
    Shove,
}

pub fn play_pending_deal_sounds(
    time: Res<Time>,
    assets: Res<UiAssets>,
    mut commands: Commands,
    mut pending: Query<(Entity, &mut PendingDealSound)>,
) {
    for (entity, mut sound) in &mut pending {
        sound.remaining -= time.delta_secs();
        if sound.remaining > 0.0 {
            continue;
        }
        if let Some(handle) = assets.audio.deal_sounds.get(sound.variant) {
            commands.spawn((AudioPlayer::new(handle.clone()), PlaybackSettings::DESPAWN));
        }
        commands.entity(entity).despawn();
    }
}
