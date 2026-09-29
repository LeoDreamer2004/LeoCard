//! 国标麻将番种报读音频及结算时间轴。

use super::{MahjongActionVoice, MahjongAssets};
use crate::app::presentation::{
    GameSummaryAnimation, SUMMARY_ROW_ENTRY_DURATION, SUMMARY_ROW_START_DELAY,
};
use crate::app::runtime::{AppearancePreferences, ClientResource};
use bevy::audio::Volume;
use bevy::prelude::*;
use leocard_mahjong::Fan;
use leocard_protocol::{
    GameSnapshot, MahjongHandResultView, MahjongPhaseView, MahjongSnapshot, MatchId, PlayerId,
};

const FAN_OUTCOME_LEAD: f32 = 0.14;
const FAN_HAND_LEAD: f32 = 0.28;
const FAN_WINNER_GAP: f32 = 0.08;
const FAN_VOICE_HOLD: f32 = 0.22;
const FAN_TOTAL_HOLD: f32 = 1.35;

pub(super) fn queue_mahjong_action_voice(
    commands: &mut Commands,
    assets: &MahjongAssets,
    preferences: &AppearancePreferences,
    game: &MahjongSnapshot,
    player: PlayerId,
    action: MahjongActionVoice,
) {
    if !preferences.mahjong_action_voices {
        return;
    }
    let Some(profile) = game.players.iter().find(|profile| profile.id == player) else {
        return;
    };
    let sound = assets.action_voice(profile.game_profiles.gender, action);
    commands.spawn((
        AudioPlayer::new(sound.clone()),
        PlaybackSettings {
            volume: Volume::Linear(0.88),
            ..PlaybackSettings::DESPAWN
        },
    ));
}

pub(super) struct MahjongFanVoiceSpec {
    pub fan: Fan,
    pub duration: f32,
}

// 与 Fan 枚举对应；时长取男、女两套 Ogg 的较长值，避免结算动画抢在语音前出现。
pub(super) const FAN_VOICE_SPECS: [MahjongFanVoiceSpec; 81] = [
    MahjongFanVoiceSpec {
        fan: Fan::BigFourWinds,
        duration: 1.040,
    },
    MahjongFanVoiceSpec {
        fan: Fan::BigThreeDragons,
        duration: 0.960,
    },
    MahjongFanVoiceSpec {
        fan: Fan::AllGreen,
        duration: 0.960,
    },
    MahjongFanVoiceSpec {
        fan: Fan::NineGates,
        duration: 1.360,
    },
    MahjongFanVoiceSpec {
        fan: Fan::FourKongs,
        duration: 0.880,
    },
    MahjongFanVoiceSpec {
        fan: Fan::SevenShiftedPairs,
        duration: 0.880,
    },
    MahjongFanVoiceSpec {
        fan: Fan::ThirteenOrphans,
        duration: 0.960,
    },
    MahjongFanVoiceSpec {
        fan: Fan::AllTerminals,
        duration: 1.280,
    },
    MahjongFanVoiceSpec {
        fan: Fan::LittleFourWinds,
        duration: 1.120,
    },
    MahjongFanVoiceSpec {
        fan: Fan::LittleThreeDragons,
        duration: 0.960,
    },
    MahjongFanVoiceSpec {
        fan: Fan::AllHonors,
        duration: 0.880,
    },
    MahjongFanVoiceSpec {
        fan: Fan::FourConcealedPungs,
        duration: 1.120,
    },
    MahjongFanVoiceSpec {
        fan: Fan::PureTerminalChows,
        duration: 1.280,
    },
    MahjongFanVoiceSpec {
        fan: Fan::QuadrupleChow,
        duration: 1.760,
    },
    MahjongFanVoiceSpec {
        fan: Fan::FourPureShiftedPungs,
        duration: 1.200,
    },
    MahjongFanVoiceSpec {
        fan: Fan::FourPureShiftedChows,
        duration: 1.120,
    },
    MahjongFanVoiceSpec {
        fan: Fan::ThreeKongs,
        duration: 1.040,
    },
    MahjongFanVoiceSpec {
        fan: Fan::AllTerminalsAndHonors,
        duration: 1.120,
    },
    MahjongFanVoiceSpec {
        fan: Fan::SevenPairs,
        duration: 0.880,
    },
    MahjongFanVoiceSpec {
        fan: Fan::GreaterHonorsAndKnittedTiles,
        duration: 1.200,
    },
    MahjongFanVoiceSpec {
        fan: Fan::AllEvenPungs,
        duration: 1.280,
    },
    MahjongFanVoiceSpec {
        fan: Fan::FullFlush,
        duration: 1.280,
    },
    MahjongFanVoiceSpec {
        fan: Fan::PureTripleChow,
        duration: 1.680,
    },
    MahjongFanVoiceSpec {
        fan: Fan::PureShiftedPungs,
        duration: 1.520,
    },
    MahjongFanVoiceSpec {
        fan: Fan::UpperTiles,
        duration: 0.880,
    },
    MahjongFanVoiceSpec {
        fan: Fan::MiddleTiles,
        duration: 0.880,
    },
    MahjongFanVoiceSpec {
        fan: Fan::LowerTiles,
        duration: 1.120,
    },
    MahjongFanVoiceSpec {
        fan: Fan::PureStraight,
        duration: 1.200,
    },
    MahjongFanVoiceSpec {
        fan: Fan::ThreeSuitedTerminalChows,
        duration: 1.680,
    },
    MahjongFanVoiceSpec {
        fan: Fan::PureShiftedChows,
        duration: 1.280,
    },
    MahjongFanVoiceSpec {
        fan: Fan::AllFives,
        duration: 1.120,
    },
    MahjongFanVoiceSpec {
        fan: Fan::TriplePung,
        duration: 1.120,
    },
    MahjongFanVoiceSpec {
        fan: Fan::ThreeConcealedPungs,
        duration: 1.200,
    },
    MahjongFanVoiceSpec {
        fan: Fan::LesserHonorsAndKnittedTiles,
        duration: 1.040,
    },
    MahjongFanVoiceSpec {
        fan: Fan::KnittedStraight,
        duration: 1.120,
    },
    MahjongFanVoiceSpec {
        fan: Fan::UpperFour,
        duration: 1.280,
    },
    MahjongFanVoiceSpec {
        fan: Fan::LowerFour,
        duration: 1.200,
    },
    MahjongFanVoiceSpec {
        fan: Fan::BigThreeWinds,
        duration: 1.200,
    },
    MahjongFanVoiceSpec {
        fan: Fan::MixedStraight,
        duration: 0.960,
    },
    MahjongFanVoiceSpec {
        fan: Fan::ReversibleTiles,
        duration: 1.120,
    },
    MahjongFanVoiceSpec {
        fan: Fan::MixedTripleChow,
        duration: 1.600,
    },
    MahjongFanVoiceSpec {
        fan: Fan::MixedShiftedPungs,
        duration: 1.520,
    },
    MahjongFanVoiceSpec {
        fan: Fan::ChickenHand,
        duration: 0.960,
    },
    MahjongFanVoiceSpec {
        fan: Fan::LastTileDraw,
        duration: 1.360,
    },
    MahjongFanVoiceSpec {
        fan: Fan::LastTileClaim,
        duration: 1.120,
    },
    MahjongFanVoiceSpec {
        fan: Fan::OutWithReplacementTile,
        duration: 1.280,
    },
    MahjongFanVoiceSpec {
        fan: Fan::RobbingTheKong,
        duration: 1.040,
    },
    MahjongFanVoiceSpec {
        fan: Fan::TwoConcealedKongs,
        duration: 1.120,
    },
    MahjongFanVoiceSpec {
        fan: Fan::AllPungs,
        duration: 1.200,
    },
    MahjongFanVoiceSpec {
        fan: Fan::HalfFlush,
        duration: 1.280,
    },
    MahjongFanVoiceSpec {
        fan: Fan::MixedShiftedChows,
        duration: 2.000,
    },
    MahjongFanVoiceSpec {
        fan: Fan::AllTypes,
        duration: 1.280,
    },
    MahjongFanVoiceSpec {
        fan: Fan::MeldedHand,
        duration: 1.040,
    },
    MahjongFanVoiceSpec {
        fan: Fan::TwoDragonPungs,
        duration: 1.120,
    },
    MahjongFanVoiceSpec {
        fan: Fan::OutsideHand,
        duration: 1.200,
    },
    MahjongFanVoiceSpec {
        fan: Fan::FullyConcealedHand,
        duration: 1.200,
    },
    MahjongFanVoiceSpec {
        fan: Fan::TwoMeldedKongs,
        duration: 1.120,
    },
    MahjongFanVoiceSpec {
        fan: Fan::LastTile,
        duration: 0.720,
    },
    MahjongFanVoiceSpec {
        fan: Fan::DragonPung,
        duration: 0.960,
    },
    MahjongFanVoiceSpec {
        fan: Fan::PrevalentWind,
        duration: 1.440,
    },
    MahjongFanVoiceSpec {
        fan: Fan::SeatWind,
        duration: 0.880,
    },
    MahjongFanVoiceSpec {
        fan: Fan::ConcealedHand,
        duration: 0.960,
    },
    MahjongFanVoiceSpec {
        fan: Fan::AllChows,
        duration: 1.280,
    },
    MahjongFanVoiceSpec {
        fan: Fan::TileHog,
        duration: 0.960,
    },
    MahjongFanVoiceSpec {
        fan: Fan::DoublePung,
        duration: 1.280,
    },
    MahjongFanVoiceSpec {
        fan: Fan::TwoConcealedPungs,
        duration: 1.440,
    },
    MahjongFanVoiceSpec {
        fan: Fan::ConcealedKong,
        duration: 1.040,
    },
    MahjongFanVoiceSpec {
        fan: Fan::AllSimples,
        duration: 1.120,
    },
    MahjongFanVoiceSpec {
        fan: Fan::PureDoubleChow,
        duration: 1.200,
    },
    MahjongFanVoiceSpec {
        fan: Fan::MixedDoubleChow,
        duration: 1.360,
    },
    MahjongFanVoiceSpec {
        fan: Fan::ShortStraight,
        duration: 1.120,
    },
    MahjongFanVoiceSpec {
        fan: Fan::TwoTerminalChows,
        duration: 1.280,
    },
    MahjongFanVoiceSpec {
        fan: Fan::PungOfTerminalsOrHonors,
        duration: 1.360,
    },
    MahjongFanVoiceSpec {
        fan: Fan::MeldedKong,
        duration: 0.960,
    },
    MahjongFanVoiceSpec {
        fan: Fan::OneVoidedSuit,
        duration: 1.280,
    },
    MahjongFanVoiceSpec {
        fan: Fan::NoHonors,
        duration: 1.120,
    },
    MahjongFanVoiceSpec {
        fan: Fan::EdgeWait,
        duration: 1.040,
    },
    MahjongFanVoiceSpec {
        fan: Fan::ClosedWait,
        duration: 1.200,
    },
    MahjongFanVoiceSpec {
        fan: Fan::SingleWait,
        duration: 0.960,
    },
    MahjongFanVoiceSpec {
        fan: Fan::SelfDrawn,
        duration: 1.200,
    },
    MahjongFanVoiceSpec {
        fan: Fan::FlowerTiles,
        duration: 1.040,
    },
];

fn fan_voice_spec(fan: Fan) -> &'static MahjongFanVoiceSpec {
    let index = match fan {
        Fan::BigFourWinds => 0,
        Fan::BigThreeDragons => 1,
        Fan::AllGreen => 2,
        Fan::NineGates => 3,
        Fan::FourKongs => 4,
        Fan::SevenShiftedPairs => 5,
        Fan::ThirteenOrphans => 6,
        Fan::AllTerminals => 7,
        Fan::LittleFourWinds => 8,
        Fan::LittleThreeDragons => 9,
        Fan::AllHonors => 10,
        Fan::FourConcealedPungs => 11,
        Fan::PureTerminalChows => 12,
        Fan::QuadrupleChow => 13,
        Fan::FourPureShiftedPungs => 14,
        Fan::FourPureShiftedChows => 15,
        Fan::ThreeKongs => 16,
        Fan::AllTerminalsAndHonors => 17,
        Fan::SevenPairs => 18,
        Fan::GreaterHonorsAndKnittedTiles => 19,
        Fan::AllEvenPungs => 20,
        Fan::FullFlush => 21,
        Fan::PureTripleChow => 22,
        Fan::PureShiftedPungs => 23,
        Fan::UpperTiles => 24,
        Fan::MiddleTiles => 25,
        Fan::LowerTiles => 26,
        Fan::PureStraight => 27,
        Fan::ThreeSuitedTerminalChows => 28,
        Fan::PureShiftedChows => 29,
        Fan::AllFives => 30,
        Fan::TriplePung => 31,
        Fan::ThreeConcealedPungs => 32,
        Fan::LesserHonorsAndKnittedTiles => 33,
        Fan::KnittedStraight => 34,
        Fan::UpperFour => 35,
        Fan::LowerFour => 36,
        Fan::BigThreeWinds => 37,
        Fan::MixedStraight => 38,
        Fan::ReversibleTiles => 39,
        Fan::MixedTripleChow => 40,
        Fan::MixedShiftedPungs => 41,
        Fan::ChickenHand => 42,
        Fan::LastTileDraw => 43,
        Fan::LastTileClaim => 44,
        Fan::OutWithReplacementTile => 45,
        Fan::RobbingTheKong => 46,
        Fan::TwoConcealedKongs => 47,
        Fan::AllPungs => 48,
        Fan::HalfFlush => 49,
        Fan::MixedShiftedChows => 50,
        Fan::AllTypes => 51,
        Fan::MeldedHand => 52,
        Fan::TwoDragonPungs => 53,
        Fan::OutsideHand => 54,
        Fan::FullyConcealedHand => 55,
        Fan::TwoMeldedKongs => 56,
        Fan::LastTile => 57,
        Fan::DragonPung => 58,
        Fan::PrevalentWind => 59,
        Fan::SeatWind => 60,
        Fan::ConcealedHand => 61,
        Fan::AllChows => 62,
        Fan::TileHog => 63,
        Fan::DoublePung => 64,
        Fan::TwoConcealedPungs => 65,
        Fan::ConcealedKong => 66,
        Fan::AllSimples => 67,
        Fan::PureDoubleChow => 68,
        Fan::MixedDoubleChow => 69,
        Fan::ShortStraight => 70,
        Fan::TwoTerminalChows => 71,
        Fan::PungOfTerminalsOrHonors => 72,
        Fan::MeldedKong => 73,
        Fan::OneVoidedSuit => 74,
        Fan::NoHonors => 75,
        Fan::EdgeWait => 76,
        Fan::ClosedWait => 77,
        Fan::SingleWait => 78,
        Fan::SelfDrawn => 79,
        Fan::FlowerTiles => 80,
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
        next_delay += FAN_OUTCOME_LEAD;
        let hand_delay = next_delay;
        next_delay += FAN_HAND_LEAD;
        let mut fan_delays = Vec::with_capacity(winner.score.fans.len());
        for fan in &winner.score.fans {
            fan_delays.push(next_delay);
            next_delay += fan_voice_spec(fan.fan)
                .duration
                .max(SUMMARY_ROW_ENTRY_DURATION)
                + FAN_VOICE_HOLD;
        }
        let total_delay = next_delay;
        next_delay += FAN_TOTAL_HOLD + FAN_WINNER_GAP;
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
    mut commands: Commands,
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
                .find(|player| player.id == winner.player)
                .map(|player| player.game_profiles.gender)
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
            commands.spawn((
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
    commands: &mut Commands,
    playback: &mut MahjongFanVoicePlayback,
    active_voices: &Query<Entity, With<MahjongFanVoicePlayer>>,
) {
    for entity in active_voices {
        commands.entity(entity).despawn();
    }
    *playback = MahjongFanVoicePlayback::default();
}
