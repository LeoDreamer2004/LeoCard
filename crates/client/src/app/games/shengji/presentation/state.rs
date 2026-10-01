//! 双升规则事件、牌型动画和音效的统一表现层。

use crate::app::presentation::Observed;
use bevy::prelude::*;
use leocard_protocol::ShengjiBottomFlipMatchView;
use leocard_protocol::{MatchId, PlayerId, ShengjiPublicPlay};
use leocard_shengji::ShengjiCard;
use leocard_shengji::{ShengjiBidTrump, ShengjiRank};
use std::collections::VecDeque;

pub(super) const SHENGJI_TRICK_PLAY_COUNT: usize = 4;

#[derive(Clone, Debug)]
pub(super) enum ShengjiPresentationKind {
    Declaration {
        player: PlayerId,
        trump: ShengjiBidTrump,
        label: &'static str,
    },
    PowerOutage {
        from_dealer: Option<PlayerId>,
        dealer: PlayerId,
        level: ShengjiRank,
    },
    BottomFlip {
        card: ShengjiCard,
        matches: Vec<ShengjiBottomFlipMatchView>,
        dealer: Option<PlayerId>,
    },
    BottomCopy {
        from_player: Option<PlayerId>,
        player: PlayerId,
        trump: ShengjiBidTrump,
        count: u8,
    },
    CrossingStarted {
        players: Vec<PlayerId>,
    },
    CrossingReturned {
        player: PlayerId,
        complete: bool,
    },
    TrumpKill {
        player: PlayerId,
        covered: bool,
    },
    Play {
        player: PlayerId,
        kind: ShengjiPlayPresentationKind,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ShengjiPlayPresentationKind {
    Single,
    Pair,
    Tractor,
    Triple,
    Titanic,
    Bomb,
    Spaceship,
    Throw,
}

impl ShengjiPlayPresentationKind {
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Single => "单张",
            Self::Pair => "对子",
            Self::Tractor => "拖拉机",
            Self::Triple => "三同张",
            Self::Titanic => "泰坦尼克",
            Self::Bomb => "炸弹",
            Self::Spaceship => "宇宙飞船",
            Self::Throw => "甩牌",
        }
    }

    pub(super) const fn duration(self) -> f32 {
        match self {
            Self::Single => 0.45,
            Self::Pair => 0.58,
            Self::Tractor => 0.90,
            Self::Triple => 0.72,
            Self::Titanic => 1.20,
            Self::Bomb => 1.28,
            Self::Spaceship => 1.48,
            Self::Throw => 1.05,
        }
    }
}

#[derive(Clone, Debug)]
pub(super) struct ActiveShengjiPresentation {
    pub(super) kind: ShengjiPresentationKind,
    pub(super) elapsed: f32,
    pub(super) duration: f32,
}

#[derive(Resource, Default)]
pub(crate) struct ShengjiPresentationState {
    pub(super) active: Option<ActiveShengjiPresentation>,
    pub(super) queued: VecDeque<ActiveShengjiPresentation>,
    pub(super) audio_cues: Vec<ShengjiAudioCue>,
    pub(super) observed_round: Observed<MatchId, u32>,
    pub(super) bottom_copy_count: u8,
    pub(super) bottom_burier: Option<PlayerId>,
    pub(super) observed_throw_failure: Option<ObservedShengjiThrowFailure>,
    pub(super) observed_dealer: Option<PlayerId>,
    pub(super) current_trick_plays: Vec<ShengjiPublicPlay>,
    pub(super) previous_trick_plays: Vec<ShengjiPublicPlay>,
    pub(super) previous_trick_reveal_remaining: f32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct ObservedShengjiThrowFailure {
    pub(super) match_id: MatchId,
    pub(super) hand_number: u32,
    pub(super) player: PlayerId,
    pub(super) attempted: Vec<ShengjiCard>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ShengjiSoundKind {
    Confirm,
    Lock,
    PowerDown,
    PowerRelay,
    PowerUp,
    Flip,
    Copy,
    Crossing,
    CardPlace,
    CardShove,
    TrumpKillLaunch,
    TrumpKillImpact,
    Heavy,
    Bomb,
    ThrowFail,
    PenaltyFive,
    PenaltyTen,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct ShengjiAudioCue {
    pub(super) kind: ShengjiSoundKind,
    pub(super) remaining: f32,
    pub(super) volume: f32,
    pub(super) seed: u64,
}

impl ShengjiAudioCue {
    pub(super) const fn new(
        kind: ShengjiSoundKind,
        remaining: f32,
        volume: f32,
        seed: u64,
    ) -> Self {
        Self {
            kind,
            remaining,
            volume,
            seed,
        }
    }
}

#[derive(Resource)]
pub(crate) struct ShengjiSoundAssets {
    pub(super) confirm: Vec<Handle<AudioSource>>,
    pub(super) lock: Vec<Handle<AudioSource>>,
    pub(super) power_down: Vec<Handle<AudioSource>>,
    pub(super) power_relay: Vec<Handle<AudioSource>>,
    pub(super) power_up: Vec<Handle<AudioSource>>,
    pub(super) flip: Vec<Handle<AudioSource>>,
    pub(super) copy: Vec<Handle<AudioSource>>,
    pub(super) crossing: Vec<Handle<AudioSource>>,
    pub(super) place: Vec<Handle<AudioSource>>,
    pub(super) shove: Vec<Handle<AudioSource>>,
    pub(super) trump_kill_launch: Vec<Handle<AudioSource>>,
    pub(super) trump_kill_impact: Vec<Handle<AudioSource>>,
    pub(super) heavy: Vec<Handle<AudioSource>>,
    pub(super) bomb: Vec<Handle<AudioSource>>,
    pub(super) throw_fail: Vec<Handle<AudioSource>>,
    pub(super) penalty_five: Vec<Handle<AudioSource>>,
    pub(super) penalty_ten: Vec<Handle<AudioSource>>,
}

impl ShengjiSoundAssets {
    pub(crate) fn load(asset_server: &AssetServer) -> Self {
        let interface = "vendor/kenney/interface-sounds/Audio";
        let casino = "vendor/kenney/casino-audio/Audio";
        Self {
            confirm: vec![
                asset_server.load(format!("{interface}/confirmation_003.ogg")),
                asset_server.load(format!("{interface}/confirmation_004.ogg")),
            ],
            lock: vec![asset_server.load(format!("{interface}/toggle_003.ogg"))],
            power_down: vec![asset_server.load("audio/shengji/power-off.ogg")],
            power_relay: vec![
                asset_server.load(format!("{interface}/switch_003.ogg")),
                asset_server.load(format!("{interface}/switch_004.ogg")),
            ],
            power_up: vec![asset_server.load("audio/shengji/power-on.ogg")],
            flip: (1..=4)
                .map(|index| asset_server.load(format!("{casino}/card-place-{index}.ogg")))
                .collect(),
            copy: vec![
                asset_server.load(format!("{casino}/cards-pack-take-out-1.ogg")),
                asset_server.load(format!("{casino}/cards-pack-take-out-2.ogg")),
            ],
            crossing: (1..=4)
                .map(|index| asset_server.load(format!("{casino}/card-shove-{index}.ogg")))
                .collect(),
            place: (1..=4)
                .map(|index| asset_server.load(format!("{casino}/card-place-{index}.ogg")))
                .collect(),
            shove: (1..=4)
                .map(|index| asset_server.load(format!("{casino}/card-shove-{index}.ogg")))
                .collect(),
            trump_kill_launch: vec![asset_server.load(format!("{interface}/pluck_002.ogg"))],
            trump_kill_impact: vec![asset_server.load(format!("{interface}/drop_004.ogg"))],
            heavy: vec![
                asset_server.load(format!("{interface}/bong_001.ogg")),
                asset_server.load(format!("{interface}/drop_004.ogg")),
            ],
            bomb: vec![asset_server.load("vendor/noname/damage_fire2.mp3")],
            throw_fail: vec![asset_server.load(format!("{interface}/scratch_004.ogg"))],
            penalty_five: vec![asset_server.load(format!("{casino}/chip-lay-2.ogg"))],
            penalty_ten: vec![asset_server.load(format!("{casino}/chips-collide-4.ogg"))],
        }
    }

    pub(super) fn variants(&self, kind: ShengjiSoundKind) -> &[Handle<AudioSource>] {
        match kind {
            ShengjiSoundKind::Confirm => &self.confirm,
            ShengjiSoundKind::Lock => &self.lock,
            ShengjiSoundKind::PowerDown => &self.power_down,
            ShengjiSoundKind::PowerRelay => &self.power_relay,
            ShengjiSoundKind::PowerUp => &self.power_up,
            ShengjiSoundKind::Flip => &self.flip,
            ShengjiSoundKind::Copy => &self.copy,
            ShengjiSoundKind::Crossing => &self.crossing,
            ShengjiSoundKind::CardPlace => &self.place,
            ShengjiSoundKind::CardShove => &self.shove,
            ShengjiSoundKind::TrumpKillLaunch => &self.trump_kill_launch,
            ShengjiSoundKind::TrumpKillImpact => &self.trump_kill_impact,
            ShengjiSoundKind::Heavy => &self.heavy,
            ShengjiSoundKind::Bomb => &self.bomb,
            ShengjiSoundKind::ThrowFail => &self.throw_fail,
            ShengjiSoundKind::PenaltyFive => &self.penalty_five,
            ShengjiSoundKind::PenaltyTen => &self.penalty_ten,
        }
    }
}

#[derive(Component)]
pub(crate) struct ShengjiPresentationRoot {
    pub(super) base_translation: Vec2,
}

#[derive(Component)]
pub(crate) struct ShengjiPresentationVeil;

#[derive(Component)]
pub(crate) struct ShengjiPresentationText {
    pub(super) base_color: Color,
}

#[derive(Component)]
pub(crate) struct ShengjiPresentationDivider;

#[derive(Component)]
pub(crate) struct ShengjiPresentationPacket {
    pub(super) index: usize,
    pub(super) count: usize,
    pub(super) route_index: usize,
    pub(super) start: Vec2,
    pub(super) end: Vec2,
}

#[derive(Component)]
pub(crate) struct ShengjiPowerOutageVisual {
    pub(super) kind: ShengjiPowerOutageVisualKind,
}

#[derive(Clone, Copy, Debug)]
pub(super) enum ShengjiPowerOutageVisualKind {
    Spark {
        index: usize,
        count: usize,
        start: Vec2,
        end: Vec2,
    },
    TargetRing {
        target: Vec2,
    },
    LevelFlash {
        target: Vec2,
    },
}

#[derive(Component)]
pub(crate) struct ShengjiTrumpKillVisual {
    pub(super) kind: ShengjiTrumpKillVisualKind,
}

#[derive(Clone, Copy, Debug)]
pub(super) enum ShengjiTrumpKillVisualKind {
    Target,
    Dart,
    ImpactRing,
}

#[derive(Component)]
pub(crate) struct ShengjiBottomFlipVisual {
    pub(super) kind: ShengjiBottomFlipVisualKind,
}

#[derive(Clone, Copy, Debug)]
pub(super) enum ShengjiBottomFlipVisualKind {
    ScanSpark {
        player_index: usize,
        player_count: usize,
        spark_index: usize,
        spark_count: usize,
        start: Vec2,
        end: Vec2,
    },
    ReplySpark {
        player_index: usize,
        player_count: usize,
        spark_index: usize,
        spark_count: usize,
        start: Vec2,
        end: Vec2,
    },
}

/// 扣底面板中需要按节奏出现的元素。面板本身由 `view` 构建，
/// 这个标记让表现层可以在不重建 UI 的情况下动画它们。
#[derive(Component)]
pub(crate) enum ShengjiBottomFlipPanelElement {
    CentralCard,
    MatchRow {
        player: PlayerId,
        index: usize,
        count: usize,
    },
    DealerLine,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct ShengjiPresentationRoute {
    pub(super) start: Vec2,
    pub(super) end: Vec2,
    pub(super) packet_count: usize,
}
