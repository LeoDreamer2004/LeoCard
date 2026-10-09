use super::super::MahjongAssets;
use super::state::{MahjongActionPrompt, MahjongPromptPlayback};
use bevy::{audio::Volume, prelude::*};

pub(in super::super) fn play_mahjong_action_prompt(
    mut commands: Commands,
    assets: Res<MahjongAssets>,
    prompts: Query<&MahjongActionPrompt>,
    mut playback: ResMut<MahjongPromptPlayback>,
) {
    let prompt = prompts.iter().next().copied();
    if prompt != playback.previous {
        if prompt.is_some() {
            commands.spawn((
                AudioPlayer::new(assets.action_prompt.clone()),
                PlaybackSettings {
                    volume: Volume::Linear(0.48),
                    ..PlaybackSettings::DESPAWN
                },
            ));
        }
        playback.previous = prompt;
    }
}
