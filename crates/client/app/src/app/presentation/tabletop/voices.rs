use crate::app::runtime::ClientResource;
use bevy::audio::Volume;
use bevy::prelude::*;
use leocard_protocol::{GameEventContext, MatchId, PlayerGender};
use std::collections::VecDeque;

#[derive(Clone, Copy)]
pub(crate) enum CardVoice {
    Straight,
    ConsecutivePairs,
    Airplane,
    Bomb,
    HeavenBomb,
    Rocket,
    Pass,
    TripleSingle,
    TriplePair,
    FourTwo,
    BidZero,
    BidOne,
    BidTwo,
    BidThree,
}

impl CardVoice {
    const FILES: [&str; 14] = [
        "straight",
        "consecutive_pairs",
        "airplane",
        "bomb",
        "heaven_bomb",
        "rocket",
        "pass",
        "triple_single",
        "triple_pair",
        "four_two",
        "bid_zero",
        "bid_one",
        "bid_two",
        "bid_three",
    ];
}

#[derive(Resource)]
pub(crate) struct CardVoices {
    variants: [[[Handle<AudioSource>; 2]; CardVoice::FILES.len()]; 2],
    match_id: Option<MatchId>,
    sequence: Option<u128>,
    pending: VecDeque<Handle<AudioSource>>,
}

impl FromWorld for CardVoices {
    fn from_world(world: &mut World) -> Self {
        let server = world.resource::<AssetServer>();
        Self {
            variants: ["male", "female"].map(|gender| {
                CardVoice::FILES.map(|pattern| {
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

impl CardVoices {
    pub(crate) fn queue(
        &mut self,
        context: GameEventContext,
        gender: PlayerGender,
        pattern: CardVoice,
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
pub(crate) struct CardVoicePlayback;

pub(crate) fn sync_card_voices(
    client: Option<Res<ClientResource>>,
    mut voices: ResMut<CardVoices>,
    playing: Query<Entity, With<CardVoicePlayback>>,
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

pub(crate) fn play_card_voices(
    mut voices: ResMut<CardVoices>,
    playing: Query<(), With<CardVoicePlayback>>,
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
            CardVoicePlayback,
        ));
    }
}
