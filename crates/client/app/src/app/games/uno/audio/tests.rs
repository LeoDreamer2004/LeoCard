use super::*;
use crate::app::games::GameFeedback;
use crate::app::games::uno::{UNO_PLAY_CARD_DURATION, uno_should_show_reverse_effect};
use leocard_game_common::PlayerId;
use leocard_protocol::UnoEvent;
use leocard_uno::{UnoCard, UnoChallengeResult, UnoColor, UnoFace};

fn played(face: UnoFace, count: u8, index: u8) -> UnoEvent {
    UnoEvent::CardPlayed {
        player: PlayerId(1),
        card: match face {
            UnoFace::Number(value) => UnoCard::number(UnoColor::Red, value, index),
            UnoFace::Wild | UnoFace::WildStackNumber => UnoCard::wild(face, index),
            _ => UnoCard::action(UnoColor::Red, face, index),
        },
        chosen_color: None,
        play_index: index,
        play_count: count,
    }
}

#[test]
fn played_cards_have_separate_throw_and_landing_beats() {
    let cues = played(UnoFace::Number(7), 1, 0).feedback((PlayerId(0), 10));
    assert_eq!(cues[0].kind, UnoSoundKind::CardThrow);
    assert_eq!(cues[1].kind, UnoSoundKind::CardLand);
    assert_eq!(cues[1].remaining, UNO_PLAY_CARD_DURATION);
}

#[test]
fn dense_penalties_cap_draw_sounds_at_four() {
    let cues = UnoEvent::CardsDrawn {
        player: PlayerId(1),
        count: 16,
        penalty: true,
        card_backs: Vec::new(),
    }
    .feedback((PlayerId(0), 10));
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
    let called = UnoEvent::UnoCalled {
        player: PlayerId(0),
    }
    .feedback((PlayerId(0), 1));
    let played = played(UnoFace::Number(3), 1, 0).feedback((PlayerId(0), 2));
    assert!(called[0].volume > played[0].volume);
}

#[test]
fn special_rules_keep_distinct_semantic_cues() {
    let reverse = played(UnoFace::Reverse, 1, 0).feedback((PlayerId(0), 1));
    assert!(reverse.iter().any(|cue| cue.kind == UnoSoundKind::Reverse));
    let skip = played(UnoFace::Skip, 1, 0).feedback((PlayerId(0), 2));
    assert!(skip.iter().any(|cue| cue.kind == UnoSoundKind::Skip));

    let palette = UnoEvent::ColorChosen {
        player: PlayerId(1),
        color: UnoColor::Blue,
    }
    .feedback((PlayerId(0), 3));
    assert_eq!(
        palette.iter().map(|cue| cue.kind).collect::<Vec<_>>(),
        [UnoSoundKind::PaletteOpen, UnoSoundKind::PaletteSelect]
    );

    let challenge = UnoEvent::ChallengeResolved {
        challenger: PlayerId(0),
        offender: PlayerId(1),
        result: UnoChallengeResult::Successful,
        penalized: PlayerId(1),
        count: 4,
        card_backs: Vec::new(),
    }
    .feedback((PlayerId(0), 4));
    assert_eq!(challenge[0].kind, UnoSoundKind::Challenge);
    assert_eq!(challenge[1].kind, UnoSoundKind::Success);
    assert_eq!(challenge[2].remaining, 0.18);

    let report = UnoEvent::UnoReported {
        reporter: PlayerId(0),
        target: PlayerId(1),
        card_backs: Vec::new(),
    }
    .feedback((PlayerId(0), 5));
    assert_eq!(
        report.iter().map(|cue| cue.kind).collect::<Vec<_>>(),
        [UnoSoundKind::Report, UnoSoundKind::Penalty]
    );
}

#[test]
fn simultaneous_pairs_share_audio_and_cancel_double_reverse_feedback() {
    let first = played(UnoFace::Reverse, 2, 0);
    let second = played(UnoFace::Reverse, 2, 1);
    let cues = first.feedback((PlayerId(0), 1));
    assert_eq!(
        cues.iter().map(|cue| cue.kind).collect::<Vec<_>>(),
        [UnoSoundKind::CardThrow, UnoSoundKind::CardLand]
    );
    assert!(second.feedback((PlayerId(0), 2)).is_empty());
    assert!(!uno_should_show_reverse_effect(
        UnoCard::action(UnoColor::Red, UnoFace::Reverse, 0),
        0,
        2,
    ));

    for (face, semantic) in [
        (UnoFace::DrawTwo, UnoSoundKind::Penalty),
        (UnoFace::Skip, UnoSoundKind::Skip),
    ] {
        let cues = played(face, 2, 0).feedback((PlayerId(0), 3));
        assert_eq!(cues.iter().filter(|cue| cue.kind == semantic).count(), 1);
        assert!(played(face, 2, 1).feedback((PlayerId(0), 4)).is_empty());
    }
}
