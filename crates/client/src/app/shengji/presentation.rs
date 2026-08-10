//! 双升规则事件、牌型动画和音效的统一表现层。

use super::*;
use std::collections::VecDeque;

const SHENGJI_TRICK_PLAY_COUNT: usize = 4;

#[derive(Clone, Debug)]
pub(in crate::app) enum ShengjiPresentationKind {
    Declaration {
        player: PlayerId,
        trump: leocard_shengji::BidTrump,
        label: &'static str,
    },
    PowerOutage {
        from_dealer: Option<PlayerId>,
        dealer: PlayerId,
        level: leocard_shengji::Rank,
    },
    BottomFlip {
        card: ShengjiCard,
        matches: Vec<leocard_protocol::ShengjiBottomFlipMatchView>,
        dealer: Option<PlayerId>,
    },
    BottomCopy {
        from_player: Option<PlayerId>,
        player: PlayerId,
        trump: leocard_shengji::BidTrump,
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
pub(in crate::app) enum ShengjiPlayPresentationKind {
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
    pub(in crate::app) const fn label(self) -> &'static str {
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
pub(in crate::app) struct ActiveShengjiPresentation {
    pub(in crate::app) kind: ShengjiPresentationKind,
    pub(in crate::app) elapsed: f32,
    pub(in crate::app) duration: f32,
}

#[derive(Resource, Default)]
pub(in crate::app) struct ShengjiPresentationState {
    pub(in crate::app) active: Option<ActiveShengjiPresentation>,
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
pub(in crate::app) struct ShengjiSoundAssets {
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
    pub(in crate::app) fn load(asset_server: &AssetServer) -> Self {
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
pub(in crate::app) struct ShengjiPresentationRoot {
    base_translation: Vec2,
}

#[derive(Component)]
pub(in crate::app) struct ShengjiPresentationVeil;

#[derive(Component)]
pub(in crate::app) struct ShengjiPresentationText {
    base_color: Color,
}

#[derive(Component)]
pub(in crate::app) struct ShengjiPresentationDivider;

#[derive(Component)]
pub(in crate::app) struct ShengjiPresentationPacket {
    index: usize,
    count: usize,
    route_index: usize,
    start: Vec2,
    end: Vec2,
}

#[derive(Component)]
pub(in crate::app) struct ShengjiPowerOutageVisual {
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
pub(in crate::app) struct ShengjiTrumpKillVisual {
    kind: ShengjiTrumpKillVisualKind,
}

#[derive(Clone, Copy, Debug)]
enum ShengjiTrumpKillVisualKind {
    Target,
    Dart,
    ImpactRing,
}

#[derive(Component)]
pub(in crate::app) struct ShengjiBottomFlipVisual {
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
pub(in crate::app) enum ShengjiBottomFlipPanelElement {
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

pub(in crate::app) fn sync_shengji_presentation(
    mut client: Option<ResMut<ClientResource>>,
    mut state: ResMut<ShengjiPresentationState>,
    mut ui: ResMut<UiState>,
) {
    let Some(client) = client.as_deref_mut() else {
        if state.active.take().is_some() {
            ui.dirty = true;
        }
        state.observed_match = None;
        state.queued.clear();
        state.audio_cues.clear();
        state.bottom_burier = None;
        state.observed_throw_failure = None;
        state.observed_dealer = None;
        state.clear_trick_history();
        return;
    };
    let game_state = client.0.model().shengji_game().map(|game| {
        (
            game.match_id,
            game.hand_number,
            game.throw_failure.clone(),
            game.rules.throw_penalty,
            game.dealer,
            game.trump,
        )
    });
    let Some((match_id, hand_number, throw_failure, throw_penalty, current_dealer, current_trump)) =
        game_state
    else {
        if state.active.take().is_some() {
            ui.dirty = true;
        }
        state.observed_match = None;
        state.queued.clear();
        state.audio_cues.clear();
        state.bottom_burier = None;
        state.observed_throw_failure = None;
        state.observed_dealer = None;
        state.clear_trick_history();
        client.0.take_shengji_events();
        return;
    };
    let match_changed = state.observed_match != Some(match_id);
    if match_changed || state.observed_hand != hand_number {
        if match_changed {
            state.observed_dealer = None;
        }
        state.observed_match = Some(match_id);
        state.observed_hand = hand_number;
        state.bottom_copy_count = 0;
        state.bottom_burier = None;
        state.observed_throw_failure = None;
        state.active = None;
        state.queued.clear();
        state.audio_cues.clear();
        state.clear_trick_history();
    }

    let observed_throw_failure =
        throw_failure
            .as_ref()
            .map(|failure| ObservedShengjiThrowFailure {
                match_id,
                hand_number,
                player: failure.player,
                attempted: failure.attempted.clone(),
            });
    if state.observed_throw_failure != observed_throw_failure {
        if throw_failure
            .as_ref()
            .is_some_and(|failure| failure.stage == ShengjiThrowFailureStage::Showing)
        {
            queue_throw_failure_audio(&mut state.audio_cues, throw_penalty);
        }
        state.observed_throw_failure = observed_throw_failure;
    }

    let events = client.0.take_shengji_events();
    if events.is_empty() {
        state.observed_dealer = current_dealer;
        return;
    }
    for event in events {
        let trump_kill = match &event {
            leocard_protocol::ShengjiEvent::CardsPlayed { play, is_lead } => {
                state.winning_trump_kill(play, *is_lead, current_trump)
            }
            _ => None,
        };
        state.observe_trick_event(&event);
        match event {
            leocard_protocol::ShengjiEvent::DeclarationChanged { declaration } => {
                let label = match declaration.kind {
                    leocard_shengji::BidKind::Initial => "亮主",
                    leocard_shengji::BidKind::Protect => "自保",
                    leocard_shengji::BidKind::Counter => "反主",
                    leocard_shengji::BidKind::SelfCounter => "自反",
                };
                let start = state.activate(
                    ShengjiPresentationKind::Declaration {
                        player: declaration.player,
                        trump: declaration.trump,
                        label,
                    },
                    1.65,
                );
                state.audio_cues.push(ShengjiAudioCue::new(
                    ShengjiSoundKind::Confirm,
                    start,
                    0.28,
                    3,
                ));
            }
            leocard_protocol::ShengjiEvent::BiddingLocked { .. } => {
                state
                    .audio_cues
                    .push(ShengjiAudioCue::new(ShengjiSoundKind::Lock, 0.0, 0.32, 5));
            }
            leocard_protocol::ShengjiEvent::PowerOutageDealerChanged { dealer, level } => {
                let from_dealer = state.observed_dealer;
                let start = state.activate(
                    ShengjiPresentationKind::PowerOutage {
                        from_dealer,
                        dealer,
                        level,
                    },
                    2.0,
                );
                state.audio_cues.extend([
                    ShengjiAudioCue::new(ShengjiSoundKind::PowerDown, start, 0.52, 7),
                    ShengjiAudioCue::new(ShengjiSoundKind::PowerRelay, start + 0.34, 0.25, 9),
                    ShengjiAudioCue::new(ShengjiSoundKind::PowerRelay, start + 0.49, 0.29, 10),
                    ShengjiAudioCue::new(ShengjiSoundKind::PowerRelay, start + 0.64, 0.34, 12),
                    ShengjiAudioCue::new(ShengjiSoundKind::PowerUp, start + 0.78, 0.48, 11),
                ]);
            }
            leocard_protocol::ShengjiEvent::BottomCardRevealed { reveal } => {
                let matches = reveal.matches.clone();
                let dealer = reveal.dealer;
                let start = state.activate(
                    ShengjiPresentationKind::BottomFlip {
                        card: reveal.card,
                        matches: matches.clone(),
                        dealer,
                    },
                    2.65,
                );
                state.audio_cues.push(ShengjiAudioCue::new(
                    ShengjiSoundKind::Flip,
                    start,
                    0.46,
                    13,
                ));
                if !matches.is_empty() {
                    state.audio_cues.push(ShengjiAudioCue::new(
                        ShengjiSoundKind::Confirm,
                        start + 0.82,
                        0.25,
                        14,
                    ));
                }
                if dealer.is_some() {
                    state.audio_cues.push(ShengjiAudioCue::new(
                        ShengjiSoundKind::Lock,
                        start + 1.72,
                        0.31,
                        15,
                    ));
                }
            }
            leocard_protocol::ShengjiEvent::BottomCopied { declaration } => {
                state.bottom_copy_count = state.bottom_copy_count.saturating_add(1);
                let count = state.bottom_copy_count;
                let from_player = state.bottom_burier;
                let start = state.activate(
                    ShengjiPresentationKind::BottomCopy {
                        from_player,
                        player: declaration.player,
                        trump: declaration.trump,
                        count,
                    },
                    1.75,
                );
                state.audio_cues.extend([
                    ShengjiAudioCue::new(ShengjiSoundKind::Copy, start, 0.38, 17),
                    ShengjiAudioCue::new(ShengjiSoundKind::Lock, start + 0.72, 0.26, 19),
                ]);
            }
            leocard_protocol::ShengjiEvent::FiveTrumpCrossingStarted { players } => {
                let start =
                    state.activate(ShengjiPresentationKind::CrossingStarted { players }, 1.85);
                state.audio_cues.push(ShengjiAudioCue::new(
                    ShengjiSoundKind::Crossing,
                    start + 0.04,
                    0.48,
                    23,
                ));
            }
            leocard_protocol::ShengjiEvent::FiveTrumpCrossingReturned { player, complete } => {
                let start = state.activate(
                    ShengjiPresentationKind::CrossingReturned { player, complete },
                    if complete { 1.55 } else { 1.35 },
                );
                state.audio_cues.push(ShengjiAudioCue::new(
                    ShengjiSoundKind::Crossing,
                    start,
                    0.34,
                    29,
                ));
            }
            leocard_protocol::ShengjiEvent::CardsBuried { dealer } => {
                state.bottom_burier = Some(dealer);
            }
            leocard_protocol::ShengjiEvent::CardsPlayed { play, is_lead } => {
                let kind = classify_play_presentation(&play.play);
                let start = if let Some(covered) = trump_kill {
                    let start = state.activate(
                        ShengjiPresentationKind::TrumpKill {
                            player: play.player,
                            covered,
                        },
                        1.12,
                    );
                    state.audio_cues.extend([
                        ShengjiAudioCue::new(
                            ShengjiSoundKind::TrumpKillLaunch,
                            start + 0.10,
                            0.34,
                            play.player.0 as u64 + 61,
                        ),
                        ShengjiAudioCue::new(
                            ShengjiSoundKind::TrumpKillImpact,
                            start + 0.52,
                            0.46,
                            play.player.0 as u64 + 67,
                        ),
                    ]);
                    start
                } else if should_show_play_presentation(kind, is_lead, play.throw_penalty) {
                    state.activate(
                        ShengjiPresentationKind::Play {
                            player: play.player,
                            kind,
                        },
                        kind.duration(),
                    )
                } else {
                    0.0
                };
                queue_play_audio(&mut state.audio_cues, kind, play.player.0 as u64, start);
            }
            _ => {}
        }
    }
    state.observed_dealer = current_dealer;
    ui.dirty = true;
}

fn queue_throw_failure_audio(cues: &mut Vec<ShengjiAudioCue>, penalty: ShengjiThrowPenalty) {
    cues.extend([
        ShengjiAudioCue::new(ShengjiSoundKind::ThrowFail, 0.18, 0.44, 37),
        ShengjiAudioCue::new(ShengjiSoundKind::CardShove, 1.08, 0.38, 41),
    ]);
    match penalty {
        ShengjiThrowPenalty::None => {}
        ShengjiThrowPenalty::FivePerCard => cues.push(ShengjiAudioCue::new(
            ShengjiSoundKind::PenaltyFive,
            0.42,
            0.40,
            43,
        )),
        ShengjiThrowPenalty::TenPerCard => cues.extend([
            ShengjiAudioCue::new(ShengjiSoundKind::PenaltyTen, 0.42, 0.48, 47),
            ShengjiAudioCue::new(ShengjiSoundKind::Heavy, 0.49, 0.24, 53),
        ]),
    }
}

impl ShengjiPresentationState {
    fn winning_trump_kill(
        &self,
        challenger: &ShengjiPublicPlay,
        is_lead: bool,
        trump: Option<ShengjiTrump>,
    ) -> Option<bool> {
        let trump = trump?;
        let lead = &self.current_trick_plays.first()?.play;
        if is_lead
            || !matches!(lead.category, leocard_shengji::Category::Suit(_))
            || challenger.play.category != leocard_shengji::Category::Trump
        {
            return None;
        }
        let mut winner = lead;
        for previous in self.current_trick_plays.iter().skip(1) {
            if leocard_shengji::compare_for_trick(lead, winner, &previous.play, trump).is_gt() {
                winner = &previous.play;
            }
        }
        leocard_shengji::compare_for_trick(lead, winner, &challenger.play, trump)
            .is_gt()
            .then_some(winner.category == leocard_shengji::Category::Trump)
    }

    pub(in crate::app) fn has_previous_trick(&self) -> bool {
        self.previous_trick_plays.len() == SHENGJI_TRICK_PLAY_COUNT
    }

    pub(in crate::app) fn revealed_previous_trick(&self) -> Option<&[ShengjiPublicPlay]> {
        (self.previous_trick_reveal_remaining > 0.0 && self.has_previous_trick())
            .then_some(self.previous_trick_plays.as_slice())
    }

    pub(in crate::app) fn reveal_previous_trick(&mut self) {
        if self.has_previous_trick() {
            self.previous_trick_reveal_remaining = 2.0;
        }
    }

    fn clear_trick_history(&mut self) {
        self.current_trick_plays.clear();
        self.previous_trick_plays.clear();
        self.previous_trick_reveal_remaining = 0.0;
    }

    fn observe_trick_event(&mut self, event: &leocard_protocol::ShengjiEvent) {
        match event {
            leocard_protocol::ShengjiEvent::CardsPlayed { play, is_lead } => {
                if *is_lead {
                    self.current_trick_plays.clear();
                }
                if let Some(existing) = self
                    .current_trick_plays
                    .iter_mut()
                    .find(|existing| existing.player == play.player)
                {
                    *existing = play.clone();
                } else {
                    self.current_trick_plays.push(play.clone());
                }
            }
            leocard_protocol::ShengjiEvent::TrickFinished { .. } => {
                if self.current_trick_plays.len() == SHENGJI_TRICK_PLAY_COUNT {
                    self.previous_trick_plays = std::mem::take(&mut self.current_trick_plays);
                } else {
                    self.current_trick_plays.clear();
                }
                self.previous_trick_reveal_remaining = 0.0;
            }
            _ => {}
        }
    }

    fn activate(&mut self, kind: ShengjiPresentationKind, duration: f32) -> f32 {
        let start_delay = self
            .active
            .as_ref()
            .map(|active| (active.duration - active.elapsed).max(0.0))
            .unwrap_or(0.0)
            + self
                .queued
                .iter()
                .map(|presentation| presentation.duration)
                .sum::<f32>();
        let presentation = ActiveShengjiPresentation {
            kind,
            elapsed: 0.0,
            duration,
        };
        if self.active.is_none() {
            self.active = Some(presentation);
        } else {
            self.queued.push_back(presentation);
        }
        start_delay
    }
}

pub(in crate::app) fn advance_shengji_presentation(
    time: Res<Time>,
    mut state: ResMut<ShengjiPresentationState>,
    mut ui: ResMut<UiState>,
) {
    let delta = time.delta_secs();
    if state.previous_trick_reveal_remaining > 0.0 {
        state.previous_trick_reveal_remaining =
            (state.previous_trick_reveal_remaining - delta).max(0.0);
        if state.previous_trick_reveal_remaining == 0.0 {
            ui.dirty = true;
        }
    }
    let Some(active) = state.active.as_mut() else {
        return;
    };
    active.elapsed += delta;
    if active.elapsed >= active.duration {
        state.active = state.queued.pop_front();
        ui.dirty = true;
    }
}

pub(in crate::app) fn play_shengji_audio_cues(
    time: Res<Time>,
    assets: Res<ShengjiSoundAssets>,
    mut state: ResMut<ShengjiPresentationState>,
    mut commands: Commands,
) {
    let mut waiting = Vec::with_capacity(state.audio_cues.len());
    let mut ready = Vec::new();
    for mut cue in std::mem::take(&mut state.audio_cues) {
        cue.remaining -= time.delta_secs();
        if cue.remaining <= 0.0 {
            ready.push(cue);
        } else {
            waiting.push(cue);
        }
    }
    state.audio_cues = waiting;
    for cue in ready {
        let variants = assets.variants(cue.kind);
        if variants.is_empty() {
            continue;
        }
        commands.spawn((
            AudioPlayer::new(variants[cue.seed as usize % variants.len()].clone()),
            PlaybackSettings {
                volume: Volume::Linear(cue.volume),
                ..PlaybackSettings::DESPAWN
            },
        ));
    }
}

pub(in crate::app) fn animate_shengji_presentation(
    state: Res<ShengjiPresentationState>,
    mut roots: Query<
        (&ShengjiPresentationRoot, &mut UiTransform, &mut Visibility),
        (
            With<ShengjiPresentationRoot>,
            Without<ShengjiPresentationVeil>,
            Without<ShengjiPresentationDivider>,
            Without<ShengjiPowerOutageVisual>,
            Without<ShengjiTrumpKillVisual>,
            Without<ShengjiBottomFlipVisual>,
        ),
    >,
    mut veils: Query<
        &mut BackgroundColor,
        (
            With<ShengjiPresentationVeil>,
            Without<ShengjiPowerOutageVisual>,
            Without<ShengjiTrumpKillVisual>,
            Without<ShengjiBottomFlipVisual>,
        ),
    >,
    mut texts: Query<(&ShengjiPresentationText, &mut TextColor)>,
    mut dividers: Query<
        (&mut UiTransform, &mut BackgroundColor),
        (
            Without<ShengjiPresentationRoot>,
            Without<ShengjiPresentationVeil>,
            Without<ShengjiPresentationPacket>,
            Without<ShengjiPowerOutageVisual>,
            Without<ShengjiTrumpKillVisual>,
            Without<ShengjiBottomFlipVisual>,
            With<ShengjiPresentationDivider>,
        ),
    >,
    mut packets: Query<
        (
            &ShengjiPresentationPacket,
            &mut Node,
            &mut UiTransform,
            &mut ImageNode,
        ),
        (
            Without<ShengjiPresentationRoot>,
            Without<ShengjiPresentationDivider>,
            Without<ShengjiPowerOutageVisual>,
            Without<ShengjiTrumpKillVisual>,
            Without<ShengjiBottomFlipVisual>,
        ),
    >,
    mut power_visuals: Query<
        (
            &ShengjiPowerOutageVisual,
            &mut Node,
            &mut UiTransform,
            &mut BackgroundColor,
            Option<&mut BorderColor>,
            &mut Visibility,
        ),
        (
            Without<ShengjiPresentationRoot>,
            Without<ShengjiPresentationVeil>,
            Without<ShengjiPresentationDivider>,
            Without<ShengjiPresentationPacket>,
            Without<ShengjiTrumpKillVisual>,
            Without<ShengjiBottomFlipVisual>,
        ),
    >,
    mut trump_kill_visuals: Query<
        (
            &ShengjiTrumpKillVisual,
            &mut Node,
            &mut UiTransform,
            Option<&mut ImageNode>,
            &mut BackgroundColor,
            Option<&mut BorderColor>,
            &mut Visibility,
        ),
        (
            Without<ShengjiPresentationRoot>,
            Without<ShengjiPresentationVeil>,
            Without<ShengjiPresentationDivider>,
            Without<ShengjiPresentationPacket>,
            Without<ShengjiPowerOutageVisual>,
            Without<ShengjiBottomFlipVisual>,
        ),
    >,
    mut bottom_flip_visuals: Query<
        (
            &ShengjiBottomFlipVisual,
            &mut Node,
            &mut UiTransform,
            &mut BackgroundColor,
            &mut Visibility,
        ),
        (
            Without<ShengjiPresentationRoot>,
            Without<ShengjiPresentationVeil>,
            Without<ShengjiPresentationDivider>,
            Without<ShengjiPresentationPacket>,
            Without<ShengjiPowerOutageVisual>,
            Without<ShengjiTrumpKillVisual>,
        ),
    >,
) {
    let Some(active) = state.active.as_ref() else {
        for (_, _, mut visibility) in &mut roots {
            *visibility = Visibility::Hidden;
        }
        return;
    };
    let progress = (active.elapsed / active.duration).clamp(0.0, 1.0);
    let enter = ease_out_cubic((progress / 0.16).clamp(0.0, 1.0));
    let exit = ease_out_cubic(((progress - 0.82) / 0.18).clamp(0.0, 1.0));
    let alpha = enter * (1.0 - exit);
    for (root, mut transform, mut visibility) in &mut roots {
        *visibility = Visibility::Visible;
        transform.scale = Vec2::splat(0.98 + 0.02 * enter);
        transform.translation = Val2::px(
            root.base_translation.x,
            root.base_translation.y + 6.0 * (1.0 - enter) - 3.0 * exit,
        );
    }
    let outage = matches!(active.kind, ShengjiPresentationKind::PowerOutage { .. });
    for mut background in &mut veils {
        let outage_alpha = if !outage {
            0.0
        } else if progress < 0.10 {
            ease_out_cubic(progress / 0.10)
        } else if progress < 0.55 {
            1.0
        } else {
            1.0 - ease_out_cubic(((progress - 0.55) / 0.18).clamp(0.0, 1.0))
        };
        background.0 = Color::BLACK.with_alpha(0.46 * outage_alpha);
    }
    for (text, mut color) in &mut texts {
        color.0 = text.base_color.with_alpha(alpha);
    }
    for (mut transform, mut background) in &mut dividers {
        transform.scale = Vec2::new(0.72 + 0.28 * enter, 1.0);
        background.0 = presentation_color(&active.kind).with_alpha(0.72 * alpha);
    }
    for (packet, mut node, mut transform, mut image) in &mut packets {
        let spread = packet.index as f32 - (packet.count.saturating_sub(1) as f32 * 0.5);
        let stagger = packet.index as f32 / packet.count.max(1) as f32 * 0.08
            + packet.route_index as f32 * 0.015;
        let packet_progress = ((progress - 0.06 - stagger) / 0.68).clamp(0.0, 1.0);
        let travel = ease_out_cubic(packet_progress);
        let position = packet.start.lerp(packet.end, travel);
        node.left = percent(position.x);
        node.top = percent(position.y);

        let direction = (packet.end - packet.start).normalize_or_zero();
        let perpendicular = Vec2::new(-direction.y, direction.x);
        let arc = (packet_progress * std::f32::consts::PI).sin();
        let offset = perpendicular * (spread * 7.0 + arc * (24.0 + packet.index as f32));
        transform.translation = Val2::px(-17.0 + offset.x, -24.0 + offset.y);
        transform.rotation = Rot2::radians((spread * 1.2_f32).to_radians());
        transform.scale = Vec2::splat(0.92 + arc * 0.08);
        image.color = Color::WHITE.with_alpha(alpha);
    }
    for (visual, mut node, mut transform, mut background, border, mut visibility) in
        &mut power_visuals
    {
        match visual.kind {
            ShengjiPowerOutageVisualKind::Spark {
                index,
                count,
                start,
                end,
            } => {
                let stagger = index as f32 / count.max(1) as f32 * 0.12;
                let local = ((progress - 0.15 - stagger) / 0.27).clamp(0.0, 1.0);
                let alive = progress >= 0.15 + stagger && local < 1.0;
                *visibility = if alive {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                };
                let travel = ease_out_cubic(local);
                let direction = (end - start).normalize_or_zero();
                let perpendicular = Vec2::new(-direction.y, direction.x);
                let polarity = if index % 2 == 0 { 1.0 } else { -1.0 };
                let position = start.lerp(end, travel)
                    + perpendicular * ((local * std::f32::consts::PI).sin() * polarity * 1.8);
                node.left = percent(position.x);
                node.top = percent(position.y);
                transform.translation = Val2::px(-4.0, -4.0);
                transform.scale =
                    Vec2::splat(0.72 + (local * std::f32::consts::PI).sin().max(0.0) * 0.58);
                let spark_alpha = (local * std::f32::consts::PI).sin().max(0.0);
                background.0 = Color::srgba(0.42, 0.88, 1.0, spark_alpha * 0.96);
            }
            ShengjiPowerOutageVisualKind::TargetRing { target } => {
                let local = ((progress - 0.38) / 0.30).clamp(0.0, 1.0);
                let alive = (0.38..0.68).contains(&progress);
                *visibility = if alive {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                };
                node.left = percent(target.x);
                node.top = percent(target.y);
                transform.translation = Val2::px(-25.0, -25.0);
                transform.scale = Vec2::splat(0.54 + ease_out_cubic(local) * 0.82);
                background.0 = Color::NONE;
                if let Some(mut border) = border {
                    let ring_alpha = (local * std::f32::consts::PI).sin().max(0.0);
                    border.set_all(Color::srgba(0.42, 0.88, 1.0, ring_alpha * 0.92));
                }
            }
            ShengjiPowerOutageVisualKind::LevelFlash { target } => {
                let local = ((progress - 0.45) / 0.43).clamp(0.0, 1.0);
                let enter = ease_out_cubic((local / 0.20).clamp(0.0, 1.0));
                let exit = ease_out_cubic(((local - 0.76) / 0.24).clamp(0.0, 1.0));
                let flash_alpha = enter * (1.0 - exit);
                *visibility = if flash_alpha > 0.0 {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                };
                node.left = percent(target.x);
                node.top = percent(target.y);
                transform.translation = Val2::px(-58.0, -17.0 + 4.0 * (1.0 - enter));
                transform.scale = Vec2::new(0.82 + 0.18 * enter, 1.0);
                background.0 = HEADER_BG.with_alpha(flash_alpha * 0.92);
                if let Some(mut border) = border {
                    border.set_all(ACCENT.with_alpha(flash_alpha * 0.88));
                }
            }
        }
    }
    for (visual, mut node, mut transform, mut background, mut visibility) in
        &mut bottom_flip_visuals
    {
        match visual.kind {
            ShengjiBottomFlipVisualKind::ScanSpark {
                player_index,
                player_count,
                spark_index,
                spark_count,
                start,
                end,
            } => {
                let player_stagger = player_index as f32 / player_count.max(1) as f32 * 0.10;
                let spark_stagger = spark_index as f32 / spark_count.max(1) as f32 * 0.055;
                let local =
                    ((progress - 0.10 - player_stagger - spark_stagger) / 0.22).clamp(0.0, 1.0);
                let alive = progress >= 0.10 + player_stagger + spark_stagger && local < 1.0;
                *visibility = if alive {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                };
                let travel = ease_out_cubic(local);
                let direction = (end - start).normalize_or_zero();
                let perpendicular = Vec2::new(-direction.y, direction.x);
                let polarity = if spark_index % 2 == 0 { 1.0 } else { -1.0 };
                let position = start.lerp(end, travel);
                node.left = percent(position.x);
                node.top = percent(position.y);
                let arc = (local * std::f32::consts::PI).sin();
                transform.translation = Val2::px(
                    -3.5 + perpendicular.x * arc * polarity * 4.0,
                    -3.5 + perpendicular.y * arc * polarity * 4.0,
                );
                transform.scale = Vec2::splat(0.70 + arc * 0.52);
                background.0 = Color::srgba(0.44, 0.90, 1.0, arc * 0.92);
            }
            ShengjiBottomFlipVisualKind::ReplySpark {
                player_index,
                player_count,
                spark_index,
                spark_count,
                start,
                end,
            } => {
                let player_stagger = player_index as f32 / player_count.max(1) as f32 * 0.10;
                let spark_stagger = spark_index as f32 / spark_count.max(1) as f32 * 0.05;
                let local =
                    ((progress - 0.35 - player_stagger - spark_stagger) / 0.24).clamp(0.0, 1.0);
                let alive = progress >= 0.35 + player_stagger + spark_stagger && local < 1.0;
                *visibility = if alive {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                };
                let travel = ease_out_cubic(local);
                let direction = (end - start).normalize_or_zero();
                let perpendicular = Vec2::new(-direction.y, direction.x);
                let polarity = if spark_index % 2 == 0 { 1.0 } else { -1.0 };
                let position = start.lerp(end, travel);
                node.left = percent(position.x);
                node.top = percent(position.y);
                let arc = (local * std::f32::consts::PI).sin();
                transform.translation = Val2::px(
                    -4.0 + perpendicular.x * arc * polarity * 5.0,
                    -4.0 + perpendicular.y * arc * polarity * 5.0,
                );
                transform.rotation = Rot2::radians(polarity * arc * 0.35);
                transform.scale = Vec2::new(0.78 + arc * 0.38, 0.78 + arc * 0.38);
                background.0 = Color::srgba(0.40, 0.94, 0.62, arc * 0.94);
            }
        }
    }
    for (visual, mut node, mut transform, image, mut background, border, mut visibility) in
        &mut trump_kill_visuals
    {
        match visual.kind {
            ShengjiTrumpKillVisualKind::Target => {
                let reveal = ease_out_cubic((progress / 0.16).clamp(0.0, 1.0));
                let fade = 1.0 - ease_out_cubic(((progress - 0.84) / 0.16).clamp(0.0, 1.0));
                let impact = ((progress - 0.48) / 0.16).clamp(0.0, 1.0);
                let pulse = (impact * std::f32::consts::PI).sin().max(0.0);
                *visibility = if fade > 0.0 {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                };
                transform.scale = Vec2::splat((0.68 + 0.32 * reveal) * (1.0 + pulse * 0.16));
                if let Some(mut image) = image {
                    image.color = Color::srgba(0.36, 0.94, 0.67, reveal * fade * 0.92);
                }
                background.0 = Color::NONE;
            }
            ShengjiTrumpKillVisualKind::Dart => {
                let local = ((progress - 0.10) / 0.42).clamp(0.0, 1.0);
                let alive = (0.10..0.58).contains(&progress);
                *visibility = if alive {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                };
                let travel = ease_out_cubic(local);
                node.left = px(-42.0 + 118.0 * travel);
                node.top = px(4.0 - (local * std::f32::consts::PI).sin() * 7.0);
                transform.rotation = Rot2::radians((-45.0 + (1.0 - local) * 3.0).to_radians());
                transform.scale = Vec2::splat(0.88 + 0.12 * travel);
                if let Some(mut image) = image {
                    let fade = 1.0 - ((local - 0.90) / 0.10).clamp(0.0, 1.0);
                    image.color = Color::srgba(1.0, 0.78, 0.16, fade);
                }
                background.0 = Color::NONE;
            }
            ShengjiTrumpKillVisualKind::ImpactRing => {
                let local = ((progress - 0.48) / 0.30).clamp(0.0, 1.0);
                let alive = (0.48..0.78).contains(&progress);
                *visibility = if alive {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                };
                transform.scale = Vec2::splat(0.42 + ease_out_cubic(local) * 1.04);
                background.0 = Color::NONE;
                if let Some(mut border) = border {
                    border.set_all(ACCENT.with_alpha((1.0 - local) * 0.90));
                }
            }
        }
    }
}

pub(in crate::app) fn animate_shengji_bottom_flip_markers(
    state: Res<ShengjiPresentationState>,
    mut panel_elements: Query<
        (
            &ShengjiBottomFlipPanelElement,
            &mut UiTransform,
            &mut Visibility,
        ),
        Without<PlayerAvatarAnchor>,
    >,
    mut avatars: Query<
        (&PlayerAvatarAnchor, &mut UiTransform),
        Without<ShengjiBottomFlipPanelElement>,
    >,
) {
    let bottom_flip = state.active.as_ref().and_then(|active| {
        let ShengjiPresentationKind::BottomFlip {
            matches, dealer, ..
        } = &active.kind
        else {
            return None;
        };
        Some((
            (active.elapsed / active.duration).clamp(0.0, 1.0),
            matches.as_slice(),
            *dealer,
        ))
    });

    let Some((progress, matches, dealer)) = bottom_flip else {
        for (_, mut transform, mut visibility) in &mut panel_elements {
            *transform = UiTransform::IDENTITY;
            *visibility = Visibility::Visible;
        }
        for (_, mut transform) in &mut avatars {
            *transform = UiTransform::IDENTITY;
        }
        return;
    };

    for (element, mut transform, mut visibility) in &mut panel_elements {
        match *element {
            ShengjiBottomFlipPanelElement::CentralCard => {
                let reveal = ease_out_cubic((progress / 0.16).clamp(0.0, 1.0));
                *visibility = Visibility::Visible;
                transform.scale = Vec2::new(0.06 + reveal * 0.94, 0.90 + reveal * 0.10);
                transform.translation = Val2::px(0.0, 5.0 * (1.0 - reveal));
            }
            ShengjiBottomFlipPanelElement::MatchRow {
                player,
                index,
                count,
            } => {
                let still_matches = matches.iter().any(|matched| matched.player == player);
                let stagger = index as f32 / count.max(1) as f32 * 0.10;
                let reveal = ease_out_cubic(((progress - 0.50 - stagger) / 0.13).clamp(0.0, 1.0));
                *visibility = if still_matches && reveal > 0.0 {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                };
                let direction = if index % 2 == 0 { -1.0 } else { 1.0 };
                transform.translation = Val2::px(direction * 18.0 * (1.0 - reveal), 0.0);
                transform.scale = Vec2::new(0.94 + 0.06 * reveal, 0.94 + 0.06 * reveal);
            }
            ShengjiBottomFlipPanelElement::DealerLine => {
                let reveal = ease_out_cubic(((progress - 0.68) / 0.13).clamp(0.0, 1.0));
                *visibility = if reveal > 0.0 {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                };
                let bounce = (reveal * std::f32::consts::PI).sin().max(0.0);
                transform.translation = Val2::px(0.0, 7.0 * (1.0 - reveal));
                transform.scale = Vec2::splat(0.88 + reveal * 0.12 + bounce * 0.08);
            }
        }
    }

    for (anchor, mut transform) in &mut avatars {
        *transform = UiTransform::IDENTITY;
        let Some(index) = matches
            .iter()
            .position(|matched| matched.player == anchor.0)
        else {
            continue;
        };
        let stagger = index as f32 / matches.len().max(1) as f32 * 0.10;
        let response = ((progress - 0.27 - stagger) / 0.22).clamp(0.0, 1.0);
        let response_pulse = (response * std::f32::consts::PI).sin().max(0.0);
        let dealer_pulse = if dealer == Some(anchor.0) {
            (((progress - 0.63) / 0.20).clamp(0.0, 1.0) * std::f32::consts::PI)
                .sin()
                .max(0.0)
        } else {
            0.0
        };
        transform.scale = Vec2::splat(1.0 + response_pulse * 0.14 + dealer_pulse * 0.16);
        transform.rotation = Rot2::radians(response_pulse * 0.035 - dealer_pulse * 0.025);
    }
}

pub(in crate::app) fn animate_shengji_power_outage_markers(
    state: Res<ShengjiPresentationState>,
    mut badges: Query<
        (&mut UiTransform, &mut BackgroundColor, &mut Visibility),
        With<ShengjiDealerBadge>,
    >,
    mut levels: Query<
        (&ShengjiLevelIndicator, &mut UiTransform, &mut TextColor),
        Without<ShengjiDealerBadge>,
    >,
) {
    let progress = state.active.as_ref().and_then(|active| {
        matches!(active.kind, ShengjiPresentationKind::PowerOutage { .. })
            .then_some((active.elapsed / active.duration).clamp(0.0, 1.0))
    });
    let Some(progress) = progress else {
        for (mut transform, mut background, mut visibility) in &mut badges {
            *visibility = Visibility::Visible;
            *transform = UiTransform::IDENTITY;
            background.0 = ACCENT;
        }
        for (indicator, mut transform, mut color) in &mut levels {
            *transform = UiTransform::IDENTITY;
            color.0 = indicator.base_color;
        }
        return;
    };

    let badge_reveal = ease_out_cubic(((progress - 0.39) / 0.14).clamp(0.0, 1.0));
    for (mut transform, mut background, mut visibility) in &mut badges {
        *visibility = if badge_reveal > 0.0 {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        transform.scale = Vec2::splat(0.52 + 0.58 * badge_reveal - 0.10 * badge_reveal.powi(2));
        background.0 = ACCENT.with_alpha(badge_reveal);
    }

    let dim = 1.0 - 0.82 * ease_out_cubic((progress / 0.10).clamp(0.0, 1.0));
    let level_reveal = ease_out_cubic(((progress - 0.43) / 0.18).clamp(0.0, 1.0));
    for (indicator, mut transform, mut color) in &mut levels {
        transform.scale = Vec2::new(0.08 + 0.92 * level_reveal, 1.0);
        transform.translation = Val2::px(0.0, -3.0 * (1.0 - level_reveal));
        color.0 = indicator
            .base_color
            .with_alpha(dim.max(0.18 + level_reveal * 0.82));
    }
}

fn shengji_relative_seat(game: &ShengjiSnapshot, player: PlayerId) -> Option<u8> {
    const SEAT_COUNT: u8 = 4;
    let own_seat = game
        .players
        .iter()
        .find(|candidate| candidate.id == game.you)?
        .seat
        .0;
    let player_seat = game
        .players
        .iter()
        .find(|candidate| candidate.id == player)?
        .seat
        .0;
    Some((player_seat + SEAT_COUNT - own_seat) % SEAT_COUNT)
}

/// 百分比坐标以整张牌桌为基准，并落在各座位靠桌面内侧的出牌区。
fn shengji_seat_route_anchor(relative_seat: u8) -> Vec2 {
    match relative_seat {
        0 => Vec2::new(50.0, 88.0),
        1 => Vec2::new(20.0, 52.0),
        2 => Vec2::new(50.0, 20.0),
        3 => Vec2::new(80.0, 52.0),
        _ => Vec2::new(50.0, 50.0),
    }
}

fn shengji_player_route_anchor(game: &ShengjiSnapshot, player: PlayerId) -> Option<Vec2> {
    shengji_relative_seat(game, player).map(shengji_seat_route_anchor)
}

/// 扣底的“查找同牌”动画要指向玩家本人，而不是平时的出牌区。
/// 这组锚点因此落在四个玩家框的头像附近。
fn shengji_seat_player_anchor(relative_seat: u8) -> Vec2 {
    match relative_seat {
        0 => Vec2::new(92.0, 91.0),
        1 => Vec2::new(4.5, 52.0),
        2 => Vec2::new(50.0, 10.5),
        3 => Vec2::new(95.5, 52.0),
        _ => Vec2::new(50.0, 50.0),
    }
}

fn shengji_player_panel_anchor(game: &ShengjiSnapshot, player: PlayerId) -> Option<Vec2> {
    shengji_relative_seat(game, player).map(shengji_seat_player_anchor)
}

fn shengji_power_outage_anchors(
    kind: &ShengjiPresentationKind,
    game: &ShengjiSnapshot,
) -> Option<(Vec2, Vec2)> {
    let ShengjiPresentationKind::PowerOutage {
        from_dealer,
        dealer,
        ..
    } = kind
    else {
        return None;
    };
    let start = from_dealer
        .and_then(|player| shengji_player_route_anchor(game, player))
        .unwrap_or(Vec2::new(50.0, 50.0));
    let end = shengji_player_route_anchor(game, *dealer).unwrap_or(Vec2::new(50.0, 50.0));
    Some((start, end))
}

fn shengji_partner_player(game: &ShengjiSnapshot, player: PlayerId) -> Option<PlayerId> {
    let seat = game
        .players
        .iter()
        .find(|candidate| candidate.id == player)?
        .seat
        .0;
    game.players
        .iter()
        .find(|candidate| candidate.seat.0 == (seat + 2) % 4)
        .map(|candidate| candidate.id)
}

fn shengji_presentation_routes(
    kind: &ShengjiPresentationKind,
    game: &ShengjiSnapshot,
) -> Vec<ShengjiPresentationRoute> {
    match kind {
        ShengjiPresentationKind::BottomCopy {
            from_player,
            player,
            ..
        } => shengji_player_route_anchor(game, *player)
            .map(|end| ShengjiPresentationRoute {
                start: from_player
                    .and_then(|player| shengji_player_route_anchor(game, player))
                    .unwrap_or(Vec2::new(50.0, 50.0)),
                end,
                packet_count: game.rules.kitty_size(),
            })
            .into_iter()
            .collect(),
        ShengjiPresentationKind::CrossingStarted { players } => players
            .iter()
            .filter_map(|player| {
                let relative = shengji_relative_seat(game, *player)?;
                Some(ShengjiPresentationRoute {
                    start: shengji_seat_route_anchor(relative),
                    end: shengji_seat_route_anchor((relative + 2) % 4),
                    packet_count: 5,
                })
            })
            .collect(),
        ShengjiPresentationKind::CrossingReturned { player, .. } => {
            shengji_relative_seat(game, *player)
                .map(|relative| ShengjiPresentationRoute {
                    start: shengji_seat_route_anchor(relative),
                    end: shengji_seat_route_anchor((relative + 2) % 4),
                    packet_count: 5,
                })
                .into_iter()
                .collect()
        }
        _ => Vec::new(),
    }
}

pub(in crate::app) fn add_shengji_presentation_overlay(
    commands: &mut Commands,
    table: Entity,
    game: &ShengjiSnapshot,
    state: &ShengjiPresentationState,
    assets: &UiAssets,
) {
    let Some(active) = state.active.as_ref() else {
        return;
    };
    let is_play = matches!(
        &active.kind,
        ShengjiPresentationKind::Play { .. } | ShengjiPresentationKind::TrumpKill { .. }
    );
    let is_trump_kill = matches!(&active.kind, ShengjiPresentationKind::TrumpKill { .. });
    let bottom_flip_matches = match &active.kind {
        ShengjiPresentationKind::BottomFlip { matches, .. } => Some(matches.as_slice()),
        _ => None,
    };
    let is_bottom_flip = bottom_flip_matches.is_some();
    let power_outage_anchors = shengji_power_outage_anchors(&active.kind, game);
    let is_power_outage = power_outage_anchors.is_some();
    let routes = shengji_presentation_routes(&active.kind, game);
    let is_routed = !routes.is_empty();
    let mut root_node = Node {
        position_type: PositionType::Absolute,
        height: px(132),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        flex_direction: FlexDirection::Column,
        row_gap: px(8),
        ..default()
    };
    let mut base_translation = Vec2::ZERO;
    let anchored_player = match &active.kind {
        ShengjiPresentationKind::Play { player, .. }
        | ShengjiPresentationKind::TrumpKill { player, .. } => Some(*player),
        _ => None,
    };
    if let Some(player) = anchored_player {
        root_node.width = px(if is_trump_kill { 190 } else { 180 });
        root_node.height = px(if is_trump_kill { 92 } else { 64 });
        let half_width = if is_trump_kill { 95.0 } else { 90.0 };
        if is_trump_kill {
            root_node.justify_content = JustifyContent::FlexEnd;
            root_node.row_gap = px(2);
            root_node.padding = UiRect::bottom(px(3));
        }
        let own_seat = game
            .players
            .iter()
            .find(|candidate| candidate.id == game.you)
            .map_or(0, |candidate| candidate.seat.0);
        let player_seat = game
            .players
            .iter()
            .find(|candidate| candidate.id == player)
            .map_or(own_seat, |candidate| candidate.seat.0);
        match (player_seat + 4 - own_seat) % 4 {
            0 => {
                root_node.left = percent(50);
                root_node.bottom = px(116);
                base_translation.x = -half_width;
            }
            1 => {
                root_node.left = px(260);
                root_node.top = percent(50);
                base_translation.y = -92.0;
            }
            2 => {
                root_node.left = percent(50);
                root_node.top = px(202);
                base_translation.x = -half_width;
            }
            3 => {
                root_node.right = px(260);
                root_node.top = percent(50);
                base_translation.y = -92.0;
            }
            _ => unreachable!(),
        }
    } else if is_routed || is_power_outage || is_bottom_flip {
        // 交牌和换庄演出覆盖整张桌面，移动方向才能和真实座位一致。
        root_node.left = px(0);
        root_node.top = px(0);
        root_node.width = percent(100);
        root_node.height = percent(100);
        root_node.row_gap = px(5);
    } else {
        // 规则事件始终以牌桌的几何中心为锚点。
        root_node.left = percent(25);
        root_node.right = percent(25);
        root_node.top = percent(50);
        base_translation.y = -66.0;
    }
    let root = spawn_node(commands, table, root_node, None);
    commands.entity(root).insert((
        ShengjiPresentationRoot { base_translation },
        GlobalZIndex(if is_bottom_flip { 920 } else { 520 }),
        FocusPolicy::Pass,
    ));
    let veil = spawn_node(
        commands,
        root,
        Node {
            position_type: PositionType::Absolute,
            left: px(-500),
            right: px(-500),
            top: px(-420),
            bottom: px(-420),
            ..default()
        },
        Some(Color::NONE),
    );
    commands
        .entity(veil)
        .insert((ShengjiPresentationVeil, FocusPolicy::Pass));

    if let Some(matches) = bottom_flip_matches {
        const CENTER: Vec2 = Vec2::new(50.0, 30.5);
        const SCAN_SPARK_COUNT: usize = 6;
        const REPLY_SPARK_COUNT: usize = 5;
        for (player_index, matched) in matches.iter().enumerate() {
            let Some(target) = shengji_player_panel_anchor(game, matched.player) else {
                continue;
            };
            for spark_index in 0..SCAN_SPARK_COUNT {
                let spark = spawn_node(
                    commands,
                    root,
                    Node {
                        position_type: PositionType::Absolute,
                        left: percent(CENTER.x),
                        top: percent(CENTER.y),
                        width: px(7),
                        height: px(7),
                        border_radius: BorderRadius::all(percent(50)),
                        ..default()
                    },
                    Some(Color::NONE),
                );
                commands.entity(spark).insert((
                    ShengjiBottomFlipVisual {
                        kind: ShengjiBottomFlipVisualKind::ScanSpark {
                            player_index,
                            player_count: matches.len(),
                            spark_index,
                            spark_count: SCAN_SPARK_COUNT,
                            start: CENTER,
                            end: target,
                        },
                    },
                    UiTransform::from_translation(Val2::px(-3.5, -3.5)),
                    Visibility::Hidden,
                    FocusPolicy::Pass,
                ));
            }

            let reply_end = CENTER.lerp(target, 0.30);
            for spark_index in 0..REPLY_SPARK_COUNT {
                let spark = spawn_node(
                    commands,
                    root,
                    Node {
                        position_type: PositionType::Absolute,
                        left: percent(target.x),
                        top: percent(target.y),
                        width: px(8),
                        height: px(8),
                        border_radius: BorderRadius::all(px(2)),
                        ..default()
                    },
                    Some(Color::NONE),
                );
                commands.entity(spark).insert((
                    ShengjiBottomFlipVisual {
                        kind: ShengjiBottomFlipVisualKind::ReplySpark {
                            player_index,
                            player_count: matches.len(),
                            spark_index,
                            spark_count: REPLY_SPARK_COUNT,
                            start: target,
                            end: reply_end,
                        },
                    },
                    UiTransform::from_translation(Val2::px(-4.0, -4.0)),
                    Visibility::Hidden,
                    FocusPolicy::Pass,
                ));
            }
        }
    }

    if let Some((start, end)) = power_outage_anchors {
        const SPARK_COUNT: usize = 7;
        for index in 0..SPARK_COUNT {
            let spark = spawn_node(
                commands,
                root,
                Node {
                    position_type: PositionType::Absolute,
                    left: percent(start.x),
                    top: percent(start.y),
                    width: px(8),
                    height: px(8),
                    border_radius: BorderRadius::all(percent(50)),
                    ..default()
                },
                Some(Color::NONE),
            );
            commands.entity(spark).insert((
                ShengjiPowerOutageVisual {
                    kind: ShengjiPowerOutageVisualKind::Spark {
                        index,
                        count: SPARK_COUNT,
                        start,
                        end,
                    },
                },
                UiTransform::from_translation(Val2::px(-4.0, -4.0)),
                Visibility::Hidden,
                FocusPolicy::Pass,
            ));
        }

        let ring = spawn_node(
            commands,
            root,
            Node {
                position_type: PositionType::Absolute,
                left: percent(end.x),
                top: percent(end.y),
                width: px(50),
                height: px(50),
                border: UiRect::all(px(2)),
                border_radius: BorderRadius::all(percent(50)),
                ..default()
            },
            Some(Color::NONE),
        );
        commands.entity(ring).insert((
            ShengjiPowerOutageVisual {
                kind: ShengjiPowerOutageVisualKind::TargetRing { target: end },
            },
            BorderColor::all(Color::NONE),
            UiTransform::from_translation(Val2::px(-25.0, -25.0)),
            Visibility::Hidden,
            FocusPolicy::Pass,
        ));

        let level_target = end.lerp(Vec2::new(50.0, 50.0), 0.24);
        let level = spawn_node(
            commands,
            root,
            Node {
                position_type: PositionType::Absolute,
                left: percent(level_target.x),
                top: percent(level_target.y),
                width: px(116),
                height: px(34),
                border: UiRect::all(px(1)),
                border_radius: BorderRadius::all(px(5)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            Some(Color::NONE),
        );
        commands.entity(level).insert((
            ShengjiPowerOutageVisual {
                kind: ShengjiPowerOutageVisualKind::LevelFlash {
                    target: level_target,
                },
            },
            BorderColor::all(Color::NONE),
            UiTransform::from_translation(Val2::px(-58.0, -17.0)),
            Visibility::Hidden,
            FocusPolicy::Pass,
        ));
        let rank = match &active.kind {
            ShengjiPresentationKind::PowerOutage { level, .. } => rank_label(*level),
            _ => unreachable!(),
        };
        add_text(
            commands,
            level,
            format!("新庄 · 打{rank}"),
            15.0,
            ACCENT,
            assets,
        );
    }

    if is_trump_kill {
        let target = spawn_node(
            commands,
            root,
            Node {
                position_type: PositionType::Absolute,
                left: px(108),
                top: px(1),
                width: px(58),
                height: px(58),
                ..default()
            },
            Some(Color::NONE),
        );
        commands.entity(target).insert((
            ShengjiTrumpKillVisual {
                kind: ShengjiTrumpKillVisualKind::Target,
            },
            ImageNode::new(assets.shengji_target.clone()).with_color(Color::NONE),
            UiTransform::IDENTITY,
            Visibility::Hidden,
            FocusPolicy::Pass,
        ));

        let dart = spawn_node(
            commands,
            root,
            Node {
                position_type: PositionType::Absolute,
                left: px(-42),
                top: px(10),
                width: px(52),
                height: px(52),
                ..default()
            },
            Some(Color::NONE),
        );
        commands.entity(dart).insert((
            ShengjiTrumpKillVisual {
                kind: ShengjiTrumpKillVisualKind::Dart,
            },
            ImageNode::new(assets.shengji_dart.clone()).with_color(Color::NONE),
            UiTransform {
                rotation: Rot2::radians((-45.0_f32).to_radians()),
                ..default()
            },
            Visibility::Hidden,
            FocusPolicy::Pass,
        ));

        let ring = spawn_node(
            commands,
            root,
            Node {
                position_type: PositionType::Absolute,
                left: px(110),
                top: px(3),
                width: px(56),
                height: px(56),
                border: UiRect::all(px(2)),
                border_radius: BorderRadius::all(percent(50)),
                ..default()
            },
            Some(Color::NONE),
        );
        commands.entity(ring).insert((
            ShengjiTrumpKillVisual {
                kind: ShengjiTrumpKillVisualKind::ImpactRing,
            },
            BorderColor::all(Color::NONE),
            UiTransform::IDENTITY,
            Visibility::Hidden,
            FocusPolicy::Pass,
        ));
    }

    for (route_index, route) in routes.iter().enumerate() {
        for index in 0..route.packet_count {
            let packet = spawn_node(
                commands,
                root,
                Node {
                    position_type: PositionType::Absolute,
                    left: percent(route.start.x),
                    top: percent(route.start.y),
                    width: px(34),
                    height: px(48),
                    border_radius: BorderRadius::all(px(4)),
                    ..default()
                },
                None,
            );
            commands.entity(packet).insert((
                ShengjiPresentationPacket {
                    index,
                    count: route.packet_count,
                    route_index,
                    start: route.start,
                    end: route.end,
                },
                ImageNode::new(assets.card_back.clone()).with_mode(NodeImageMode::Stretch),
                UiTransform::from_translation(Val2::px(-17.0, -24.0)),
                FocusPolicy::Pass,
            ));
        }
    }

    // 扣底的标题、底牌和匹配牌已经在中央面板内完整展示，
    // 此处只叠加玩家定向的交互动画，避免再显示一套重复文字。
    if is_bottom_flip {
        return;
    }

    let (title, subtitle) = presentation_text(&active.kind, game);
    let title_color = presentation_color(&active.kind);
    let title = add_text(
        commands,
        root,
        title,
        if is_play {
            if is_trump_kill { 17.0 } else { 19.0 }
        } else if is_routed {
            22.0
        } else {
            25.0
        },
        title_color,
        assets,
    );
    commands.entity(title).insert(ShengjiPresentationText {
        base_color: title_color,
    });
    let divider = spawn_node(
        commands,
        root,
        Node {
            width: px(if is_trump_kill {
                42.0
            } else if is_play {
                54.0
            } else {
                88.0
            }),
            height: px(2),
            ..default()
        },
        Some(title_color.with_alpha(0.72)),
    );
    commands
        .entity(divider)
        .insert((ShengjiPresentationDivider, FocusPolicy::Pass));
    if !subtitle.is_empty() {
        let subtitle = add_text(commands, root, subtitle, 15.0, TEXT, assets);
        commands
            .entity(subtitle)
            .insert(ShengjiPresentationText { base_color: TEXT });
    }
}

fn classify_play_presentation(
    play: &leocard_shengji::ClassifiedPlay,
) -> ShengjiPlayPresentationKind {
    if play.is_throw() {
        return ShengjiPlayPresentationKind::Throw;
    }
    match play.strongest_component() {
        leocard_shengji::Component::Single { .. } => ShengjiPlayPresentationKind::Single,
        leocard_shengji::Component::Pair { .. } => ShengjiPlayPresentationKind::Pair,
        leocard_shengji::Component::Tractor { .. } => ShengjiPlayPresentationKind::Tractor,
        leocard_shengji::Component::Triple { .. } => ShengjiPlayPresentationKind::Triple,
        leocard_shengji::Component::Titanic { .. } => ShengjiPlayPresentationKind::Titanic,
        leocard_shengji::Component::Quad { .. } => ShengjiPlayPresentationKind::Bomb,
        leocard_shengji::Component::Spaceship { .. } => ShengjiPlayPresentationKind::Spaceship,
    }
}

fn should_show_play_presentation(
    kind: ShengjiPlayPresentationKind,
    is_lead: bool,
    throw_penalty: u16,
) -> bool {
    if throw_penalty > 0
        || matches!(
            kind,
            ShengjiPlayPresentationKind::Single
                | ShengjiPlayPresentationKind::Pair
                | ShengjiPlayPresentationKind::Triple
        )
    {
        return false;
    }
    kind != ShengjiPlayPresentationKind::Throw || is_lead
}

fn queue_play_audio(
    cues: &mut Vec<ShengjiAudioCue>,
    kind: ShengjiPlayPresentationKind,
    seed: u64,
    start: f32,
) {
    let cue = |kind, remaining, volume, offset| {
        ShengjiAudioCue::new(
            kind,
            start + remaining,
            volume,
            seed.wrapping_mul(31) + offset,
        )
    };
    match kind {
        ShengjiPlayPresentationKind::Single => {
            cues.push(cue(ShengjiSoundKind::CardPlace, 0.0, 0.34, 1));
        }
        ShengjiPlayPresentationKind::Pair => cues.extend([
            cue(ShengjiSoundKind::CardPlace, 0.0, 0.30, 1),
            cue(ShengjiSoundKind::CardPlace, 0.075, 0.42, 2),
        ]),
        ShengjiPlayPresentationKind::Tractor => cues.extend([
            cue(ShengjiSoundKind::CardShove, 0.0, 0.44, 1),
            cue(ShengjiSoundKind::CardPlace, 0.18, 0.32, 2),
        ]),
        ShengjiPlayPresentationKind::Triple => cues.extend([
            cue(ShengjiSoundKind::CardPlace, 0.0, 0.26, 1),
            cue(ShengjiSoundKind::CardPlace, 0.065, 0.32, 2),
            cue(ShengjiSoundKind::CardPlace, 0.13, 0.44, 3),
        ]),
        ShengjiPlayPresentationKind::Titanic => cues.extend([
            cue(ShengjiSoundKind::CardShove, 0.0, 0.46, 1),
            cue(ShengjiSoundKind::Heavy, 0.16, 0.48, 2),
        ]),
        ShengjiPlayPresentationKind::Bomb => cues.extend([
            cue(ShengjiSoundKind::CardPlace, 0.0, 0.32, 1),
            cue(ShengjiSoundKind::Bomb, 0.10, 0.52, 2),
        ]),
        ShengjiPlayPresentationKind::Spaceship => cues.extend([
            cue(ShengjiSoundKind::Bomb, 0.0, 0.48, 1),
            cue(ShengjiSoundKind::Bomb, 0.18, 0.44, 2),
            cue(ShengjiSoundKind::CardShove, 0.28, 0.50, 3),
        ]),
        ShengjiPlayPresentationKind::Throw => cues.extend([
            cue(ShengjiSoundKind::CardShove, 0.0, 0.46, 1),
            cue(ShengjiSoundKind::CardPlace, 0.20, 0.30, 2),
        ]),
    }
}

fn presentation_text(kind: &ShengjiPresentationKind, game: &ShengjiSnapshot) -> (String, String) {
    let player_name = |player: PlayerId| {
        game.players
            .iter()
            .find(|candidate| candidate.id == player)
            .map_or_else(
                || format!("玩家{}", player.0 + 1),
                |player| player.name.clone(),
            )
    };
    match kind {
        ShengjiPresentationKind::Declaration {
            player,
            trump,
            label,
        } => (
            (*label).to_owned(),
            format!("{} · {}", player_name(*player), bid_trump_label(*trump)),
        ),
        ShengjiPresentationKind::PowerOutage {
            from_dealer,
            dealer,
            level,
        } => (
            "断电换庄".to_owned(),
            from_dealer.map_or_else(
                || format!("{} 接庄 · 打{}", player_name(*dealer), rank_label(*level)),
                |from_dealer| {
                    format!(
                        "{} → {} · 打{}",
                        player_name(from_dealer),
                        player_name(*dealer),
                        rank_label(*level)
                    )
                },
            ),
        ),
        ShengjiPresentationKind::BottomFlip {
            card,
            matches,
            dealer,
        } => (
            "扳底翻牌".to_owned(),
            dealer.map_or_else(
                || format!("{} 人持有同牌，继续判定", matches.len()),
                |dealer| {
                    format!(
                        "{} 坐庄 · 翻出{}",
                        player_name(dealer),
                        card_face_label(*card)
                    )
                },
            ),
        ),
        ShengjiPresentationKind::BottomCopy {
            from_player,
            player,
            trump,
            count,
        } => (
            format!("第{count}次抄底"),
            from_player.map_or_else(
                || format!("{} · {}", player_name(*player), bid_trump_label(*trump)),
                |from_player| {
                    format!(
                        "{} → {} · {}",
                        player_name(from_player),
                        player_name(*player),
                        bid_trump_label(*trump)
                    )
                },
            ),
        ),
        ShengjiPresentationKind::CrossingStarted { players } => {
            let directions = players
                .iter()
                .filter_map(|player| {
                    shengji_partner_player(game, *player).map(|partner| {
                        format!("{} → {}", player_name(*player), player_name(partner))
                    })
                })
                .collect::<Vec<_>>()
                .join("　");
            (
                "五主过江".to_owned(),
                if directions.is_empty() {
                    format!("{} 名玩家向对家交牌", players.len())
                } else {
                    directions
                },
            )
        }
        ShengjiPresentationKind::CrossingReturned { player, complete } => (
            if *complete {
                "过江完成".to_owned()
            } else {
                "归还五张".to_owned()
            },
            shengji_partner_player(game, *player).map_or_else(
                || format!("{} 已完成归还", player_name(*player)),
                |partner| format!("{} → {}", player_name(*player), player_name(partner)),
            ),
        ),
        ShengjiPresentationKind::TrumpKill { covered, .. } => (
            if *covered { "盖毙" } else { "毙牌" }.to_owned(),
            String::new(),
        ),
        ShengjiPresentationKind::Play { kind, .. } => (kind.label().to_owned(), String::new()),
    }
}

fn presentation_color(kind: &ShengjiPresentationKind) -> Color {
    match kind {
        ShengjiPresentationKind::PowerOutage { .. } => Color::srgb(0.35, 0.78, 1.0),
        ShengjiPresentationKind::BottomFlip { .. } => Color::srgb(1.0, 0.70, 0.16),
        ShengjiPresentationKind::BottomCopy { .. } => Color::srgb(0.82, 0.43, 1.0),
        ShengjiPresentationKind::CrossingStarted { .. }
        | ShengjiPresentationKind::CrossingReturned { .. } => Color::srgb(0.20, 0.88, 0.72),
        ShengjiPresentationKind::TrumpKill { covered: true, .. } => Color::srgb(1.0, 0.62, 0.14),
        ShengjiPresentationKind::TrumpKill { covered: false, .. } => Color::srgb(0.36, 0.94, 0.67),
        ShengjiPresentationKind::Play {
            kind: ShengjiPlayPresentationKind::Bomb,
            ..
        }
        | ShengjiPresentationKind::Play {
            kind: ShengjiPlayPresentationKind::Spaceship,
            ..
        } => Color::srgb(1.0, 0.27, 0.16),
        ShengjiPresentationKind::Play {
            kind: ShengjiPlayPresentationKind::Titanic,
            ..
        } => Color::srgb(0.24, 0.68, 1.0),
        _ => ACCENT,
    }
}

fn bid_trump_label(trump: leocard_shengji::BidTrump) -> String {
    match trump {
        leocard_shengji::BidTrump::Suit(ShengjiSuit::Diamond) => "♦ 方块主",
        leocard_shengji::BidTrump::Suit(ShengjiSuit::Club) => "♣ 梅花主",
        leocard_shengji::BidTrump::Suit(ShengjiSuit::Heart) => "♥ 红桃主",
        leocard_shengji::BidTrump::Suit(ShengjiSuit::Spade) => "♠ 黑桃主",
        leocard_shengji::BidTrump::NoTrumpSmallJoker
        | leocard_shengji::BidTrump::NoTrumpBigJoker => "无主",
    }
    .to_owned()
}

fn rank_label(rank: leocard_shengji::Rank) -> &'static str {
    match rank {
        leocard_shengji::Rank::Two => "2",
        leocard_shengji::Rank::Three => "3",
        leocard_shengji::Rank::Four => "4",
        leocard_shengji::Rank::Five => "5",
        leocard_shengji::Rank::Six => "6",
        leocard_shengji::Rank::Seven => "7",
        leocard_shengji::Rank::Eight => "8",
        leocard_shengji::Rank::Nine => "9",
        leocard_shengji::Rank::Ten => "10",
        leocard_shengji::Rank::Jack => "J",
        leocard_shengji::Rank::Queen => "Q",
        leocard_shengji::Rank::King => "K",
        leocard_shengji::Rank::Ace => "A",
        leocard_shengji::Rank::SmallJoker => "小王",
        leocard_shengji::Rank::BigJoker => "大王",
    }
}

fn card_face_label(card: ShengjiCard) -> String {
    let suit = match card.suit() {
        Some(ShengjiSuit::Diamond) => "♦",
        Some(ShengjiSuit::Club) => "♣",
        Some(ShengjiSuit::Heart) => "♥",
        Some(ShengjiSuit::Spade) => "♠",
        None => "",
    };
    format!("{suit}{}", rank_label(card.rank()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn routed_presentations_point_at_all_four_relative_seats() {
        let own = shengji_seat_route_anchor(0);
        let left = shengji_seat_route_anchor(1);
        let top = shengji_seat_route_anchor(2);
        let right = shengji_seat_route_anchor(3);

        assert!(own.y > 50.0);
        assert!(top.y < 50.0);
        assert!(left.x < 50.0);
        assert!(right.x > 50.0);
        assert_ne!(own, top);
        assert_ne!(left, right);
    }

    #[test]
    fn five_and_ten_point_throw_penalties_use_distinct_sound_weights() {
        let mut five = Vec::new();
        queue_throw_failure_audio(&mut five, ShengjiThrowPenalty::FivePerCard);
        let mut ten = Vec::new();
        queue_throw_failure_audio(&mut ten, ShengjiThrowPenalty::TenPerCard);

        assert!(
            five.iter()
                .any(|cue| cue.kind == ShengjiSoundKind::PenaltyFive)
        );
        assert!(!five.iter().any(|cue| cue.kind == ShengjiSoundKind::Heavy));
        assert!(
            ten.iter()
                .any(|cue| cue.kind == ShengjiSoundKind::PenaltyTen)
        );
        assert!(ten.iter().any(|cue| cue.kind == ShengjiSoundKind::Heavy));
    }

    #[test]
    fn every_shengji_structure_maps_to_the_declared_presentation_family() {
        use leocard_shengji::{Category, ClassifiedPlay, Component};

        let copies = |rank, count: u8| {
            (0..count)
                .map(|deck| ShengjiCard::suited(deck, ShengjiSuit::Spade, rank))
                .collect::<Vec<_>>()
        };
        let play = |components: Vec<Component>| ClassifiedPlay {
            cards: components.iter().flat_map(Component::cards).collect(),
            category: Category::Suit(ShengjiSuit::Spade),
            components,
        };
        let single_card = copies(leocard_shengji::Rank::Three, 1)[0];
        let pair_cards: [ShengjiCard; 2] =
            copies(leocard_shengji::Rank::Four, 2).try_into().unwrap();
        let triple_cards: [ShengjiCard; 3] =
            copies(leocard_shengji::Rank::Five, 3).try_into().unwrap();
        let bomb_cards: [ShengjiCard; 4] =
            copies(leocard_shengji::Rank::Six, 4).try_into().unwrap();
        let tractor_cards = [
            copies(leocard_shengji::Rank::Seven, 2),
            copies(leocard_shengji::Rank::Eight, 2),
        ]
        .concat();
        let titanic_cards = [
            copies(leocard_shengji::Rank::Nine, 3),
            copies(leocard_shengji::Rank::Ten, 3),
        ]
        .concat();
        let spaceship_cards = [
            copies(leocard_shengji::Rank::Jack, 4),
            copies(leocard_shengji::Rank::Queen, 4),
        ]
        .concat();

        let cases = [
            (
                play(vec![Component::Single {
                    card: single_card,
                    strength: 1,
                }]),
                ShengjiPlayPresentationKind::Single,
            ),
            (
                play(vec![Component::Pair {
                    cards: pair_cards,
                    strength: 2,
                }]),
                ShengjiPlayPresentationKind::Pair,
            ),
            (
                play(vec![Component::Tractor {
                    cards: tractor_cards,
                    pair_count: 2,
                    top_strength: 4,
                }]),
                ShengjiPlayPresentationKind::Tractor,
            ),
            (
                play(vec![Component::Triple {
                    cards: triple_cards,
                    strength: 5,
                }]),
                ShengjiPlayPresentationKind::Triple,
            ),
            (
                play(vec![Component::Titanic {
                    cards: titanic_cards,
                    triple_count: 2,
                    top_strength: 7,
                }]),
                ShengjiPlayPresentationKind::Titanic,
            ),
            (
                play(vec![Component::Quad {
                    cards: bomb_cards,
                    strength: 8,
                }]),
                ShengjiPlayPresentationKind::Bomb,
            ),
            (
                play(vec![Component::Spaceship {
                    cards: spaceship_cards,
                    quad_count: 2,
                    top_strength: 10,
                }]),
                ShengjiPlayPresentationKind::Spaceship,
            ),
            (
                play(vec![
                    Component::Single {
                        card: single_card,
                        strength: 1,
                    },
                    Component::Pair {
                        cards: pair_cards,
                        strength: 2,
                    },
                ]),
                ShengjiPlayPresentationKind::Throw,
            ),
        ];
        for (classified, expected) in cases {
            assert_eq!(classify_play_presentation(&classified), expected);
        }
    }

    #[test]
    fn rare_play_effects_last_longer_than_routine_plays() {
        assert!(
            ShengjiPlayPresentationKind::Spaceship.duration()
                > ShengjiPlayPresentationKind::Pair.duration()
        );
        assert!(
            ShengjiPlayPresentationKind::Titanic.duration()
                > ShengjiPlayPresentationKind::Triple.duration()
        );
    }

    #[test]
    fn presentations_queue_instead_of_overwriting_a_rule_effect() {
        let mut state = ShengjiPresentationState::default();
        assert_eq!(
            state.activate(
                ShengjiPresentationKind::PowerOutage {
                    from_dealer: Some(PlayerId(0)),
                    dealer: PlayerId(1),
                    level: leocard_shengji::Rank::Three,
                },
                1.2,
            ),
            0.0
        );
        assert_eq!(
            state.activate(
                ShengjiPresentationKind::Declaration {
                    player: PlayerId(1),
                    trump: leocard_shengji::BidTrump::Suit(ShengjiSuit::Spade),
                    label: "亮主",
                },
                0.7,
            ),
            1.2
        );
        assert!(matches!(
            state.active.as_ref().map(|active| &active.kind),
            Some(ShengjiPresentationKind::PowerOutage { .. })
        ));
        assert!(matches!(
            state.queued.front().map(|active| &active.kind),
            Some(ShengjiPresentationKind::Declaration { .. })
        ));
    }

    #[test]
    fn routine_cards_and_following_mixed_shapes_do_not_show_type_labels() {
        assert!(!should_show_play_presentation(
            ShengjiPlayPresentationKind::Single,
            true,
            0,
        ));
        assert!(!should_show_play_presentation(
            ShengjiPlayPresentationKind::Pair,
            true,
            0,
        ));
        assert!(!should_show_play_presentation(
            ShengjiPlayPresentationKind::Triple,
            true,
            0,
        ));
        assert!(!should_show_play_presentation(
            ShengjiPlayPresentationKind::Throw,
            false,
            0,
        ));
        assert!(should_show_play_presentation(
            ShengjiPlayPresentationKind::Throw,
            true,
            0,
        ));
        assert!(should_show_play_presentation(
            ShengjiPlayPresentationKind::Tractor,
            false,
            0,
        ));
        assert!(!should_show_play_presentation(
            ShengjiPlayPresentationKind::Throw,
            true,
            5,
        ));
    }

    #[test]
    fn only_a_structure_matching_winning_trump_play_triggers_the_target_effect() {
        use leocard_shengji::{Category, ClassifiedPlay, Component};

        let trump =
            ShengjiTrump::new(leocard_shengji::Rank::Ten, Some(ShengjiSuit::Heart)).unwrap();
        let single = |player: u8, card: ShengjiCard, category| ShengjiPublicPlay {
            player: PlayerId(player),
            play: ClassifiedPlay {
                cards: vec![card],
                category,
                components: vec![Component::Single {
                    card,
                    strength: trump.strength(card),
                }],
            },
            throw_penalty: 0,
        };
        let lead = single(
            0,
            ShengjiCard::suited(0, ShengjiSuit::Spade, leocard_shengji::Rank::Ace),
            Category::Suit(ShengjiSuit::Spade),
        );
        let first_kill = single(
            1,
            ShengjiCard::suited(0, ShengjiSuit::Heart, leocard_shengji::Rank::Three),
            Category::Trump,
        );
        let cover_kill = single(
            2,
            ShengjiCard::suited(0, ShengjiSuit::Heart, leocard_shengji::Rank::Ace),
            Category::Trump,
        );
        let losing_trump = single(
            3,
            ShengjiCard::suited(1, ShengjiSuit::Heart, leocard_shengji::Rank::Four),
            Category::Trump,
        );

        let mut state = ShengjiPresentationState::default();
        state.current_trick_plays.push(lead);
        assert_eq!(
            state.winning_trump_kill(&first_kill, false, Some(trump)),
            Some(false)
        );
        state.current_trick_plays.push(first_kill);
        assert_eq!(
            state.winning_trump_kill(&cover_kill, false, Some(trump)),
            Some(true)
        );
        state.current_trick_plays.push(cover_kill);
        assert_eq!(
            state.winning_trump_kill(&losing_trump, false, Some(trump)),
            None
        );

        let pair_cards = [
            ShengjiCard::suited(0, ShengjiSuit::Spade, leocard_shengji::Rank::Nine),
            ShengjiCard::suited(1, ShengjiSuit::Spade, leocard_shengji::Rank::Nine),
        ];
        let pair_lead = ShengjiPublicPlay {
            player: PlayerId(0),
            play: ClassifiedPlay {
                cards: pair_cards.to_vec(),
                category: Category::Suit(ShengjiSuit::Spade),
                components: vec![Component::Pair {
                    cards: pair_cards,
                    strength: trump.strength(pair_cards[0]),
                }],
            },
            throw_penalty: 0,
        };
        let discard_cards = [
            ShengjiCard::suited(0, ShengjiSuit::Heart, leocard_shengji::Rank::Three),
            ShengjiCard::suited(0, ShengjiSuit::Heart, leocard_shengji::Rank::Four),
        ];
        let structure_mismatch = ShengjiPublicPlay {
            player: PlayerId(1),
            play: ClassifiedPlay {
                cards: discard_cards.to_vec(),
                category: Category::Trump,
                components: discard_cards
                    .into_iter()
                    .map(|card| Component::Single {
                        card,
                        strength: trump.strength(card),
                    })
                    .collect(),
            },
            throw_penalty: 0,
        };
        state.current_trick_plays = vec![pair_lead];
        assert_eq!(
            state.winning_trump_kill(&structure_mismatch, false, Some(trump)),
            None
        );
    }

    #[test]
    fn previous_trick_becomes_replayable_only_after_all_four_public_plays_finish() {
        let mut state = ShengjiPresentationState::default();
        for player in 0..4 {
            let card =
                ShengjiCard::suited(player, ShengjiSuit::Spade, leocard_shengji::Rank::Three);
            state.observe_trick_event(&leocard_protocol::ShengjiEvent::CardsPlayed {
                play: ShengjiPublicPlay {
                    player: PlayerId(player),
                    play: leocard_shengji::ClassifiedPlay {
                        cards: vec![card],
                        category: leocard_shengji::Category::Suit(ShengjiSuit::Spade),
                        components: vec![leocard_shengji::Component::Single { card, strength: 1 }],
                    },
                    throw_penalty: 0,
                },
                is_lead: player == 0,
            });
            assert!(!state.has_previous_trick());
        }
        state.observe_trick_event(&leocard_protocol::ShengjiEvent::TrickFinished {
            winner: PlayerId(0),
            points: 0,
            collecting_score: 0,
        });

        assert!(state.has_previous_trick());
        assert!(state.revealed_previous_trick().is_none());
        state.reveal_previous_trick();
        assert_eq!(state.previous_trick_reveal_remaining, 2.0);
        assert_eq!(state.revealed_previous_trick().unwrap().len(), 4);
    }
}
