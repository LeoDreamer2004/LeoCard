use super::patterns::is_nine_gates;
use super::{FanValues, Form, MahjongFan, MahjongWinSource, ScoreContext};
use crate::{MahjongDragon, MahjongKongKind, MahjongSuit, MahjongTileKind};

pub(super) fn collect_special_fans(values: &mut FanValues, context: &ScoreContext<'_>) {
    values.add(MahjongFan::BigFourWinds, u8::from(context.big_four));
    values.add(
        MahjongFan::BigThreeDragons,
        u8::from(context.big_three_dragons),
    );
    values.add(
        MahjongFan::AllGreen,
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
    values.add(
        MahjongFan::NineGates,
        u8::from(is_nine_gates(context.input)),
    );
    values.add(MahjongFan::FourKongs, u8::from(context.kongs.len() == 4));
    values.add(
        MahjongFan::SevenShiftedPairs,
        u8::from(matches!(context.form, Form::SevenPairs { shifted: true })),
    );
    values.add(
        MahjongFan::ThirteenOrphans,
        u8::from(matches!(context.form, Form::ThirteenOrphans)),
    );
    values.add(MahjongFan::AllTerminals, u8::from(context.all_terminals));
    values.add(MahjongFan::LittleFourWinds, u8::from(context.little_four));
    values.add(
        MahjongFan::LittleThreeDragons,
        u8::from(context.little_three),
    );
    values.add(MahjongFan::AllHonors, u8::from(context.all_honors));
    values.add(
        MahjongFan::FourConcealedPungs,
        u8::from(context.concealed_pungs == 4),
    );
    values.add(
        MahjongFan::PureTerminalChows,
        u8::from(context.pure_terminal_chows),
    );
    collect_win_source_fans(values, context);
}

fn collect_win_source_fans(values: &mut FanValues, context: &ScoreContext<'_>) {
    if context.input.context.last_wall_tile {
        if context.input.context.source.is_self_draw() {
            values.add(MahjongFan::LastTileDraw, 1);
        } else if matches!(context.input.context.source, MahjongWinSource::Discard(_)) {
            values.add(MahjongFan::LastTileClaim, 1);
        }
    }
    match context.input.context.source {
        MahjongWinSource::KongReplacement => values.add(MahjongFan::OutWithReplacementTile, 1),
        MahjongWinSource::RobbingKong(_) => values.add(MahjongFan::RobbingTheKong, 1),
        _ => {}
    }
    add_kong_fans(values, &context.kongs);
    if context.input.context.last_of_kind
        && !matches!(
            context.input.context.source,
            MahjongWinSource::RobbingKong(_)
        )
    {
        values.add(MahjongFan::LastTile, 1);
    }
    if matches!(
        context.input.context.source,
        MahjongWinSource::SelfDraw | MahjongWinSource::FlowerReplacement
    ) {
        values.add(MahjongFan::SelfDrawn, 1);
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
            1 => values.insert_points(MahjongFan::ConcealedKong, 1, 2),
            2 => values.insert_points(MahjongFan::TwoConcealedKongs, 1, 8),
            _ => {}
        }
        return;
    }
    match (concealed, melded) {
        (2, 0) => values.insert_points(MahjongFan::TwoConcealedKongs, 1, 8),
        (0, 2) => values.insert_points(MahjongFan::TwoMeldedKongs, 1, 4),
        (1, 1) => values.insert_points(MahjongFan::TwoMeldedKongs, 1, 6),
        (1, 0) => values.insert_points(MahjongFan::ConcealedKong, 1, 2),
        (0, 1) => values.insert_points(MahjongFan::MeldedKong, 1, 1),
        _ => {}
    }
}
