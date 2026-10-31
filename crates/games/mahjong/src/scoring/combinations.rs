use super::patterns::{
    all_suited_in, has_mixed_shifted_chows, is_reversible, max_matching, set_contains_rank,
};
use super::{FanValues, Form, MahjongFan, ScoreContext, SetKind};
use crate::MahjongTileKind;

pub(super) fn collect_combination_fans(
    values: &mut FanValues,
    context: &ScoreContext<'_>,
    unique_wait: bool,
) {
    collect_high_value_fans(values, context);
    collect_medium_value_fans(values, context);
    collect_basic_fans(values, context);
    collect_relation_fans(values, context);
    collect_wait_fan(values, context, unique_wait);
}

fn collect_high_value_fans(values: &mut FanValues, context: &ScoreContext<'_>) {
    values.add(MahjongFan::QuadrupleChow, u8::from(context.quadruple_chow));
    values.add(
        MahjongFan::FourPureShiftedPungs,
        u8::from(context.four_shifted_pungs),
    );
    values.add(
        MahjongFan::FourPureShiftedChows,
        u8::from(context.four_shifted_chows),
    );
    values.add(MahjongFan::ThreeKongs, u8::from(context.kongs.len() == 3));
    values.add(
        MahjongFan::AllTerminalsAndHonors,
        u8::from(context.terminals_honors),
    );
    values.add(
        MahjongFan::SevenPairs,
        u8::from(matches!(context.form, Form::SevenPairs { shifted: false })),
    );
    values.add(
        MahjongFan::GreaterHonorsAndKnittedTiles,
        u8::from(matches!(
            context.form,
            Form::Knitted {
                greater: true,
                pair: None,
                ..
            }
        )),
    );
    values.add(MahjongFan::AllEvenPungs, u8::from(context.all_even_pungs));
    values.add(MahjongFan::FullFlush, u8::from(context.full_flush));
    values.add(
        MahjongFan::PureTripleChow,
        u8::from(context.pure_triple && !context.quadruple_chow),
    );
    values.add(
        MahjongFan::PureShiftedPungs,
        u8::from(context.pure_shifted_pungs && !context.four_shifted_pungs),
    );
    values.add(
        MahjongFan::UpperTiles,
        u8::from(all_suited_in(&context.all_tiles, 7, 9)),
    );
    values.add(
        MahjongFan::MiddleTiles,
        u8::from(all_suited_in(&context.all_tiles, 4, 6)),
    );
    values.add(
        MahjongFan::LowerTiles,
        u8::from(all_suited_in(&context.all_tiles, 1, 3)),
    );
}

fn collect_medium_value_fans(values: &mut FanValues, context: &ScoreContext<'_>) {
    values.add(MahjongFan::PureStraight, u8::from(context.pure_straight));
    values.add(
        MahjongFan::ThreeSuitedTerminalChows,
        u8::from(context.three_suited_terminal),
    );
    values.add(
        MahjongFan::PureShiftedChows,
        u8::from(context.pure_shifted_chows && !context.four_shifted_chows),
    );
    let all_fives = context.sets_pair.is_some_and(|(sets, pair)| {
        context.is_standard
            && matches!(pair, MahjongTileKind::Suited { rank: 5, .. })
            && sets.iter().all(|set| set_contains_rank(set, 5))
    });
    values.add(MahjongFan::AllFives, u8::from(all_fives));
    values.add(MahjongFan::TriplePung, u8::from(context.triple_pung));
    values.add(
        MahjongFan::ThreeConcealedPungs,
        u8::from(context.concealed_pungs == 3),
    );
    values.add(
        MahjongFan::LesserHonorsAndKnittedTiles,
        u8::from(matches!(
            context.form,
            Form::Knitted {
                greater: false,
                pair: None,
                ..
            }
        )),
    );
    values.add(
        MahjongFan::KnittedStraight,
        u8::from(matches!(context.form, Form::Knitted { straight: true, .. })),
    );
    values.add(
        MahjongFan::UpperFour,
        u8::from(all_suited_in(&context.all_tiles, 6, 9)),
    );
    values.add(
        MahjongFan::LowerFour,
        u8::from(all_suited_in(&context.all_tiles, 1, 4)),
    );
    values.add(
        MahjongFan::BigThreeWinds,
        u8::from(context.wind_pungs == 3 && !context.big_four && !context.little_four),
    );
    values.add(MahjongFan::MixedStraight, u8::from(context.mixed_straight));
    values.add(
        MahjongFan::ReversibleTiles,
        u8::from(context.all_tiles.iter().all(is_reversible)),
    );
    values.add(MahjongFan::MixedTripleChow, u8::from(context.mixed_triple));
    values.add(
        MahjongFan::MixedShiftedPungs,
        u8::from(context.mixed_shifted_pungs),
    );
}

fn collect_basic_fans(values: &mut FanValues, context: &ScoreContext<'_>) {
    values.add(MahjongFan::AllPungs, u8::from(context.all_pungs));
    values.add(
        MahjongFan::HalfFlush,
        u8::from(context.suit_count == 1 && context.has_honor),
    );
    values.add(
        MahjongFan::MixedShiftedChows,
        u8::from(has_mixed_shifted_chows(&context.chows)),
    );
    values.add(MahjongFan::AllTypes, u8::from(context.all_types));
    values.add(MahjongFan::MeldedHand, u8::from(context.melded_hand));
    values.add(
        MahjongFan::TwoDragonPungs,
        u8::from(context.dragon_pungs == 2 && !context.big_three_dragons && !context.little_three),
    );
    values.add(MahjongFan::OutsideHand, u8::from(context.outside));
    values.add(
        MahjongFan::FullyConcealedHand,
        u8::from(context.fully_concealed),
    );
    if !context.big_three_dragons && !context.little_three && context.dragon_pungs == 1 {
        values.add(MahjongFan::DragonPung, 1);
    }
    if !context.big_four {
        if context
            .pungs
            .contains(&MahjongTileKind::Wind(context.input.context.prevalent_wind))
        {
            values.add(MahjongFan::PrevalentWind, 1);
        }
        if context
            .pungs
            .contains(&MahjongTileKind::Wind(context.input.context.seat_wind))
        {
            values.add(MahjongFan::SeatWind, 1);
        }
    }
    values.add(MahjongFan::ConcealedHand, u8::from(context.concealed_hand));
    values.add(MahjongFan::AllChows, u8::from(context.all_chows));
    if !context.quadruple_chow {
        values.add(MahjongFan::TileHog, context.tile_hogs);
    }
    if !context.triple_pung {
        values.add(MahjongFan::DoublePung, context.double_pungs);
    }
    values.add(
        MahjongFan::TwoConcealedPungs,
        u8::from(context.concealed_pungs == 2),
    );
    values.add(MahjongFan::AllSimples, u8::from(context.all_simples));
    collect_terminal_and_suit_fans(values, context);
}

fn collect_terminal_and_suit_fans(values: &mut FanValues, context: &ScoreContext<'_>) {
    if !context.big_four
        && !context.all_honors
        && !context.all_terminals
        && !context.terminals_honors
    {
        let terminal_honor_pungs = context
            .pungs
            .iter()
            .filter(|tile| match tile {
                MahjongTileKind::Suited { rank: 1 | 9, .. } => true,
                MahjongTileKind::Wind(wind) if context.wind_pungs < 3 => {
                    *wind != context.input.context.prevalent_wind
                        && *wind != context.input.context.seat_wind
                }
                _ => false,
            })
            .count() as u8;
        values.add(MahjongFan::PungOfTerminalsOrHonors, terminal_honor_pungs);
    }
    if !context.full_flush && !context.pure_terminal_chows {
        values.add(MahjongFan::OneVoidedSuit, u8::from(context.suit_count <= 2));
    }
    if !context.full_flush
        && !context.all_chows
        && !values.contains(MahjongFan::UpperTiles)
        && !values.contains(MahjongFan::LowerTiles)
        && !values.contains(MahjongFan::UpperFour)
        && !values.contains(MahjongFan::LowerFour)
        && !context.three_suited_terminal
    {
        values.add(MahjongFan::NoHonors, u8::from(!context.has_honor));
    }
}

fn collect_relation_fans(values: &mut FanValues, context: &ScoreContext<'_>) {
    if context.quadruple_chow
        || context.pure_terminal_chows
        || context.four_shifted_chows
        || context.three_suited_terminal
    {
        return;
    }
    let mut pure_double = max_matching(&context.chows, |left, right| left == right);
    let mut mixed_double = max_matching(&context.chows, |left, right| {
        left.0 != right.0 && left.1 == right.1
    });
    if context.pure_triple {
        pure_double = 0;
        mixed_double = mixed_double.min(1);
    }
    if context.mixed_triple {
        pure_double = pure_double.min(1);
        mixed_double = 0;
    }
    if context.three_suited_terminal {
        mixed_double = 0;
    }
    let short_straight = if context.pure_straight {
        0
    } else {
        max_matching(&context.chows, |left, right| {
            left.0 == right.0 && left.1.abs_diff(right.1) == 3
        })
    };
    let terminal_chows = if context.pure_straight {
        0
    } else {
        max_matching(&context.chows, |left, right| {
            left.0 == right.0 && [left.1, right.1].contains(&1) && [left.1, right.1].contains(&7)
        })
    };
    let three_chow_fan = context.pure_triple
        || context.pure_straight
        || context.pure_shifted_chows
        || context.mixed_straight
        || context.mixed_triple
        || has_mixed_shifted_chows(&context.chows);
    let mut remaining = if three_chow_fan {
        1
    } else {
        context.chows.len().saturating_sub(1) as u8
    };
    for (fan, count) in [
        (MahjongFan::PureDoubleChow, pure_double),
        (MahjongFan::MixedDoubleChow, mixed_double),
        (MahjongFan::ShortStraight, short_straight),
        (MahjongFan::TwoTerminalChows, terminal_chows),
    ] {
        let awarded = count.min(remaining);
        values.add(fan, awarded);
        remaining -= awarded;
    }
}

fn collect_wait_fan(values: &mut FanValues, context: &ScoreContext<'_>, unique_wait: bool) {
    if !unique_wait {
        return;
    }
    let Some((sets, pair)) = context.sets_pair else {
        return;
    };
    let single =
        pair == context.input.winning_tile && !context.melded_hand && context.kongs.len() < 4;
    let mut edge = false;
    let mut closed = false;
    for set in sets {
        if let SetKind::Chow { suit, start } = set.kind
            && let MahjongTileKind::Suited {
                suit: win_suit,
                rank,
            } = context.input.winning_tile
            && suit == win_suit
        {
            edge |= (start == 1 && rank == 3) || (start == 7 && rank == 7);
            closed |= rank == start + 1;
        }
    }
    if single {
        values.add(MahjongFan::SingleWait, 1);
    } else if edge {
        values.add(MahjongFan::EdgeWait, 1);
    } else if closed {
        values.add(MahjongFan::ClosedWait, 1);
    }
}
