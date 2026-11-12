use super::super::major::MajorShowcaseSequence;
use super::super::timeline::WinShowcaseTiming;
use super::stage::{ShowcaseAudioStage, ShowcaseClip};
use leocard_protocol::MatchId;
use std::collections::HashSet;

fn stage(glyphs: usize) -> ShowcaseAudioStage {
    ShowcaseAudioStage::major(
        (MatchId([1; 16]), 0),
        WinShowcaseTiming {
            reveal_duration: 8.0,
            start: 1.0,
            duration: 7.0,
        },
        glyphs,
    )
}

#[test]
fn impacts_follow_tile_contact_and_the_actual_number_of_landing_glyphs() {
    for glyphs in 2..=5 {
        let stage = stage(glyphs);
        let mut played = HashSet::new();
        assert_eq!(
            stage.due(0.0, &mut played)[0].0.clip,
            ShowcaseClip::Background
        );
        let contact =
            MajorShowcaseSequence::WINNING_TILE_START + MajorShowcaseSequence::TILE_FLIGHT;
        assert_eq!(
            stage.due(contact, &mut played)[0].0.clip,
            ShowcaseClip::Tiles
        );
        for index in 0..glyphs {
            let contact = MajorShowcaseSequence::FIRST_GLYPH
                + index as f32 * MajorShowcaseSequence::GLYPH_INTERVAL
                + MajorShowcaseSequence::GLYPH_FALL;
            let cues = stage.due(contact, &mut played);
            assert_eq!(cues.len(), 1);
            assert_eq!(cues[0].0.clip, ShowcaseClip::Glyph);
        }
        assert_eq!(played.len(), 2 + glyphs);
    }
}

#[test]
fn late_frames_resume_background_without_replaying_impacts_or_rebuilt_stages() {
    let mut played = HashSet::new();
    let cues = stage(3).due(4.0, &mut played);
    assert_eq!(cues.len(), 1);
    assert_eq!(cues[0].0.clip, ShowcaseClip::Background);
    assert_eq!(cues[0].1, 4.0);
    assert!(stage(3).due(4.01, &mut played).is_empty());
    assert!(stage(3).due(7.0, &mut played).is_empty());
}
