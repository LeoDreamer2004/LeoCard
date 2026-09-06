//! 双升规则事件、牌型动画和音效的统一表现层。

mod animation;
mod content;
mod overlay;
mod routes;
mod sync;
#[cfg(test)]
#[path = "presentation_tests.rs"]
mod tests;

use super::*;
pub use animation::*;
use content::*;
use leocard_protocol::ShengjiBottomFlipMatchView;
use leocard_protocol::{MatchId, PlayerId, ShengjiPublicPlay};
use leocard_shengji::ShengjiCard;
use leocard_shengji::{ShengjiBidTrump, ShengjiRank};
pub use overlay::*;
use routes::*;
use std::collections::VecDeque;
pub use sync::*;

const SHENGJI_TRICK_PLAY_COUNT: usize = 4;

#[derive(Clone, Debug)]
enum ShengjiPresentationKind {
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
enum ShengjiPlayPresentationKind {
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
    pub const fn label(self) -> &'static str {
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

    const fn duration(self) -> f32 {
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
struct ActiveShengjiPresentation {
    kind: ShengjiPresentationKind,
    elapsed: f32,
    duration: f32,
}

#[derive(Resource, Default)]
pub struct ShengjiPresentationState {
    active: Option<ActiveShengjiPresentation>,
    queued: VecDeque<ActiveShengjiPresentation>,
    audio_cues: Vec<ShengjiAudioCue>,
    observed_match: Option<MatchId>,
    observed_hand: u32,
    bottom_copy_count: u8,
    bottom_burier: Option<PlayerId>,
    observed_throw_failure: Option<ObservedShengjiThrowFailure>,
    observed_dealer: Option<PlayerId>,
    current_trick_plays: Vec<ShengjiPublicPlay>,
    previous_trick_plays: Vec<ShengjiPublicPlay>,
    previous_trick_reveal_remaining: f32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ObservedShengjiThrowFailure {
    match_id: MatchId,
    hand_number: u32,
    player: PlayerId,
    attempted: Vec<ShengjiCard>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ShengjiSoundKind {
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
struct ShengjiAudioCue {
    kind: ShengjiSoundKind,
    remaining: f32,
    volume: f32,
    seed: u64,
}

impl ShengjiAudioCue {
    const fn new(kind: ShengjiSoundKind, remaining: f32, volume: f32, seed: u64) -> Self {
        Self {
            kind,
            remaining,
            volume,
            seed,
        }
    }
}

#[derive(Resource)]
pub struct ShengjiSoundAssets {
    confirm: Vec<Handle<AudioSource>>,
    lock: Vec<Handle<AudioSource>>,
    power_down: Vec<Handle<AudioSource>>,
    power_relay: Vec<Handle<AudioSource>>,
    power_up: Vec<Handle<AudioSource>>,
    flip: Vec<Handle<AudioSource>>,
    copy: Vec<Handle<AudioSource>>,
    crossing: Vec<Handle<AudioSource>>,
    place: Vec<Handle<AudioSource>>,
    shove: Vec<Handle<AudioSource>>,
    trump_kill_launch: Vec<Handle<AudioSource>>,
    trump_kill_impact: Vec<Handle<AudioSource>>,
    heavy: Vec<Handle<AudioSource>>,
    bomb: Vec<Handle<AudioSource>>,
    throw_fail: Vec<Handle<AudioSource>>,
    penalty_five: Vec<Handle<AudioSource>>,
    penalty_ten: Vec<Handle<AudioSource>>,
}

impl ShengjiSoundAssets {
    pub fn load(asset_server: &AssetServer) -> Self {
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

    fn variants(&self, kind: ShengjiSoundKind) -> &[Handle<AudioSource>] {
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
pub struct ShengjiPresentationRoot {
    base_translation: Vec2,
}

#[derive(Component)]
pub struct ShengjiPresentationVeil;

#[derive(Component)]
pub struct ShengjiPresentationText {
    base_color: Color,
}

#[derive(Component)]
pub struct ShengjiPresentationDivider;

#[derive(Component)]
pub struct ShengjiPresentationPacket {
    index: usize,
    count: usize,
    route_index: usize,
    start: Vec2,
    end: Vec2,
}

#[derive(Component)]
pub struct ShengjiPowerOutageVisual {
    kind: ShengjiPowerOutageVisualKind,
}

#[derive(Clone, Copy, Debug)]
enum ShengjiPowerOutageVisualKind {
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
pub struct ShengjiTrumpKillVisual {
    kind: ShengjiTrumpKillVisualKind,
}

#[derive(Clone, Copy, Debug)]
enum ShengjiTrumpKillVisualKind {
    Target,
    Dart,
    ImpactRing,
}

#[derive(Component)]
pub struct ShengjiBottomFlipVisual {
    kind: ShengjiBottomFlipVisualKind,
}

#[derive(Clone, Copy, Debug)]
enum ShengjiBottomFlipVisualKind {
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
pub enum ShengjiBottomFlipPanelElement {
    CentralCard,
    MatchRow {
        player: PlayerId,
        index: usize,
        count: usize,
    },
    DealerLine,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct ShengjiPresentationRoute {
    start: Vec2,
    end: Vec2,
    packet_count: usize,
}
