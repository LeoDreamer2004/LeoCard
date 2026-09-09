use super::{Fan, FanValue, Form, MahjongScoreResult, ScoreContext};
use std::collections::BTreeMap;

pub(super) struct FanValues(BTreeMap<Fan, (u8, u16)>);

impl FanValues {
    pub(super) fn new() -> Self {
        Self(BTreeMap::new())
    }

    pub(super) fn add(&mut self, fan: Fan, count: impl Into<u8>) {
        let count = count.into();
        if count > 0 {
            self.0.insert(fan, (count, fan.points() * u16::from(count)));
        }
    }

    pub(super) fn insert_points(&mut self, fan: Fan, count: u8, points: u16) {
        if count > 0 {
            self.0.insert(fan, (count, points));
        }
    }

    pub(super) fn contains(&self, fan: Fan) -> bool {
        self.0.contains_key(&fan)
    }

    pub(super) fn remove(&mut self, fan: Fan) {
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
        values.add(Fan::ChickenHand, 1);
    }
    let flower_points = context.input.context.flower_count;
    values.insert_points(Fan::FlowerTiles, flower_points, u16::from(flower_points));
    let fans = values
        .0
        .into_iter()
        .map(|(fan, (count, points))| FanValue { fan, count, points })
        .collect::<Vec<_>>();
    let points_without_flowers = fans
        .iter()
        .filter(|value| value.fan != Fan::FlowerTiles)
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
    let remove = |values: &mut FanValues, fans: &[Fan]| {
        for fan in fans {
            values.remove(*fan);
        }
    };
    if context.big_four {
        remove(
            values,
            &[
                Fan::BigThreeWinds,
                Fan::AllPungs,
                Fan::PrevalentWind,
                Fan::SeatWind,
                Fan::PungOfTerminalsOrHonors,
            ],
        );
    }
    if context.big_three_dragons || context.little_three {
        remove(values, &[Fan::TwoDragonPungs, Fan::DragonPung]);
    }
    if context.little_four {
        remove(values, &[Fan::BigThreeWinds, Fan::PungOfTerminalsOrHonors]);
    }
    if context.all_honors || context.all_terminals || context.terminals_honors {
        remove(
            values,
            &[
                Fan::AllPungs,
                Fan::OutsideHand,
                Fan::PungOfTerminalsOrHonors,
            ],
        );
    }
    if context.all_terminals {
        values.remove(Fan::NoHonors);
    }
    if context.all_even_pungs {
        remove(values, &[Fan::AllPungs, Fan::AllSimples]);
    }
    if context.fully_concealed {
        values.remove(Fan::SelfDrawn);
    }
    match context.form {
        Form::SevenPairs { shifted: true } => remove(
            values,
            &[
                Fan::SevenPairs,
                Fan::FullFlush,
                Fan::ConcealedHand,
                Fan::SingleWait,
            ],
        ),
        Form::SevenPairs { shifted: false } => {
            remove(values, &[Fan::ConcealedHand, Fan::SingleWait]);
        }
        Form::ThirteenOrphans => remove(
            values,
            &[
                Fan::AllTerminalsAndHonors,
                Fan::AllTypes,
                Fan::ConcealedHand,
                Fan::SingleWait,
            ],
        ),
        Form::Knitted { pair: None, .. } => {
            remove(values, &[Fan::AllTypes, Fan::ConcealedHand]);
        }
        Form::Knitted { pair: Some(_), .. } | Form::Standard { .. } => {}
    }
    if values.contains(Fan::NineGates) {
        remove(
            values,
            &[
                Fan::FullFlush,
                Fan::ConcealedHand,
                Fan::PungOfTerminalsOrHonors,
            ],
        );
    }
    if values.contains(Fan::FourKongs) {
        remove(
            values,
            &[
                Fan::ThreeKongs,
                Fan::TwoMeldedKongs,
                Fan::MeldedKong,
                Fan::SingleWait,
            ],
        );
    }
    if values.contains(Fan::FourConcealedPungs) {
        remove(
            values,
            &[
                Fan::AllPungs,
                Fan::ThreeConcealedPungs,
                Fan::TwoConcealedPungs,
                Fan::ConcealedHand,
            ],
        );
    } else if values.contains(Fan::ThreeConcealedPungs) {
        values.remove(Fan::TwoConcealedPungs);
    }
    if values.contains(Fan::FullFlush) {
        values.remove(Fan::NoHonors);
    }
    if values.contains(Fan::HalfFlush) || context.all_honors {
        values.remove(Fan::OneVoidedSuit);
    }
    if values.contains(Fan::AllChows) {
        values.remove(Fan::NoHonors);
    }
    if values.contains(Fan::MeldedHand) {
        values.remove(Fan::SingleWait);
    }
    if values.contains(Fan::ReversibleTiles) {
        values.remove(Fan::OneVoidedSuit);
    }
    if values.contains(Fan::OutWithReplacementTile) || values.contains(Fan::LastTileDraw) {
        values.remove(Fan::SelfDrawn);
    }
    if values.contains(Fan::RobbingTheKong) {
        values.remove(Fan::LastTile);
    }
    if values.contains(Fan::PureTerminalChows) {
        remove(
            values,
            &[
                Fan::SevenPairs,
                Fan::FullFlush,
                Fan::AllChows,
                Fan::NoHonors,
                Fan::PureDoubleChow,
                Fan::TwoTerminalChows,
            ],
        );
    }
    if values.contains(Fan::QuadrupleChow) {
        remove(
            values,
            &[
                Fan::PureTripleChow,
                Fan::PureShiftedPungs,
                Fan::TileHog,
                Fan::PureDoubleChow,
            ],
        );
    }
    if values.contains(Fan::FourPureShiftedPungs) {
        remove(
            values,
            &[Fan::PureShiftedPungs, Fan::PureTripleChow, Fan::AllPungs],
        );
    }
    if values.contains(Fan::FourPureShiftedChows) {
        remove(
            values,
            &[
                Fan::PureShiftedChows,
                Fan::ShortStraight,
                Fan::TwoTerminalChows,
            ],
        );
    }
    if values.contains(Fan::PureTripleChow) {
        remove(values, &[Fan::PureShiftedPungs, Fan::PureDoubleChow]);
    }
    if values.contains(Fan::ThreeSuitedTerminalChows) {
        remove(
            values,
            &[
                Fan::AllChows,
                Fan::NoHonors,
                Fan::MixedDoubleChow,
                Fan::TwoTerminalChows,
            ],
        );
    }
    if values.contains(Fan::AllFives) {
        remove(values, &[Fan::AllSimples, Fan::NoHonors]);
    }
    for fan in [
        Fan::UpperTiles,
        Fan::LowerTiles,
        Fan::UpperFour,
        Fan::LowerFour,
    ] {
        if values.contains(fan) {
            values.remove(Fan::NoHonors);
        }
    }
    if values.contains(Fan::MiddleTiles) {
        values.remove(Fan::AllSimples);
    }
    if values.contains(Fan::AllSimples) || context.all_even_pungs {
        values.remove(Fan::NoHonors);
    }
    if values.contains(Fan::TwoDragonPungs) {
        values.remove(Fan::DragonPung);
    }
    if values.contains(Fan::TwoConcealedKongs) {
        values.remove(Fan::ConcealedKong);
    }
    if values.contains(Fan::TwoMeldedKongs) {
        remove(values, &[Fan::MeldedKong, Fan::ConcealedKong]);
    }
}
