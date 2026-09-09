//! 麻将牌张、吃碰杠、补花、和牌与推牌演出。

mod claim;
mod tiles;
mod win_animation;
mod win_view;

use super::{
    ActiveMahjongClaimPresentation, MAHJONG_CLAIM_FLIGHT_DELAY, MAHJONG_CLAIM_FLIGHT_DURATION,
    MAHJONG_CLAIM_HAND_SHIFT_DURATION, MAHJONG_CLAIM_PRESENTATION_DURATION,
    MAHJONG_FLOWER_PRESENTATION_DURATION, MAHJONG_OWN_HAND_LEFT, MAHJONG_OWN_MELD_WIDTH,
    MAHJONG_REMOTE_MELD_WIDTH, MahjongAssets, MahjongClaimFlight, MahjongClaimHandShift,
    MahjongClaimHeldTile, MahjongClaimLabel, MahjongClaimPresentationState, MahjongDealSpec,
    MahjongDealTile, MahjongFlowerLabel, MahjongTileMaterial, MahjongWinDecoration,
    MahjongWinDecorationKind, MahjongWinEffect, MahjongWinEffectText, MahjongWinEffectTier,
    MahjongWinFanGlyph, MahjongWinScreenShake, MahjongWinStageKind, MahjongWinStagePart,
    MahjongWinningHand, mahjong_claim_landing_time, mahjong_local_light, mahjong_local_shadow,
    mahjong_win_effect_tier, mahjong_win_reveal_duration, mahjong_win_stage_start,
};
pub(crate) use claim::*;
pub(crate) use tiles::*;
pub(crate) use win_animation::*;
pub(crate) use win_view::*;

const MAHJONG_WIN_PUSH_DURATION: f32 = 0.42;
