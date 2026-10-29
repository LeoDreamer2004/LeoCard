use crate::app::runtime::{AppearancePreferences, CardFace, ClientResource};
use bevy::audio::Volume;
use bevy::prelude::*;
use leocard_protocol::{GameEventContext, GameKind, MatchId, PlayerGender};
use std::collections::HashMap;

#[derive(Clone, Copy)]
pub(crate) enum CardVoice {
    Single(CardFace),
    Pair(CardFace),
    Triple,
    TripleSequence,
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
    const RANK_FILES: [&str; 15] = [
        "ace",
        "2",
        "3",
        "4",
        "5",
        "6",
        "7",
        "8",
        "9",
        "10",
        "jack",
        "queen",
        "king",
        "small_joker",
        "big_joker",
    ];

    const FILES: [&str; 16] = [
        "triple",
        "triple_sequence",
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

    fn file_name(self) -> String {
        match self {
            Self::Single(face) => return format!("single_{}", Self::rank_file(face)),
            Self::Pair(face) => return format!("pair_{}", Self::rank_file(face)),
            Self::Triple => "triple",
            Self::TripleSequence => "triple_sequence",
            Self::Straight => "straight",
            Self::ConsecutivePairs => "consecutive_pairs",
            Self::Airplane => "airplane",
            Self::Bomb => "bomb",
            Self::HeavenBomb => "heaven_bomb",
            Self::Rocket => "rocket",
            Self::Pass => "pass",
            Self::TripleSingle => "triple_single",
            Self::TriplePair => "triple_pair",
            Self::FourTwo => "four_two",
            Self::BidZero => "bid_zero",
            Self::BidOne => "bid_one",
            Self::BidTwo => "bid_two",
            Self::BidThree => "bid_three",
        }
        .to_owned()
    }

    fn rank_file(face: CardFace) -> &'static str {
        let rank = match face {
            CardFace::Suited { rank, .. } => rank as usize,
            CardFace::SmallJoker => 13,
            CardFace::BigJoker => 14,
        };
        Self::RANK_FILES[rank]
    }
}

#[derive(Resource)]
pub(crate) struct CardVoices {
    variants: HashMap<String, [Vec<Handle<AudioSource>>; 2]>,
    match_id: Option<MatchId>,
    sequence: Option<u128>,
    pending: Option<Handle<AudioSource>>,
}

impl FromWorld for CardVoices {
    fn from_world(world: &mut World) -> Self {
        let server = world.resource::<AssetServer>();
        Self {
            variants: CardVoice::FILES
                .map(|pattern| (pattern.to_owned(), if pattern == "pass" { 6 } else { 2 }))
                .into_iter()
                .chain(["single", "pair"].into_iter().flat_map(|prefix| {
                    CardVoice::RANK_FILES.map(|rank| (format!("{prefix}_{rank}"), 1))
                }))
                .map(|(pattern, count)| {
                    let variants = ["male", "female"].map(|gender| {
                        (1..=count)
                            .map(|variant| {
                                let file = if count == 1 {
                                    format!("{pattern}.ogg")
                                } else {
                                    format!("{pattern}_{variant}.ogg")
                                };
                                server.load(format!("audio/card_patterns/{gender}/{file}"))
                            })
                            .collect()
                    });
                    (pattern, variants)
                })
                .collect(),
            match_id: None,
            sequence: None,
            pending: None,
        }
    }
}

impl CardVoices {
    pub(crate) fn announce(
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
        let variants = &self.variants[&pattern.file_name()][gender];
        // 出牌播报紧跟最新动作，快速操作或批量收到事件时不积压旧语音。
        self.pending = Some(variants[fastrand::usize(..variants.len())].clone());
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
        voices.pending = None;
        for entity in &playing {
            commands.entity(entity).despawn();
        }
    }
}

pub(crate) fn play_card_voices(
    client: Option<Res<ClientResource>>,
    preferences: Res<AppearancePreferences>,
    mut voices: ResMut<CardVoices>,
    playing: Query<Entity, With<CardVoicePlayback>>,
    mut commands: Commands,
) {
    let enabled = client
        .as_deref()
        .and_then(|client| client.0.model().game_snapshot())
        .is_some_and(|game| match game.kind() {
            GameKind::QiGui523 => preferences.qigui_action_voices,
            GameKind::Doudizhu => preferences.doudizhu_action_voices,
            _ => false,
        });
    let sound = voices.pending.take().filter(|_| enabled);
    if enabled && sound.is_none() {
        return;
    }
    for entity in &playing {
        commands.entity(entity).despawn();
    }
    let Some(sound) = sound else {
        return;
    };
    commands.spawn((
        AudioPlayer::new(sound),
        PlaybackSettings {
            volume: Volume::Linear(0.8),
            ..PlaybackSettings::DESPAWN
        },
        CardVoicePlayback,
    ));
}
