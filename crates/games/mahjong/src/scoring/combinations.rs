use super::patterns::{
    all_suited_in, has_mixed_shifted_chows, is_reversible, max_matching, set_contains_rank,
};
use super::{Fan, FanValues, Form, ScoreContext, SetKind};
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
    values.add(Fan::QuadrupleChow, u8::from(context.quadruple_chow));
    values.add(
        Fan::FourPureShiftedPungs,
        u8::from(context.four_shifted_pungs),
    );
    values.add(
        Fan::FourPureShiftedChows,
        u8::from(context.four_shifted_chows),
    );
    values.add(Fan::ThreeKongs, u8::from(context.kongs.len() == 3));
    values.add(
        Fan::AllTerminalsAndHonors,
        u8::from(context.terminals_honors),
    );
    values.add(
        Fan::SevenPairs,
        u8::from(matches!(context.form, Form::SevenPairs { shifted: false })),
    );
    values.add(
        Fan::GreaterHonorsAndKnittedTiles,
        u8::from(matches!(
            context.form,
            Form::Knitted {
                greater: true,
                pair: None,
                ..
            }
        )),
    );
    values.add(Fan::AllEvenPungs, u8::from(context.all_even_pungs));
    values.add(Fan::FullFlush, u8::from(context.full_flush));
    values.add(
        Fan::PureTripleChow,
        u8::from(context.pure_triple && !context.quadruple_chow),
    );
    values.add(
        Fan::PureShiftedPungs,
        u8::from(context.pure_shifted_pungs && !context.four_shifted_pungs),
    );
    values.add(
        Fan::UpperTiles,
        u8::from(all_suited_in(&context.all_tiles, 7, 9)),
    );
    values.add(
        Fan::MiddleTiles,
        u8::from(all_suited_in(&context.all_tiles, 4, 6)),
    );
    values.add(
        Fan::LowerTiles,
        u8::from(all_suited_in(&context.all_tiles, 1, 3)),
    );
}

fn collect_medium_value_fans(values: &mut FanValues, context: &ScoreContext<'_>) {
    values.add(Fan::PureStraight, u8::from(context.pure_straight));
    values.add(
        Fan::ThreeSuitedTerminalChows,
        u8::from(context.three_suited_terminal),
    );
    values.add(
        Fan::PureShiftedChows,
        u8::from(context.pure_shifted_chows && !context.four_shifted_chows),
    );
    let all_fives = context.sets_pair.is_some_and(|(sets, pair)| {
        context.is_standard
            && matches!(pair, MahjongTileKind::Suited { rank: 5, .. })
            && sets.iter().all(|set| set_contains_rank(set, 5))
    });
    values.add(Fan::AllFives, u8::from(all_fives));
    values.add(Fan::TriplePung, u8::from(context.triple_pung));
    values.add(
        Fan::ThreeConcealedPungs,
        u8::from(context.concealed_pungs == 3),
    );
    values.add(
        Fan::LesserHonorsAndKnittedTiles,
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
        Fan::KnittedStraight,
        u8::from(matches!(context.form, Form::Knitted { straight: true, .. })),
    );
    values.add(
        Fan::UpperFour,
        u8::from(all_suited_in(&context.all_tiles, 6, 9)),
    );
    values.add(
        Fan::LowerFour,
        u8::from(all_suited_in(&context.all_tiles, 1, 4)),
    );
    values.add(
        Fan::BigThreeWinds,
        u8::from(context.wind_pungs == 3 && !context.big_four && !context.little_four),
    );
    values.add(Fan::MixedStraight, u8::from(context.mixed_straight));
    values.add(
        Fan::ReversibleTiles,
        u8::from(context.all_tiles.iter().all(is_reversible)),
    );
    values.add(Fan::MixedTripleChow, u8::from(context.mixed_triple));
    values.add(
        Fan::MixedShiftedPungs,
        u8::from(context.mixed_shifted_pungs),
    );
}

fn collect_basic_fans(values: &mut FanValues, context: &ScoreContext<'_>) {
    values.add(Fan::AllPungs, u8::from(context.all_pungs));
    values.add(
        Fan::HalfFlush,
        u8::from(context.suit_count == 1 && context.has_honor),
    );
    values.add(
        Fan::MixedShiftedChows,
        u8::from(has_mixed_shifted_chows(&context.chows)),
    );
    values.add(Fan::AllTypes, u8::from(context.all_types));
    values.add(Fan::MeldedHand, u8::from(context.melded_hand));
    values.add(
        Fan::TwoDragonPungs,
        u8::from(context.dragon_pungs == 2 && !context.big_three_dragons && !context.little_three),
    );
    values.add(Fan::OutsideHand, u8::from(context.outside));
    values.add(Fan::FullyConcealedHand, u8::from(context.fully_concealed));
    if !context.big_three_dragons && !context.little_three && context.dragon_pungs == 1 {
        values.add(Fan::DragonPung, 1);
    }
    if !context.big_four {
        if context
            .pungs
            .contains(&MahjongTileKind::Wind(context.input.context.prevalent_wind))
        {
            values.add(Fan::PrevalentWind, 1);
        }
        if context
            .pungs
            .contains(&MahjongTileKind::Wind(context.input.context.seat_wind))
        {
            values.add(Fan::SeatWind, 1);
        }
    }
    values.add(Fan::ConcealedHand, u8::from(context.concealed_hand));
    values.add(Fan::AllChows, u8::from(context.all_chows));
    if !context.quadruple_chow {
        values.add(Fan::TileHog, context.tile_hogs);
    }
    if !context.triple_pung {
        values.add(Fan::DoublePung, context.double_pungs);
    }
    values.add(
        Fan::TwoConcealedPungs,
        u8::from(context.concealed_pungs == 2),
    );
    values.add(Fan::AllSimples, u8::from(context.all_simples));
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
                MahjongTileKind::Wind(wind) => {
                    *wind != context.input.context.prevalent_wind
                        && *wind != context.input.context.seat_wind
                }
                _ => false,
            })
            .count() as u8;
        values.add(Fan::PungOfTerminalsOrHonors, terminal_honor_pungs);
    }
    if !context.full_flush && !context.pure_terminal_chows {
        values.add(Fan::OneVoidedSuit, u8::from(context.suit_count <= 2));
    }
    if !context.full_flush
        && !context.all_chows
        && !values.contains(Fan::UpperTiles)
        && !values.contains(Fan::LowerTiles)
        && !values.contains(Fan::UpperFour)
        && !values.contains(Fan::LowerFour)
        && !context.three_suited_terminal
    {
        values.add(Fan::NoHonors, u8::from(!context.has_honor));
    }
}

fn collect_relation_fans(values: &mut FanValues, context: &ScoreContext<'_>) {
    if context.quadruple_chow || context.pure_terminal_chows {
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
        if pure_double > 0 {
            pure_double = 1;
            mixed_double = 0;
        } else {
            mixed_double = mixed_double.min(1);
        }
    }
    if context.three_suited_terminal {
        mixed_double = 0;
    }
    values.add(Fan::PureDoubleChow, pure_double);
    values.add(Fan::MixedDoubleChow, mixed_double);
    if !context.pure_straight && !context.four_shifted_chows {
        values.add(
            Fan::ShortStraight,
            max_matching(&context.chows, |left, right| {
                left.0 == right.0 && left.1.abs_diff(right.1) == 3
            }),
        );
    }
    if !context.pure_straight && !context.four_shifted_chows && !context.three_suited_terminal {
        values.add(
            Fan::TwoTerminalChows,
            max_matching(&context.chows, |left, right| {
                left.0 == right.0
                    && [left.1, right.1].contains(&1)
                    && [left.1, right.1].contains(&7)
            }),
        );
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
        values.add(Fan::SingleWait, 1);
    } else if edge {
        values.add(Fan::EdgeWait, 1);
    } else if closed {
        values.add(Fan::ClosedWait, 1);
    }
}
