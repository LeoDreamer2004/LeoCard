use super::combinations::collect_combination_fans;
use super::context::ScoreContext;
use super::finalize::{FanValues, finish_score};
use super::forms::{unique_wait, validated_forms};
use super::special::collect_special_fans;
use super::{Form, MahjongScoreResult, ScoreError, ScoreInput};
use crate::{MahjongTileKind, Meld};

pub fn is_complete_hand(concealed: &[MahjongTileKind], melds: &[Meld]) -> bool {
    validated_forms(concealed, melds, None).is_ok_and(|forms| !forms.is_empty())
}

/// Structural waits, excluding fifth copies already held in the hand or melds.
pub fn waiting_tile_kinds(concealed: &[MahjongTileKind], melds: &[Meld]) -> Vec<MahjongTileKind> {
    let mut before = concealed.to_vec();
    let mut wins = Vec::new();
    for index in 0..34 {
        let tile = MahjongTileKind::from_index34(index).expect("valid index");
        let used = before.iter().filter(|held| **held == tile).count()
            + melds
                .iter()
                .flat_map(|meld| meld.tile_kinds())
                .filter(|held| *held == tile)
                .count();
        if used < 4 {
            before.push(tile);
            if is_complete_hand(&before, melds) {
                wins.push(tile);
            }
            before.pop();
        }
    }
    wins
}

pub fn score_hand(input: &ScoreInput) -> Result<MahjongScoreResult, ScoreError> {
    let forms = validated_forms(&input.concealed, &input.melds, Some(input.winning_tile))?;
    if forms.is_empty() {
        return Err(ScoreError::NotComplete);
    }
    let unique_wait = unique_wait(&input.concealed, &input.melds, input.winning_tile);
    forms
        .iter()
        .map(|form| score_form(input, form, unique_wait))
        .max_by_key(|result| result.total_points)
        .ok_or(ScoreError::NotComplete)
}

fn score_form(input: &ScoreInput, form: &Form, unique_wait: bool) -> MahjongScoreResult {
    let context = ScoreContext::new(input, form);
    let mut values = FanValues::new();
    collect_special_fans(&mut values, &context);
    collect_combination_fans(&mut values, &context, unique_wait);
    finish_score(values, &context)
}
