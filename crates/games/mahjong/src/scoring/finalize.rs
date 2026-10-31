use super::{Form, MahjongFan, MahjongFanValue, MahjongScoreResult, ScoreContext};
use std::collections::BTreeMap;

pub(super) struct FanValues(BTreeMap<MahjongFan, (u8, u16)>);

impl FanValues {
    pub(super) fn new() -> Self {
        Self(BTreeMap::new())
    }

    pub(super) fn add(&mut self, fan: MahjongFan, count: impl Into<u8>) {
        let count = count.into();
        if count > 0 {
            self.0.insert(fan, (count, fan.points() * u16::from(count)));
        }
    }

    pub(super) fn insert_points(&mut self, fan: MahjongFan, count: u8, points: u16) {
        if count > 0 {
            self.0.insert(fan, (count, points));
        }
    }

    pub(super) fn contains(&self, fan: MahjongFan) -> bool {
        self.0.contains_key(&fan)
    }

    pub(super) fn remove(&mut self, fan: MahjongFan) {
        self.0.remove(&fan);
    }

    pub(super) fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

pub(super) fn finish_score(
    mut values: FanValues,
    context: &ScoreContext<'_>,
) -> MahjongScoreResult {
    suppress_implied(&mut values, context);
    if values.is_empty() {
        values.add(MahjongFan::ChickenHand, 1);
    }
    let flower_points = context.input.context.flower_count;
    values.insert_points(
        MahjongFan::FlowerTiles,
        flower_points,
        u16::from(flower_points),
    );
    let fans = values
        .0
        .into_iter()
        .map(|(fan, (count, points))| MahjongFanValue { fan, count, points })
        .collect::<Vec<_>>();
    let points_without_flowers = fans
        .iter()
        .filter(|value| value.fan != MahjongFan::FlowerTiles)
        .map(|value| value.points)
        .sum();
    MahjongScoreResult {
        total_points: points_without_flowers + u16::from(flower_points),
        fans,
        points_without_flowers,
        flower_points,
    }
}

fn suppress_implied(values: &mut FanValues, context: &ScoreContext<'_>) {
    let remove = |values: &mut FanValues, fans: &[MahjongFan]| {
        for fan in fans {
            values.remove(*fan);
        }
    };
    if context.big_four {
        remove(
            values,
            &[
                MahjongFan::BigThreeWinds,
                MahjongFan::AllPungs,
                MahjongFan::PrevalentWind,
                MahjongFan::SeatWind,
                MahjongFan::PungOfTerminalsOrHonors,
            ],
        );
    }
    if context.big_three_dragons || context.little_three {
        remove(
            values,
            &[MahjongFan::TwoDragonPungs, MahjongFan::DragonPung],
        );
    }
    if context.little_four {
        remove(
            values,
            &[
                MahjongFan::BigThreeWinds,
                MahjongFan::PungOfTerminalsOrHonors,
            ],
        );
    }
    if context.all_honors || context.all_terminals || context.terminals_honors {
        remove(
            values,
            &[
                MahjongFan::AllPungs,
                MahjongFan::OutsideHand,
                MahjongFan::PungOfTerminalsOrHonors,
            ],
        );
    }
    if context.all_terminals {
        values.remove(MahjongFan::NoHonors);
    }
    if context.all_even_pungs {
        remove(values, &[MahjongFan::AllPungs, MahjongFan::AllSimples]);
    }
    if context.fully_concealed {
        values.remove(MahjongFan::SelfDrawn);
    }
    match context.form {
        Form::SevenPairs { shifted: true } => remove(
            values,
            &[
                MahjongFan::SevenPairs,
                MahjongFan::FullFlush,
                MahjongFan::ConcealedHand,
                MahjongFan::SingleWait,
            ],
        ),
        Form::SevenPairs { shifted: false } => {
            remove(values, &[MahjongFan::ConcealedHand, MahjongFan::SingleWait]);
        }
        Form::ThirteenOrphans => remove(
            values,
            &[
                MahjongFan::AllTerminalsAndHonors,
                MahjongFan::AllTypes,
                MahjongFan::ConcealedHand,
                MahjongFan::SingleWait,
            ],
        ),
        Form::Knitted { pair: None, .. } => {
            remove(values, &[MahjongFan::AllTypes, MahjongFan::ConcealedHand]);
        }
        Form::Knitted { pair: Some(_), .. } | Form::Standard { .. } => {}
    }
    if values.contains(MahjongFan::NineGates) {
        remove(
            values,
            &[
                MahjongFan::FullFlush,
                MahjongFan::ConcealedHand,
                MahjongFan::PungOfTerminalsOrHonors,
            ],
        );
    }
    if values.contains(MahjongFan::FourKongs) {
        remove(
            values,
            &[
                MahjongFan::ThreeKongs,
                MahjongFan::TwoMeldedKongs,
                MahjongFan::MeldedKong,
                MahjongFan::SingleWait,
            ],
        );
    }
    if values.contains(MahjongFan::FourConcealedPungs) {
        remove(
            values,
            &[
                MahjongFan::AllPungs,
                MahjongFan::ThreeConcealedPungs,
                MahjongFan::TwoConcealedPungs,
                MahjongFan::ConcealedHand,
            ],
        );
    } else if values.contains(MahjongFan::ThreeConcealedPungs) {
        values.remove(MahjongFan::TwoConcealedPungs);
    }
    if values.contains(MahjongFan::FullFlush) {
        values.remove(MahjongFan::NoHonors);
    }
    if values.contains(MahjongFan::HalfFlush) || context.all_honors {
        values.remove(MahjongFan::OneVoidedSuit);
    }
    if values.contains(MahjongFan::AllChows) {
        values.remove(MahjongFan::NoHonors);
    }
    if values.contains(MahjongFan::MeldedHand) {
        values.remove(MahjongFan::SingleWait);
    }
    if values.contains(MahjongFan::ReversibleTiles) {
        values.remove(MahjongFan::OneVoidedSuit);
    }
    if values.contains(MahjongFan::OutWithReplacementTile)
        || values.contains(MahjongFan::LastTileDraw)
    {
        values.remove(MahjongFan::SelfDrawn);
    }
    if values.contains(MahjongFan::RobbingTheKong) {
        values.remove(MahjongFan::LastTile);
    }
    if values.contains(MahjongFan::PureTerminalChows) {
        remove(
            values,
            &[
                MahjongFan::SevenPairs,
                MahjongFan::FullFlush,
                MahjongFan::AllChows,
                MahjongFan::NoHonors,
                MahjongFan::PureDoubleChow,
                MahjongFan::TwoTerminalChows,
            ],
        );
    }
    if values.contains(MahjongFan::QuadrupleChow) {
        remove(
            values,
            &[
                MahjongFan::PureTripleChow,
                MahjongFan::PureShiftedPungs,
                MahjongFan::TileHog,
                MahjongFan::PureDoubleChow,
            ],
        );
    }
    if values.contains(MahjongFan::FourPureShiftedPungs) {
        remove(
            values,
            &[
                MahjongFan::PureShiftedPungs,
                MahjongFan::PureTripleChow,
                MahjongFan::AllPungs,
            ],
        );
    }
    if values.contains(MahjongFan::FourPureShiftedChows) {
        remove(
            values,
            &[
                MahjongFan::PureShiftedChows,
                MahjongFan::ShortStraight,
                MahjongFan::TwoTerminalChows,
            ],
        );
    }
    if values.contains(MahjongFan::PureTripleChow) {
        remove(
            values,
            &[MahjongFan::PureShiftedPungs, MahjongFan::PureDoubleChow],
        );
    }
    if values.contains(MahjongFan::ThreeSuitedTerminalChows) {
        remove(
            values,
            &[
                MahjongFan::AllChows,
                MahjongFan::NoHonors,
                MahjongFan::MixedDoubleChow,
                MahjongFan::TwoTerminalChows,
            ],
        );
    }
    if values.contains(MahjongFan::AllFives) {
        remove(values, &[MahjongFan::AllSimples, MahjongFan::NoHonors]);
    }
    if values.contains(MahjongFan::UpperTiles) {
        values.remove(MahjongFan::UpperFour);
    }
    if values.contains(MahjongFan::LowerTiles) {
        values.remove(MahjongFan::LowerFour);
    }
    for fan in [
        MahjongFan::UpperTiles,
        MahjongFan::LowerTiles,
        MahjongFan::UpperFour,
        MahjongFan::LowerFour,
    ] {
        if values.contains(fan) {
            values.remove(MahjongFan::NoHonors);
        }
    }
    if values.contains(MahjongFan::MiddleTiles) {
        remove(values, &[MahjongFan::AllSimples, MahjongFan::NoHonors]);
    }
    if values.contains(MahjongFan::AllSimples) || context.all_even_pungs {
        values.remove(MahjongFan::NoHonors);
    }
    if values.contains(MahjongFan::TwoDragonPungs) {
        values.remove(MahjongFan::DragonPung);
    }
    if values.contains(MahjongFan::TwoConcealedKongs) {
        remove(
            values,
            &[MahjongFan::ConcealedKong, MahjongFan::TwoConcealedPungs],
        );
    }
    if values.contains(MahjongFan::TwoMeldedKongs) {
        remove(values, &[MahjongFan::MeldedKong, MahjongFan::ConcealedKong]);
    }
}
