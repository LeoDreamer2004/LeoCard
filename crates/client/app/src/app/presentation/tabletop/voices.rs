use crate::app::runtime::ClientResource;
use bevy::audio::Volume;
use bevy::prelude::*;
use leocard_protocol::{GameEventContext, MatchId, PlayerGender};
use std::collections::VecDeque;

#[derive(Clone, Copy)]
pub(crate) enum CardPatternVoice {
    Straight,
    ConsecutivePairs,
    Airplane,
    Bomb,
    HeavenBomb,
    Rocket,
}

#[derive(Resource)]
pub(crate) struct CardPatternVoices {
    variants: [[[Handle<AudioSource>; 2]; 6]; 2],
    match_id: Option<MatchId>,
    sequence: Option<u128>,
    pending: VecDeque<Handle<AudioSource>>,
}

impl FromWorld for CardPatternVoices {
    fn from_world(world: &mut World) -> Self {
        let server = world.resource::<AssetServer>();
        Self {
            variants: ["male", "female"].map(|gender| {
                [
                    "straight",
                    "consecutive_pairs",
                    "airplane",
                    "bomb",
                    "heaven_bomb",
                    "rocket",
                ]
                .map(|pattern| {
                    [1, 2].map(|variant| {
                        server.load(format!(
                            "audio/card_patterns/{gender}/{pattern}_{variant}.ogg"
                        ))
                    })
                })
            }),
            match_id: None,
            sequence: None,
            pending: VecDeque::new(),
        }
    }
}

impl CardPatternVoices {
    pub(crate) fn queue(
        &mut self,
        context: GameEventContext,
        gender: PlayerGender,
        pattern: CardPatternVoice,
    ) {
        if self.match_id != Some(context.match_id)
            || self
                .sequence
                .is_some_and(|sequence| sequence >= context.sequence)
        {
            return;
        }
        self.sequence = Some(context.sequence);
        let gender = match gender {
            PlayerGender::Male => 0,
            PlayerGender::Female => 1,
        };
        self.pending
            .push_back(self.variants[gender][pattern as usize][fastrand::usize(..2)].clone());
    }
}

#[derive(Component)]
pub(crate) struct CardPatternVoicePlayback;

pub(crate) fn sync_card_pattern_voices(
    client: Option<Res<ClientResource>>,
    mut voices: ResMut<CardPatternVoices>,
    playing: Query<Entity, With<CardPatternVoicePlayback>>,
    mut commands: Commands,
) {
    let match_id = client
        .as_deref()
        .and_then(|client| client.0.model().game_snapshot())
        .map(|game| game.match_id());
    if voices.match_id != match_id {
        voices.match_id = match_id;
        voices.sequence = None;
        voices.pending.clear();
        for entity in &playing {
            commands.entity(entity).despawn();
        }
    }
}

pub(crate) fn play_card_pattern_voices(
    mut voices: ResMut<CardPatternVoices>,
    playing: Query<(), With<CardPatternVoicePlayback>>,
    mut commands: Commands,
) {
    if playing.is_empty()
        && let Some(sound) = voices.pending.pop_front()
    {
        commands.spawn((
            AudioPlayer::new(sound),
            PlaybackSettings {
                volume: Volume::Linear(0.8),
                ..PlaybackSettings::DESPAWN
            },
            CardPatternVoicePlayback,
        ));
    }
}
