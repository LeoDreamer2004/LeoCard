//! UNO 事件驱动音效。
//!
//! 声音与权威事件和视觉演出共用同一消费时点，避免按钮先响、服务器随后拒绝，
//! 也避免 UI 重建重复播放。多张罚牌只保留最多四个均匀分布的抽牌声。

use super::*;

#[derive(Default)]
pub struct UnoSoundAssets {
    select: Vec<Handle<AudioSource>>,
    card_throw: Vec<Handle<AudioSource>>,
    card_land: Vec<Handle<AudioSource>>,
    card_draw: Vec<Handle<AudioSource>>,
    penalty: Vec<Handle<AudioSource>>,
    skip: Vec<Handle<AudioSource>>,
    reverse: Vec<Handle<AudioSource>>,
    palette_open: Vec<Handle<AudioSource>>,
    palette_select: Vec<Handle<AudioSource>>,
    uno_call: Vec<Handle<AudioSource>>,
    uno_accent: Vec<Handle<AudioSource>>,
    report: Vec<Handle<AudioSource>>,
    challenge: Vec<Handle<AudioSource>>,
    success: Vec<Handle<AudioSource>>,
    failure: Vec<Handle<AudioSource>>,
}

impl UnoSoundAssets {
    pub fn load(asset_server: &AssetServer) -> Self {
        let casino = "vendor/kenney/casino-audio/Audio";
        let interface = "vendor/kenney/interface-sounds/Audio";
        Self {
            select: numbered_sounds(asset_server, interface, "select_00", 8),
            card_throw: numbered_sounds(asset_server, casino, "card-shove-", 4),
            card_land: numbered_sounds(asset_server, casino, "card-place-", 4),
            card_draw: numbered_sounds(asset_server, casino, "card-slide-", 8),
            penalty: vec![asset_server.load(format!("{interface}/bong_001.ogg"))],
            skip: numbered_sounds(asset_server, interface, "close_00", 4),
            reverse: numbered_sounds(asset_server, interface, "scroll_00", 5),
            palette_open: numbered_sounds(asset_server, interface, "glass_00", 6),
            palette_select: numbered_sounds(asset_server, interface, "pluck_00", 2),
            uno_call: numbered_sounds(asset_server, interface, "confirmation_00", 4),
            uno_accent: numbered_sounds(asset_server, interface, "maximize_00", 9),
            report: numbered_sounds(asset_server, interface, "error_00", 8),
            challenge: numbered_sounds(asset_server, interface, "question_00", 4),
            success: numbered_sounds(asset_server, interface, "confirmation_00", 4),
            failure: numbered_sounds(asset_server, interface, "error_00", 8),
        }
    }

    fn variants(&self, kind: UnoSoundKind) -> &[Handle<AudioSource>] {
        match kind {
            UnoSoundKind::CardThrow => &self.card_throw,
            UnoSoundKind::CardLand => &self.card_land,
            UnoSoundKind::CardDraw => &self.card_draw,
            UnoSoundKind::Penalty => &self.penalty,
            UnoSoundKind::Skip => &self.skip,
            UnoSoundKind::Reverse => &self.reverse,
            UnoSoundKind::PaletteOpen => &self.palette_open,
            UnoSoundKind::PaletteSelect => &self.palette_select,
            UnoSoundKind::UnoCall => &self.uno_call,
            UnoSoundKind::UnoAccent => &self.uno_accent,
            UnoSoundKind::Report => &self.report,
            UnoSoundKind::Challenge => &self.challenge,
            UnoSoundKind::Success => &self.success,
            UnoSoundKind::Failure => &self.failure,
        }
    }
}

fn numbered_sounds(
    asset_server: &AssetServer,
    directory: &str,
    prefix: &str,
    count: usize,
) -> Vec<Handle<AudioSource>> {
    (1..=count)
        .map(|index| asset_server.load(format!("{directory}/{prefix}{index}.ogg")))
        .collect()
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum UnoSoundKind {
    CardThrow,
    CardLand,
    CardDraw,
    Penalty,
    Skip,
    Reverse,
    PaletteOpen,
    PaletteSelect,
    UnoCall,
    UnoAccent,
    Report,
    Challenge,
    Success,
    Failure,
}

impl UnoSoundKind {
    fn is_card(self) -> bool {
        matches!(self, Self::CardThrow | Self::CardLand | Self::CardDraw)
    }

    fn is_prominent(self) -> bool {
        matches!(
            self,
            Self::UnoCall
                | Self::UnoAccent
                | Self::Report
                | Self::Challenge
                | Self::Success
                | Self::Failure
        )
    }
}

#[derive(Clone, Copy, Debug)]
struct UnoAudioCue {
    pub kind: UnoSoundKind,
    pub remaining: f32,
    pub volume: f32,
    pub speed: f32,
    pub seed: u64,
}

impl UnoAudioCue {
    fn new(kind: UnoSoundKind, remaining: f32, volume: f32, seed: u64) -> Self {
        Self {
            kind,
            remaining,
            volume,
            speed: 1.0,
            seed,
        }
    }

    fn with_speed(mut self, speed: f32) -> Self {
        self.speed = speed;
        self
    }
}

#[derive(Resource, Default)]
pub struct UnoAudioState {
    cues: Vec<UnoAudioCue>,
    serial: u64,
    duck_remaining: f32,
}

impl UnoAudioState {
    pub fn queue_event(&mut self, event: &UnoEvent, you: PlayerId) {
        self.serial = self.serial.wrapping_add(1);
        self.cues.extend(uno_event_sound_plan(
            event,
            you,
            self.serial.wrapping_mul(37),
        ));
    }
}

fn uno_event_sound_plan(event: &UnoEvent, you: PlayerId, seed: u64) -> Vec<UnoAudioCue> {
    match event {
        UnoEvent::CardPlayed {
            player,
            card,
            chosen_color,
            play_index,
            play_count,
        } => {
            if *play_index > 0 {
                return Vec::new();
            }
            let mut cues = vec![
                UnoAudioCue::new(UnoSoundKind::CardThrow, 0.0, 0.28, seed),
                UnoAudioCue::new(
                    UnoSoundKind::CardLand,
                    UNO_PLAY_CARD_DURATION,
                    0.43,
                    seed + 1,
                ),
            ];
            match card.face() {
                UnoFace::Reverse if play_count % 2 == 1 => {
                    cues.push(UnoAudioCue::new(
                        UnoSoundKind::Reverse,
                        0.06,
                        0.48,
                        seed + 2,
                    ));
                }
                UnoFace::Reverse => {}
                UnoFace::ReverseDrawTwo | UnoFace::WildReverseDrawFour => {
                    cues.push(UnoAudioCue::new(
                        UnoSoundKind::Reverse,
                        0.06,
                        0.48,
                        seed + 2,
                    ));
                    cues.push(UnoAudioCue::new(
                        UnoSoundKind::Penalty,
                        0.10,
                        0.43,
                        seed + 3,
                    ));
                }
                UnoFace::ReverseSkip => {
                    cues.push(UnoAudioCue::new(
                        UnoSoundKind::Reverse,
                        0.06,
                        0.48,
                        seed + 2,
                    ));
                    cues.push(UnoAudioCue::new(
                        UnoSoundKind::Skip,
                        UNO_PLAY_CARD_DURATION * 0.72,
                        0.50,
                        seed + 3,
                    ));
                }
                UnoFace::WildPowerReverse | UnoFace::WildNoU => {
                    cues.push(UnoAudioCue::new(
                        UnoSoundKind::Reverse,
                        0.06,
                        0.48,
                        seed + 2,
                    ));
                }
                UnoFace::Skip | UnoFace::SkipEveryone => cues.push(UnoAudioCue::new(
                    UnoSoundKind::Skip,
                    UNO_PLAY_CARD_DURATION * 0.72,
                    0.50,
                    seed + 2,
                )),
                UnoFace::DrawTwo
                | UnoFace::DrawOne
                | UnoFace::DrawFour
                | UnoFace::DrawFive
                | UnoFace::WildDrawTwo
                | UnoFace::WildDrawFour
                | UnoFace::WildDrawColor
                | UnoFace::WildDrawSix
                | UnoFace::WildDrawTen
                | UnoFace::StackOne
                | UnoFace::StackTwo
                | UnoFace::WildStackThree
                | UnoFace::WildStackNumber => cues.push(UnoAudioCue::new(
                    UnoSoundKind::Penalty,
                    0.10,
                    0.43,
                    seed + 2,
                )),
                UnoFace::Flip => {}
                UnoFace::Number(_)
                | UnoFace::Wild
                | UnoFace::DarkWild
                | UnoFace::SwapOne
                | UnoFace::RefreshHand
                | UnoFace::WildForceTrade
                | UnoFace::WildPassHands
                | UnoFace::DiscardAll
                | UnoFace::WildColorRoulette => {}
            }
            if let Some(color) = chosen_color {
                append_palette_cues(&mut cues, *color, seed + 4);
            }
            if *player == you {
                cues[0].volume = 0.33;
            }
            cues
        }
        UnoEvent::CardsDrawn { count, penalty, .. } => {
            let mut cues = Vec::new();
            if *penalty {
                cues.push(UnoAudioCue::new(UnoSoundKind::Penalty, 0.0, 0.48, seed));
            }
            append_draw_cues_with_interval(
                &mut cues,
                *count,
                0.0,
                *penalty,
                if !penalty && *count > 1 { 0.18 } else { 0.045 },
                seed + 1,
            );
            cues
        }
        UnoEvent::DrawPenaltyReflected { count, .. } => {
            let mut cues = vec![UnoAudioCue::new(
                UnoSoundKind::Penalty,
                UNO_PLAY_CARD_DURATION * 0.72,
                0.56,
                seed,
            )];
            append_draw_cues(
                &mut cues,
                *count,
                UNO_PLAY_CARD_DURATION * 0.72,
                true,
                seed + 1,
            );
            cues
        }
        UnoEvent::ChallengeResolved { result, count, .. } => {
            let mut cues = vec![UnoAudioCue::new(UnoSoundKind::Challenge, 0.0, 0.58, seed)];
            cues.push(UnoAudioCue::new(
                match result {
                    UnoChallengeResult::Successful => UnoSoundKind::Success,
                    UnoChallengeResult::Failed => UnoSoundKind::Failure,
                },
                0.12,
                0.66,
                seed + 1,
            ));
            append_draw_cues(&mut cues, *count, 0.18, true, seed + 2);
            cues
        }
        UnoEvent::UnoCalled { player } => vec![
            UnoAudioCue::new(
                UnoSoundKind::UnoCall,
                0.0,
                if *player == you { 0.78 } else { 0.68 },
                seed,
            ),
            UnoAudioCue::new(UnoSoundKind::UnoAccent, 0.13, 0.48, seed + 1).with_speed(1.08),
        ],
        UnoEvent::UnoReported { target, .. } => vec![
            UnoAudioCue::new(
                UnoSoundKind::Report,
                0.0,
                if *target == you { 0.80 } else { 0.68 },
                seed,
            ),
            UnoAudioCue::new(UnoSoundKind::Penalty, 0.10, 0.46, seed + 1),
        ],
        UnoEvent::SkipResolved {
            drew_card: true, ..
        } => {
            let mut cues = Vec::new();
            append_draw_cues(&mut cues, 1, 0.02, true, seed);
            cues
        }
        UnoEvent::ColorChosen { color, .. } => {
            let mut cues = Vec::new();
            append_palette_cues(&mut cues, *color, seed);
            cues
        }
        UnoEvent::HandRefreshed { count, .. } => {
            let delay = UNO_PLAY_CARD_DURATION * 0.72;
            let mut cues = vec![UnoAudioCue::new(UnoSoundKind::CardThrow, delay, 0.34, seed)];
            append_draw_cues(&mut cues, (*count).min(6), delay + 0.34, false, seed + 1);
            cues
        }
        UnoEvent::SwapOneCardTaken { .. } | UnoEvent::SwapOneCompleted { .. } => vec![
            UnoAudioCue::new(UnoSoundKind::CardThrow, 0.0, 0.30, seed),
            UnoAudioCue::new(UnoSoundKind::CardLand, 0.70, 0.36, seed + 1),
        ],
        UnoEvent::HandsTraded { .. } => vec![
            UnoAudioCue::new(UnoSoundKind::CardThrow, 0.0, 0.38, seed),
            UnoAudioCue::new(UnoSoundKind::CardLand, 0.72, 0.42, seed + 1),
        ],
        UnoEvent::HandsPassed { .. } => {
            let delay = UNO_PLAY_CARD_DURATION * 0.72;
            vec![
                UnoAudioCue::new(UnoSoundKind::CardThrow, delay, 0.38, seed),
                UnoAudioCue::new(UnoSoundKind::CardLand, delay + 0.72, 0.42, seed + 1),
            ]
        }
        UnoEvent::StackNumberRevealed { cards, .. } => {
            let delay = UNO_PLAY_CARD_DURATION + 0.08;
            let mut cues = Vec::new();
            append_draw_cues(&mut cues, cards.len() as u16, delay, false, seed);
            cues.push(UnoAudioCue::new(
                UnoSoundKind::CardLand,
                delay + cards.len().saturating_sub(1) as f32 * 0.12 + 0.62,
                0.42,
                seed + 7,
            ));
            cues
        }
        UnoEvent::CardsDiscarded { cards, .. } => vec![
            UnoAudioCue::new(UnoSoundKind::CardThrow, 0.20, 0.40, seed),
            UnoAudioCue::new(
                UnoSoundKind::CardLand,
                0.48 + cards.len().min(8) as f32 * 0.045,
                0.44,
                seed + 1,
            ),
        ],
        UnoEvent::ColorRouletteResolved { color, count, .. } => {
            let mut cues = Vec::new();
            append_palette_cues(&mut cues, *color, seed);
            append_draw_cues_with_interval(&mut cues, *count, 0.22, true, 0.18, seed + 5);
            cues
        }
        UnoEvent::SkipResolved {
            drew_card: false, ..
        }
        | UnoEvent::GameFinished { .. } => Vec::new(),
        UnoEvent::Flipped { .. } => vec![
            UnoAudioCue::new(UnoSoundKind::Reverse, 0.02, 0.56, seed).with_speed(0.78),
            UnoAudioCue::new(UnoSoundKind::PaletteOpen, 0.34, 0.40, seed + 1).with_speed(0.86),
        ],
    }
}

fn append_draw_cues(
    cues: &mut Vec<UnoAudioCue>,
    count: u16,
    base_delay: f32,
    penalty: bool,
    seed: u64,
) {
    append_draw_cues_with_interval(cues, count, base_delay, penalty, 0.045, seed);
}

fn append_draw_cues_with_interval(
    cues: &mut Vec<UnoAudioCue>,
    count: u16,
    base_delay: f32,
    penalty: bool,
    interval: f32,
    seed: u64,
) {
    let visible = usize::from(count.min(16));
    let audible = visible.min(4);
    if audible == 0 {
        return;
    }
    for index in 0..audible {
        let visual_index = if audible == 1 {
            0
        } else {
            index * visible.saturating_sub(1) / audible.saturating_sub(1)
        };
        cues.push(UnoAudioCue::new(
            UnoSoundKind::CardDraw,
            base_delay + visual_index as f32 * interval,
            if penalty { 0.27 } else { 0.34 },
            seed + index as u64,
        ));
    }
}

fn append_palette_cues(cues: &mut Vec<UnoAudioCue>, color: UnoColor, seed: u64) {
    let color_index = match color {
        UnoColor::Red => 0,
        UnoColor::Yellow => 1,
        UnoColor::Green => 2,
        UnoColor::Blue => 3,
        UnoColor::Pink => 4,
        UnoColor::Teal => 5,
        UnoColor::Orange => 6,
        UnoColor::Purple => 7,
    };
    cues.push(
        UnoAudioCue::new(UnoSoundKind::PaletteOpen, 0.02, 0.37, seed + color_index)
            .with_speed(0.96),
    );
    cues.push(
        UnoAudioCue::new(UnoSoundKind::PaletteSelect, 0.40, 0.52, seed + color_index)
            .with_speed(0.90 + (color_index % 4) as f32 * 0.09),
    );
}

pub fn play_uno_card_selection_sounds(
    buttons: Query<&Interaction, (Changed<Interaction>, With<Button>, With<UnoHandCardButton>)>,
    assets: Res<UiAssets>,
    mut commands: Commands,
) {
    if assets.games.uno_sounds.select.is_empty() {
        return;
    }
    for interaction in &buttons {
        if !matches!(interaction, Interaction::Pressed) {
            continue;
        }
        let index = fastrand::usize(..assets.games.uno_sounds.select.len());
        commands.spawn((
            AudioPlayer::new(assets.games.uno_sounds.select[index].clone()),
            PlaybackSettings {
                volume: Volume::Linear(0.22),
                speed: 1.04,
                ..PlaybackSettings::DESPAWN
            },
        ));
    }
}

pub fn play_uno_audio_cues(
    time: Res<Time>,
    client: Option<Res<ClientResource>>,
    assets: Res<UiAssets>,
    mut state: ResMut<UnoAudioState>,
    mut commands: Commands,
) {
    if client
        .as_deref()
        .and_then(|client| client.0.model().uno_game())
        .is_none()
    {
        state.cues.clear();
        state.duck_remaining = 0.0;
        return;
    }

    state.duck_remaining = (state.duck_remaining - time.delta_secs()).max(0.0);
    let mut waiting = Vec::with_capacity(state.cues.len());
    let mut ready = Vec::new();
    for mut cue in std::mem::take(&mut state.cues) {
        cue.remaining -= time.delta_secs();
        if cue.remaining <= 0.0 {
            ready.push(cue);
        } else {
            waiting.push(cue);
        }
    }
    state.cues = waiting;

    if ready.iter().any(|cue| cue.kind.is_prominent()) {
        state.duck_remaining = 0.35;
    }
    for cue in ready {
        let variants = assets.games.uno_sounds.variants(cue.kind);
        if variants.is_empty() {
            continue;
        }
        let volume = if cue.kind.is_card() && state.duck_remaining > 0.0 {
            cue.volume * 0.55
        } else {
            cue.volume
        };
        commands.spawn((
            AudioPlayer::new(variants[cue.seed as usize % variants.len()].clone()),
            PlaybackSettings {
                volume: Volume::Linear(volume),
                speed: cue.speed,
                ..PlaybackSettings::DESPAWN
            },
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn played_cards_have_separate_throw_and_landing_beats() {
        let event = UnoEvent::CardPlayed {
            player: PlayerId(1),
            card: UnoCard::number(UnoColor::Red, 7, 0),
            chosen_color: None,
            play_index: 0,
            play_count: 1,
        };
        let cues = uno_event_sound_plan(&event, PlayerId(0), 10);
        assert_eq!(cues[0].kind, UnoSoundKind::CardThrow);
        assert_eq!(cues[1].kind, UnoSoundKind::CardLand);
        assert_eq!(cues[1].remaining, UNO_PLAY_CARD_DURATION);
    }

    #[test]
    fn dense_penalties_cap_draw_sounds_at_four() {
        let event = UnoEvent::CardsDrawn {
            player: PlayerId(1),
            count: 16,
            penalty: true,
            card_backs: Vec::new(),
        };
        let cues = uno_event_sound_plan(&event, PlayerId(0), 10);
        assert_eq!(
            cues.iter()
                .filter(|cue| cue.kind == UnoSoundKind::CardDraw)
                .count(),
            4
        );
        assert!(
            cues.windows(2)
                .all(|pair| pair[0].remaining <= pair[1].remaining)
        );
    }

    #[test]
    fn semantic_alerts_are_louder_than_card_texture() {
        let called = uno_event_sound_plan(
            &UnoEvent::UnoCalled {
                player: PlayerId(0),
            },
            PlayerId(0),
            1,
        );
        let played = uno_event_sound_plan(
            &UnoEvent::CardPlayed {
                player: PlayerId(0),
                card: UnoCard::number(UnoColor::Blue, 3, 0),
                chosen_color: None,
                play_index: 0,
                play_count: 1,
            },
            PlayerId(0),
            2,
        );
        assert!(called[0].volume > played[0].volume);
    }

    #[test]
    fn special_rules_keep_distinct_semantic_cues() {
        let reverse = uno_event_sound_plan(
            &UnoEvent::CardPlayed {
                player: PlayerId(1),
                card: UnoCard::action(UnoColor::Green, UnoFace::Reverse, 0),
                chosen_color: None,
                play_index: 0,
                play_count: 1,
            },
            PlayerId(0),
            1,
        );
        assert!(reverse.iter().any(|cue| cue.kind == UnoSoundKind::Reverse));

        let skip = uno_event_sound_plan(
            &UnoEvent::CardPlayed {
                player: PlayerId(1),
                card: UnoCard::action(UnoColor::Yellow, UnoFace::Skip, 0),
                chosen_color: None,
                play_index: 0,
                play_count: 1,
            },
            PlayerId(0),
            2,
        );
        assert!(skip.iter().any(|cue| cue.kind == UnoSoundKind::Skip));

        let palette = uno_event_sound_plan(
            &UnoEvent::ColorChosen {
                player: PlayerId(1),
                color: UnoColor::Blue,
            },
            PlayerId(0),
            3,
        );
        assert_eq!(
            palette.iter().map(|cue| cue.kind).collect::<Vec<_>>(),
            [UnoSoundKind::PaletteOpen, UnoSoundKind::PaletteSelect]
        );

        let challenge = uno_event_sound_plan(
            &UnoEvent::ChallengeResolved {
                challenger: PlayerId(0),
                offender: PlayerId(1),
                result: UnoChallengeResult::Successful,
                penalized: PlayerId(1),
                count: 4,
                card_backs: Vec::new(),
            },
            PlayerId(0),
            4,
        );
        assert_eq!(challenge[0].kind, UnoSoundKind::Challenge);
        assert_eq!(challenge[1].kind, UnoSoundKind::Success);
        assert_eq!(challenge[2].remaining, 0.18);

        let report = uno_event_sound_plan(
            &UnoEvent::UnoReported {
                reporter: PlayerId(0),
                target: PlayerId(1),
                card_backs: Vec::new(),
            },
            PlayerId(0),
            5,
        );
        assert_eq!(
            report.iter().map(|cue| cue.kind).collect::<Vec<_>>(),
            [UnoSoundKind::Report, UnoSoundKind::Penalty]
        );

        let stack = uno_event_sound_plan(
            &UnoEvent::CardPlayed {
                player: PlayerId(1),
                card: UnoCard::wild(UnoFace::WildStackNumber, 0),
                chosen_color: Some(UnoColor::Green),
                play_index: 0,
                play_count: 1,
            },
            PlayerId(0),
            6,
        );
        assert_eq!(
            stack
                .iter()
                .filter(|cue| cue.kind == UnoSoundKind::Penalty)
                .count(),
            1
        );
        assert!(
            stack
                .iter()
                .any(|cue| cue.kind == UnoSoundKind::PaletteSelect)
        );

        let revealed = uno_event_sound_plan(
            &UnoEvent::StackNumberRevealed {
                player: PlayerId(1),
                cards: vec![
                    UnoCard::action(UnoColor::Blue, UnoFace::Skip, 0),
                    UnoCard::number(UnoColor::Red, 8, 0),
                ],
                value: 8,
            },
            PlayerId(0),
            7,
        );
        assert_eq!(
            revealed
                .iter()
                .filter(|cue| cue.kind == UnoSoundKind::CardDraw)
                .count(),
            2
        );
        assert!(
            revealed
                .iter()
                .any(|cue| cue.kind == UnoSoundKind::CardLand)
        );
    }

    #[test]
    fn simultaneous_pairs_share_audio_and_cancel_double_reverse_feedback() {
        let reverse_first = UnoEvent::CardPlayed {
            player: PlayerId(1),
            card: UnoCard::action(UnoColor::Red, UnoFace::Reverse, 0),
            chosen_color: None,
            play_index: 0,
            play_count: 2,
        };
        let reverse_second = UnoEvent::CardPlayed {
            player: PlayerId(1),
            card: UnoCard::action(UnoColor::Red, UnoFace::Reverse, 1),
            chosen_color: None,
            play_index: 1,
            play_count: 2,
        };
        let first_cues = uno_event_sound_plan(&reverse_first, PlayerId(0), 1);
        assert_eq!(
            first_cues.iter().map(|cue| cue.kind).collect::<Vec<_>>(),
            [UnoSoundKind::CardThrow, UnoSoundKind::CardLand]
        );
        assert!(uno_event_sound_plan(&reverse_second, PlayerId(0), 2).is_empty());
        assert!(!uno_should_show_reverse_effect(
            UnoCard::action(UnoColor::Red, UnoFace::Reverse, 0),
            0,
            2,
        ));

        for face in [UnoFace::DrawTwo, UnoFace::Skip] {
            let first = UnoEvent::CardPlayed {
                player: PlayerId(1),
                card: UnoCard::action(UnoColor::Blue, face, 0),
                chosen_color: None,
                play_index: 0,
                play_count: 2,
            };
            let second = UnoEvent::CardPlayed {
                player: PlayerId(1),
                card: UnoCard::action(UnoColor::Blue, face, 1),
                chosen_color: None,
                play_index: 1,
                play_count: 2,
            };
            let first_cues = uno_event_sound_plan(&first, PlayerId(0), 3);
            let semantic = match face {
                UnoFace::DrawTwo => UnoSoundKind::Penalty,
                UnoFace::Skip => UnoSoundKind::Skip,
                _ => unreachable!(),
            };
            assert_eq!(
                first_cues.iter().filter(|cue| cue.kind == semantic).count(),
                1
            );
            assert!(uno_event_sound_plan(&second, PlayerId(0), 4).is_empty());
        }
    }
}
