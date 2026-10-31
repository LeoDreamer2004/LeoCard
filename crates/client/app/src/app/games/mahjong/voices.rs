//! 国标麻将番种报读音频及结算时间轴。

use super::{MahjongActionVoice, MahjongAssets};
use crate::app::presentation::{
    GameSummaryAnimation, SUMMARY_ROW_ENTRY_DURATION, SUMMARY_ROW_START_DELAY,
};
use crate::app::runtime::{AppearancePreferences, ClientResource};
use bevy::audio::Volume;
use leocard_game_common::PlayerId;
use leocard_mahjong::MahjongFan;
use leocard_protocol::{
    GameSnapshot, MahjongHandResultView, MahjongPhaseView, MahjongSnapshot, MatchId,
};
use leocard_ui::UiComposer;

use bevy::prelude::*;

pub(super) fn queue_mahjong_action_voice(
    commands: &mut UiComposer,
    assets: &MahjongAssets,
    preferences: &AppearancePreferences,
    game: &MahjongSnapshot,
    player: PlayerId,
    action: MahjongActionVoice,
) {
    if !preferences.mahjong_action_voices {
        return;
    }
    let Some(profile) = game
        .players
        .iter()
        .find(|profile| profile.player.id == player)
    else {
        return;
    };
    let sound = assets.action_voice(profile.player.game_profiles.gender, action);
    commands.effect((
        AudioPlayer::new(sound.clone()),
        PlaybackSettings {
            volume: Volume::Linear(0.88),
            ..PlaybackSettings::DESPAWN
        },
    ));
}

pub(super) struct MahjongFanVoiceSpec {
    pub fan: MahjongFan,
    pub duration: f32,
}

// 与 Fan 枚举对应；时长取男、女两套 Ogg 的较长值，避免结算动画抢在语音前出现。
pub(super) const FAN_VOICE_SPECS: [MahjongFanVoiceSpec; 81] = [
    MahjongFanVoiceSpec {
        fan: MahjongFan::BigFourWinds,
        duration: 1.040,
    },
    MahjongFanVoiceSpec {
        fan: MahjongFan::BigThreeDragons,
        duration: 0.960,
    },
    MahjongFanVoiceSpec {
        fan: MahjongFan::AllGreen,
        duration: 0.960,
    },
    MahjongFanVoiceSpec {
        fan: MahjongFan::NineGates,
        duration: 1.360,
    },
    MahjongFanVoiceSpec {
        fan: MahjongFan::FourKongs,
        duration: 0.880,
    },
    MahjongFanVoiceSpec {
        fan: MahjongFan::SevenShiftedPairs,
        duration: 0.880,
    },
    MahjongFanVoiceSpec {
        fan: MahjongFan::ThirteenOrphans,
        duration: 0.960,
    },
    MahjongFanVoiceSpec {
        fan: MahjongFan::AllTerminals,
        duration: 1.280,
    },
    MahjongFanVoiceSpec {
        fan: MahjongFan::LittleFourWinds,
        duration: 1.120,
    },
    MahjongFanVoiceSpec {
        fan: MahjongFan::LittleThreeDragons,
        duration: 0.960,
    },
    MahjongFanVoiceSpec {
        fan: MahjongFan::AllHonors,
        duration: 0.880,
    },
    MahjongFanVoiceSpec {
        fan: MahjongFan::FourConcealedPungs,
        duration: 1.120,
    },
    MahjongFanVoiceSpec {
        fan: MahjongFan::PureTerminalChows,
        duration: 1.280,
    },
    MahjongFanVoiceSpec {
        fan: MahjongFan::QuadrupleChow,
        duration: 1.760,
    },
    MahjongFanVoiceSpec {
        fan: MahjongFan::FourPureShiftedPungs,
        duration: 1.200,
    },
    MahjongFanVoiceSpec {
        fan: MahjongFan::FourPureShiftedChows,
        duration: 1.120,
    },
    MahjongFanVoiceSpec {
        fan: MahjongFan::ThreeKongs,
        duration: 1.040,
    },
    MahjongFanVoiceSpec {
        fan: MahjongFan::AllTerminalsAndHonors,
        duration: 1.120,
    },
    MahjongFanVoiceSpec {
        fan: MahjongFan::SevenPairs,
        duration: 0.880,
    },
    MahjongFanVoiceSpec {
        fan: MahjongFan::GreaterHonorsAndKnittedTiles,
        duration: 1.200,
    },
    MahjongFanVoiceSpec {
        fan: MahjongFan::AllEvenPungs,
        duration: 1.280,
    },
    MahjongFanVoiceSpec {
        fan: MahjongFan::FullFlush,
        duration: 1.280,
    },
    MahjongFanVoiceSpec {
        fan: MahjongFan::PureTripleChow,
        duration: 1.680,
    },
    MahjongFanVoiceSpec {
        fan: MahjongFan::PureShiftedPungs,
        duration: 1.520,
    },
    MahjongFanVoiceSpec {
        fan: MahjongFan::UpperTiles,
        duration: 0.880,
    },
    MahjongFanVoiceSpec {
        fan: MahjongFan::MiddleTiles,
        duration: 0.880,
    },
    MahjongFanVoiceSpec {
        fan: MahjongFan::LowerTiles,
        duration: 1.120,
    },
    MahjongFanVoiceSpec {
        fan: MahjongFan::PureStraight,
        duration: 1.200,
    },
    MahjongFanVoiceSpec {
        fan: MahjongFan::ThreeSuitedTerminalChows,
        duration: 1.680,
    },
    MahjongFanVoiceSpec {
        fan: MahjongFan::PureShiftedChows,
        duration: 1.280,
    },
    MahjongFanVoiceSpec {
        fan: MahjongFan::AllFives,
        duration: 1.120,
    },
    MahjongFanVoiceSpec {
        fan: MahjongFan::TriplePung,
        duration: 1.120,
    },
    MahjongFanVoiceSpec {
        fan: MahjongFan::ThreeConcealedPungs,
        duration: 1.200,
    },
    MahjongFanVoiceSpec {
        fan: MahjongFan::LesserHonorsAndKnittedTiles,
        duration: 1.040,
    },
    MahjongFanVoiceSpec {
        fan: MahjongFan::KnittedStraight,
        duration: 1.120,
    },
    MahjongFanVoiceSpec {
        fan: MahjongFan::UpperFour,
        duration: 1.280,
    },
    MahjongFanVoiceSpec {
        fan: MahjongFan::LowerFour,
        duration: 1.200,
    },
    MahjongFanVoiceSpec {
        fan: MahjongFan::BigThreeWinds,
        duration: 1.200,
    },
    MahjongFanVoiceSpec {
        fan: MahjongFan::MixedStraight,
        duration: 0.960,
    },
    MahjongFanVoiceSpec {
        fan: MahjongFan::ReversibleTiles,
        duration: 1.120,
    },
    MahjongFanVoiceSpec {
        fan: MahjongFan::MixedTripleChow,
        duration: 1.600,
    },
    MahjongFanVoiceSpec {
        fan: MahjongFan::MixedShiftedPungs,
        duration: 1.520,
    },
    MahjongFanVoiceSpec {
        fan: MahjongFan::ChickenHand,
        duration: 0.960,
    },
    MahjongFanVoiceSpec {
        fan: MahjongFan::LastTileDraw,
        duration: 1.360,
    },
    MahjongFanVoiceSpec {
        fan: MahjongFan::LastTileClaim,
        duration: 1.120,
    },
    MahjongFanVoiceSpec {
        fan: MahjongFan::OutWithReplacementTile,
        duration: 1.280,
    },
    MahjongFanVoiceSpec {
        fan: MahjongFan::RobbingTheKong,
        duration: 1.040,
    },
    MahjongFanVoiceSpec {
        fan: MahjongFan::TwoConcealedKongs,
        duration: 1.120,
    },
    MahjongFanVoiceSpec {
        fan: MahjongFan::AllPungs,
        duration: 1.200,
    },
    MahjongFanVoiceSpec {
        fan: MahjongFan::HalfFlush,
        duration: 1.280,
    },
    MahjongFanVoiceSpec {
        fan: MahjongFan::MixedShiftedChows,
        duration: 2.000,
    },
    MahjongFanVoiceSpec {
        fan: MahjongFan::AllTypes,
        duration: 1.280,
    },
    MahjongFanVoiceSpec {
        fan: MahjongFan::MeldedHand,
        duration: 1.040,
    },
    MahjongFanVoiceSpec {
        fan: MahjongFan::TwoDragonPungs,
        duration: 1.120,
    },
    MahjongFanVoiceSpec {
        fan: MahjongFan::OutsideHand,
        duration: 1.200,
    },
    MahjongFanVoiceSpec {
        fan: MahjongFan::FullyConcealedHand,
        duration: 1.200,
    },
    MahjongFanVoiceSpec {
        fan: MahjongFan::TwoMeldedKongs,
        duration: 1.120,
    },
    MahjongFanVoiceSpec {
        fan: MahjongFan::LastTile,
        duration: 0.720,
    },
    MahjongFanVoiceSpec {
        fan: MahjongFan::DragonPung,
        duration: 0.960,
    },
    MahjongFanVoiceSpec {
        fan: MahjongFan::PrevalentWind,
        duration: 1.440,
    },
    MahjongFanVoiceSpec {
        fan: MahjongFan::SeatWind,
        duration: 0.880,
    },
    MahjongFanVoiceSpec {
        fan: MahjongFan::ConcealedHand,
        duration: 0.960,
    },
    MahjongFanVoiceSpec {
        fan: MahjongFan::AllChows,
        duration: 1.280,
    },
    MahjongFanVoiceSpec {
        fan: MahjongFan::TileHog,
        duration: 0.960,
    },
    MahjongFanVoiceSpec {
        fan: MahjongFan::DoublePung,
        duration: 1.280,
    },
    MahjongFanVoiceSpec {
        fan: MahjongFan::TwoConcealedPungs,
        duration: 1.440,
    },
    MahjongFanVoiceSpec {
        fan: MahjongFan::ConcealedKong,
        duration: 1.040,
    },
    MahjongFanVoiceSpec {
        fan: MahjongFan::AllSimples,
        duration: 1.120,
    },
    MahjongFanVoiceSpec {
        fan: MahjongFan::PureDoubleChow,
        duration: 1.200,
    },
    MahjongFanVoiceSpec {
        fan: MahjongFan::MixedDoubleChow,
        duration: 1.360,
    },
    MahjongFanVoiceSpec {
        fan: MahjongFan::ShortStraight,
        duration: 1.120,
    },
    MahjongFanVoiceSpec {
        fan: MahjongFan::TwoTerminalChows,
        duration: 1.280,
    },
    MahjongFanVoiceSpec {
        fan: MahjongFan::PungOfTerminalsOrHonors,
        duration: 1.360,
    },
    MahjongFanVoiceSpec {
        fan: MahjongFan::MeldedKong,
        duration: 0.960,
    },
    MahjongFanVoiceSpec {
        fan: MahjongFan::OneVoidedSuit,
        duration: 1.280,
    },
    MahjongFanVoiceSpec {
        fan: MahjongFan::NoHonors,
        duration: 1.120,
    },
    MahjongFanVoiceSpec {
        fan: MahjongFan::EdgeWait,
        duration: 1.040,
    },
    MahjongFanVoiceSpec {
        fan: MahjongFan::ClosedWait,
        duration: 1.200,
    },
    MahjongFanVoiceSpec {
        fan: MahjongFan::SingleWait,
        duration: 0.960,
    },
    MahjongFanVoiceSpec {
        fan: MahjongFan::SelfDrawn,
        duration: 1.200,
    },
    MahjongFanVoiceSpec {
        fan: MahjongFan::FlowerTiles,
        duration: 1.040,
    },
];

fn fan_voice_spec(fan: MahjongFan) -> &'static MahjongFanVoiceSpec {
    let index = match fan {
        MahjongFan::BigFourWinds => 0,
        MahjongFan::BigThreeDragons => 1,
        MahjongFan::AllGreen => 2,
        MahjongFan::NineGates => 3,
        MahjongFan::FourKongs => 4,
        MahjongFan::SevenShiftedPairs => 5,
        MahjongFan::ThirteenOrphans => 6,
        MahjongFan::AllTerminals => 7,
        MahjongFan::LittleFourWinds => 8,
        MahjongFan::LittleThreeDragons => 9,
        MahjongFan::AllHonors => 10,
        MahjongFan::FourConcealedPungs => 11,
        MahjongFan::PureTerminalChows => 12,
        MahjongFan::QuadrupleChow => 13,
        MahjongFan::FourPureShiftedPungs => 14,
        MahjongFan::FourPureShiftedChows => 15,
        MahjongFan::ThreeKongs => 16,
        MahjongFan::AllTerminalsAndHonors => 17,
        MahjongFan::SevenPairs => 18,
        MahjongFan::GreaterHonorsAndKnittedTiles => 19,
        MahjongFan::AllEvenPungs => 20,
        MahjongFan::FullFlush => 21,
        MahjongFan::PureTripleChow => 22,
        MahjongFan::PureShiftedPungs => 23,
        MahjongFan::UpperTiles => 24,
        MahjongFan::MiddleTiles => 25,
        MahjongFan::LowerTiles => 26,
        MahjongFan::PureStraight => 27,
        MahjongFan::ThreeSuitedTerminalChows => 28,
        MahjongFan::PureShiftedChows => 29,
        MahjongFan::AllFives => 30,
        MahjongFan::TriplePung => 31,
        MahjongFan::ThreeConcealedPungs => 32,
        MahjongFan::LesserHonorsAndKnittedTiles => 33,
        MahjongFan::KnittedStraight => 34,
        MahjongFan::UpperFour => 35,
        MahjongFan::LowerFour => 36,
        MahjongFan::BigThreeWinds => 37,
        MahjongFan::MixedStraight => 38,
        MahjongFan::ReversibleTiles => 39,
        MahjongFan::MixedTripleChow => 40,
        MahjongFan::MixedShiftedPungs => 41,
        MahjongFan::ChickenHand => 42,
        MahjongFan::LastTileDraw => 43,
        MahjongFan::LastTileClaim => 44,
        MahjongFan::OutWithReplacementTile => 45,
        MahjongFan::RobbingTheKong => 46,
        MahjongFan::TwoConcealedKongs => 47,
        MahjongFan::AllPungs => 48,
        MahjongFan::HalfFlush => 49,
        MahjongFan::MixedShiftedChows => 50,
        MahjongFan::AllTypes => 51,
        MahjongFan::MeldedHand => 52,
        MahjongFan::TwoDragonPungs => 53,
        MahjongFan::OutsideHand => 54,
        MahjongFan::FullyConcealedHand => 55,
        MahjongFan::TwoMeldedKongs => 56,
        MahjongFan::LastTile => 57,
        MahjongFan::DragonPung => 58,
        MahjongFan::PrevalentWind => 59,
        MahjongFan::SeatWind => 60,
        MahjongFan::ConcealedHand => 61,
        MahjongFan::AllChows => 62,
        MahjongFan::TileHog => 63,
        MahjongFan::DoublePung => 64,
        MahjongFan::TwoConcealedPungs => 65,
        MahjongFan::ConcealedKong => 66,
        MahjongFan::AllSimples => 67,
        MahjongFan::PureDoubleChow => 68,
        MahjongFan::MixedDoubleChow => 69,
        MahjongFan::ShortStraight => 70,
        MahjongFan::TwoTerminalChows => 71,
        MahjongFan::PungOfTerminalsOrHonors => 72,
        MahjongFan::MeldedKong => 73,
        MahjongFan::OneVoidedSuit => 74,
        MahjongFan::NoHonors => 75,
        MahjongFan::EdgeWait => 76,
        MahjongFan::ClosedWait => 77,
        MahjongFan::SingleWait => 78,
        MahjongFan::SelfDrawn => 79,
        MahjongFan::FlowerTiles => 80,
    };
    &FAN_VOICE_SPECS[index]
}

#[derive(Clone)]
pub(super) struct MahjongWinnerTiming {
    pub outcome_delay: f32,
    pub hand_delay: f32,
    pub fan_delays: Vec<f32>,
    pub total_delay: f32,
}

pub(super) struct MahjongSettlementTiming {
    pub winners: Vec<MahjongWinnerTiming>,
    pub score_rows_delay: f32,
}

pub(super) fn mahjong_settlement_timing(result: &MahjongHandResultView) -> MahjongSettlementTiming {
    let mut next_delay = SUMMARY_ROW_START_DELAY;
    let mut winners = Vec::with_capacity(result.winners.len());
    for winner in &result.winners {
        let outcome_delay = next_delay;
        next_delay += 0.14;
        let hand_delay = next_delay;
        next_delay += 0.28;
        let mut fan_delays = Vec::with_capacity(winner.score.fans.len());
        for fan in &winner.score.fans {
            fan_delays.push(next_delay);
            next_delay += fan_voice_spec(fan.fan)
                .duration
                .max(SUMMARY_ROW_ENTRY_DURATION)
                + 0.22;
        }
        let total_delay = next_delay;
        next_delay += 1.35 + 0.08;
        winners.push(MahjongWinnerTiming {
            outcome_delay,
            hand_delay,
            fan_delays,
            total_delay,
        });
    }
    MahjongSettlementTiming {
        winners,
        score_rows_delay: next_delay,
    }
}

#[derive(Resource, Default)]
pub(crate) struct MahjongFanVoicePlayback {
    hand: Option<(MatchId, u8)>,
    played: usize,
}

#[derive(Component)]
pub(super) struct MahjongFanVoicePlayer;

pub(crate) fn play_mahjong_fan_voices(
    mut commands: UiComposer,
    client: Option<Res<ClientResource>>,
    animation: Res<GameSummaryAnimation>,
    assets: Res<MahjongAssets>,
    preferences: Res<AppearancePreferences>,
    mut playback: ResMut<MahjongFanVoicePlayback>,
    active_voices: Query<Entity, With<MahjongFanVoicePlayer>>,
) {
    let Some(GameSnapshot::Mahjong(game)) = client
        .as_deref()
        .and_then(|client| client.0.model().game_snapshot())
    else {
        clear_fan_voice_playback(&mut commands, &mut playback, &active_voices);
        return;
    };
    let MahjongPhaseView::Finished { result } = &game.phase else {
        clear_fan_voice_playback(&mut commands, &mut playback, &active_voices);
        return;
    };
    let hand = (game.match_id, result.sequence_index);
    if playback.hand != Some(hand) {
        clear_fan_voice_playback(&mut commands, &mut playback, &active_voices);
        playback.hand = Some(hand);
    }
    if animation.match_id != Some(game.match_id)
        || animation.settlement_index != Some(u32::from(result.sequence_index))
        || animation.elapsed < 0.0
    {
        return;
    }

    if !preferences.mahjong_fan_voices {
        for entity in &active_voices {
            commands.entity(entity).despawn();
        }
    }

    let timing = mahjong_settlement_timing(result);
    let cues = result
        .winners
        .iter()
        .zip(&timing.winners)
        .flat_map(|(winner, timing)| {
            let gender = game
                .players
                .iter()
                .find(|player| player.player.id == winner.player)
                .map(|player| player.player.game_profiles.gender)
                .unwrap_or_default();
            winner
                .score
                .fans
                .iter()
                .zip(&timing.fan_delays)
                .map(move |(fan, delay)| (fan, *delay, gender))
        })
        .collect::<Vec<_>>();
    while let Some((fan, delay, gender)) = cues.get(playback.played).copied() {
        if animation.elapsed < delay {
            break;
        }
        if let Some(sound) = assets.fan_voice(gender, fan.fan)
            && preferences.mahjong_fan_voices
        {
            commands.effect((
                MahjongFanVoicePlayer,
                AudioPlayer::new(sound.clone()),
                PlaybackSettings {
                    volume: Volume::Linear(0.88),
                    ..PlaybackSettings::DESPAWN
                },
            ));
        }
        playback.played += 1;
    }
}

fn clear_fan_voice_playback(
    commands: &mut UiComposer,
    playback: &mut MahjongFanVoicePlayback,
    active_voices: &Query<Entity, With<MahjongFanVoicePlayer>>,
) {
    for entity in active_voices {
        commands.entity(entity).despawn();
    }
    *playback = MahjongFanVoicePlayback::default();
}
