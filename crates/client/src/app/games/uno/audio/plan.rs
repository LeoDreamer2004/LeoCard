use super::*;
use crate::app::games::uno::UNO_PLAY_CARD_DURATION;
use leocard_protocol::{PlayerId, UnoEvent};
use leocard_uno::{UnoChallengeResult, UnoColor, UnoFace};

pub(super) fn uno_event_sound_plan(event: &UnoEvent, you: PlayerId, seed: u64) -> Vec<UnoAudioCue> {
    match event {
        UnoEvent::CardPlayed {
            player,
            card,
            chosen_color,
            play_index,
            play_count,
        } => card_played_plan(
            CardPlayedSound {
                player: *player,
                face: card.face(),
                chosen_color: *chosen_color,
                play_index: *play_index,
                play_count: *play_count,
            },
            you,
            seed,
        ),
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
        } => draw_plan(1, 0.02, true, seed),
        UnoEvent::ColorChosen { color, .. } => palette_plan(*color, seed),
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
            let mut cues = draw_plan(cards.len() as u16, delay, false, seed);
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
            let mut cues = palette_plan(*color, seed);
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

struct CardPlayedSound {
    player: PlayerId,
    face: UnoFace,
    chosen_color: Option<UnoColor>,
    play_index: u8,
    play_count: u8,
}

fn card_played_plan(played: CardPlayedSound, you: PlayerId, seed: u64) -> Vec<UnoAudioCue> {
    let CardPlayedSound {
        player,
        face,
        chosen_color,
        play_index,
        play_count,
    } = played;
    if play_index > 0 {
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
    match face {
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
        | UnoFace::WildStackNumber => {
            cues.push(UnoAudioCue::new(
                UnoSoundKind::Penalty,
                0.10,
                0.43,
                seed + 2,
            ));
        }
        UnoFace::Flip
        | UnoFace::Number(_)
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
        append_palette_cues(&mut cues, color, seed + 4);
    }
    if player == you {
        cues[0].volume = 0.33;
    }
    cues
}

fn draw_plan(count: u16, delay: f32, penalty: bool, seed: u64) -> Vec<UnoAudioCue> {
    let mut cues = Vec::new();
    append_draw_cues(&mut cues, count, delay, penalty, seed);
    cues
}

fn palette_plan(color: UnoColor, seed: u64) -> Vec<UnoAudioCue> {
    let mut cues = Vec::new();
    append_palette_cues(&mut cues, color, seed);
    cues
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
