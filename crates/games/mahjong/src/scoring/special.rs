use super::patterns::is_nine_gates;
use super::{Fan, FanValues, Form, ScoreContext, WinSource};
use crate::{MahjongDragon, MahjongKongKind, MahjongSuit, MahjongTileKind};

pub(super) fn collect_special_fans(values: &mut FanValues, context: &ScoreContext<'_>) {
    values.add(Fan::BigFourWinds, u8::from(context.big_four));
    values.add(Fan::BigThreeDragons, u8::from(context.big_three_dragons));
    values.add(
        Fan::AllGreen,
        u8::from(context.all_tiles.iter().all(|tile| {
            matches!(
                tile,
                MahjongTileKind::Suited {
                    suit: MahjongSuit::Bamboo,
                    rank: 2 | 3 | 4 | 6 | 8
                } | MahjongTileKind::Dragon(MahjongDragon::Green)
            )
        })),
    );
    values.add(Fan::NineGates, u8::from(is_nine_gates(context.input)));
    values.add(Fan::FourKongs, u8::from(context.kongs.len() == 4));
    values.add(
        Fan::SevenShiftedPairs,
        u8::from(matches!(context.form, Form::SevenPairs { shifted: true })),
    );
    values.add(
        Fan::ThirteenOrphans,
        u8::from(matches!(context.form, Form::ThirteenOrphans)),
    );
    values.add(Fan::AllTerminals, u8::from(context.all_terminals));
    values.add(Fan::LittleFourWinds, u8::from(context.little_four));
    values.add(Fan::LittleThreeDragons, u8::from(context.little_three));
    values.add(Fan::AllHonors, u8::from(context.all_honors));
    values.add(
        Fan::FourConcealedPungs,
        u8::from(context.concealed_pungs == 4),
    );
    values.add(
        Fan::PureTerminalChows,
        u8::from(context.pure_terminal_chows),
    );
    collect_win_source_fans(values, context);
}

fn collect_win_source_fans(values: &mut FanValues, context: &ScoreContext<'_>) {
    if context.input.context.last_wall_tile {
        if context.input.context.source.is_self_draw() {
            values.add(Fan::LastTileDraw, 1);
        } else if matches!(context.input.context.source, WinSource::Discard(_)) {
            values.add(Fan::LastTileClaim, 1);
        }
    }
    match context.input.context.source {
        WinSource::KongReplacement => values.add(Fan::OutWithReplacementTile, 1),
        WinSource::RobbingKong(_) => values.add(Fan::RobbingTheKong, 1),
        _ => {}
    }
    add_kong_fans(values, &context.kongs);
    if context.input.context.last_of_kind
        && !matches!(context.input.context.source, WinSource::RobbingKong(_))
    {
        values.add(Fan::LastTile, 1);
    }
    if matches!(
        context.input.context.source,
        WinSource::SelfDraw | WinSource::FlowerReplacement
    ) {
        values.add(Fan::SelfDrawn, 1);
    }
}

fn add_kong_fans(values: &mut FanValues, kongs: &[MahjongKongKind]) {
    let concealed = kongs
        .iter()
        .filter(|kind| matches!(kind, MahjongKongKind::Concealed))
        .count();
    let melded = kongs.len() - concealed;
    if kongs.len() >= 3 {
        match concealed {
            1 => values.insert_points(Fan::ConcealedKong, 1, 2),
            2 => values.insert_points(Fan::TwoConcealedKongs, 1, 8),
            _ => {}
        }
        return;
    }
    match (concealed, melded) {
        (2, 0) => values.insert_points(Fan::TwoConcealedKongs, 1, 8),
        (0, 2) => values.insert_points(Fan::TwoMeldedKongs, 1, 4),
        (1, 1) => values.insert_points(Fan::TwoMeldedKongs, 1, 6),
        (1, 0) => values.insert_points(Fan::ConcealedKong, 1, 2),
        (0, 1) => values.insert_points(Fan::MeldedKong, 1, 1),
        _ => {}
    }
}
