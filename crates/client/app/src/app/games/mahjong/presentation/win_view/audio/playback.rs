use super::super::major::MajorShowcaseSequence;
use super::stage::{ShowcaseAudioStage, ShowcaseClip};
use crate::app::games::mahjong::MahjongAssets;
use crate::app::presentation::GameSummaryAnimation;
use crate::app::runtime::ClientResource;
use bevy::audio::{AudioSinkPlayback, GlobalVolume, Volume};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use leocard_protocol::{GameSnapshot, MahjongPhaseView, MatchId};
use std::collections::HashSet;
use std::time::Duration;

#[derive(Resource, Default)]
pub(crate) struct MahjongShowcasePlayback {
    hand: Option<(MatchId, u8)>,
    played: HashSet<(u32, usize)>,
}

#[derive(Component)]
pub(crate) struct MahjongShowcasePlayer {
    hand: (MatchId, u8),
    end_at: f32,
    volume: f32,
    fade: f32,
}

#[derive(SystemParam)]
pub(crate) struct ShowcaseAudioContext<'w, 's> {
    stages: Query<'w, 's, &'static ShowcaseAudioStage>,
    playback: ResMut<'w, MahjongShowcasePlayback>,
    playing: Query<
        'w,
        's,
        (
            Entity,
            &'static MahjongShowcasePlayer,
            Option<&'static mut AudioSink>,
        ),
    >,
}

pub(crate) fn play_mahjong_showcase_sounds(
    client: Option<Res<ClientResource>>,
    animation: Res<GameSummaryAnimation>,
    global_volume: Res<GlobalVolume>,
    assets: Res<MahjongAssets>,
    context: ShowcaseAudioContext,
    mut commands: Commands,
) {
    let ShowcaseAudioContext {
        stages,
        mut playback,
        mut playing,
    } = context;
    let hand = client
        .as_deref()
        .and_then(|client| client.0.model().game_snapshot())
        .and_then(GameSnapshot::mahjong)
        .and_then(|game| match &game.phase {
            MahjongPhaseView::Finished { result }
                if animation.match_id == Some(game.match_id)
                    && animation.settlement_index == Some(u32::from(result.sequence_index)) =>
            {
                Some((game.match_id, result.sequence_index))
            }
            _ => None,
        });
    if playback.hand != hand {
        playback.hand = hand;
        playback.played.clear();
    }
    for (entity, player, sink) in &mut playing {
        let remaining = player.end_at - animation.elapsed;
        if hand != Some(player.hand) || remaining <= 0.0 {
            commands.entity(entity).despawn();
        } else if player.fade > 0.0
            && let Some(mut sink) = sink
        {
            sink.set_volume(
                Volume::Linear(player.volume * (remaining / player.fade).clamp(0.0, 1.0))
                    * global_volume.volume,
            );
        }
    }
    let Some(hand) = hand else { return };
    for stage in &stages {
        if stage.hand != hand {
            continue;
        }
        let local = stage.timing.local(animation.elapsed);
        let remaining = stage.timing.duration - local;
        for (cue, age) in stage.due(local, &mut playback.played) {
            let fade = if cue.clip == ShowcaseClip::Background {
                MajorShowcaseSequence::FADE
            } else {
                0.0
            };
            let volume = cue.volume
                * if fade > 0.0 {
                    (remaining / fade).clamp(0.0, 1.0)
                } else {
                    1.0
                };
            commands.spawn((
                MahjongShowcasePlayer {
                    hand,
                    end_at: animation.elapsed + remaining,
                    volume: cue.volume,
                    fade,
                },
                AudioPlayer::new(assets.showcase_sounds.clip(cue.clip)),
                PlaybackSettings {
                    volume: Volume::Linear(volume),
                    start_position: cue.clip.sustained().then(|| Duration::from_secs_f32(age)),
                    duration: Some(Duration::from_secs_f32(remaining)),
                    ..PlaybackSettings::DESPAWN
                },
            ));
        }
    }
}
