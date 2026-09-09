use super::{CardSide, UnoCard, UnoColor, UnoFace};

fn build_colored_sides(colors: [UnoColor; 4], draw: UnoFace, skip: UnoFace) -> Vec<CardSide> {
    let mut sides = Vec::with_capacity(104);
    for color in colors {
        for value in 1..=9 {
            sides.push(CardSide::colored(color, UnoFace::Number(value)));
            sides.push(CardSide::colored(color, UnoFace::Number(value)));
        }
        for face in [draw, UnoFace::Reverse, skip, UnoFace::Flip] {
            sides.push(CardSide::colored(color, face));
            sides.push(CardSide::colored(color, face));
        }
    }
    sides
}

pub fn build_flip_light_sides() -> Vec<CardSide> {
    let mut sides = build_colored_sides(UnoColor::LIGHT, UnoFace::DrawOne, UnoFace::Skip);
    for _ in 0..4 {
        sides.push(CardSide::wild(UnoFace::Wild));
        sides.push(CardSide::wild(UnoFace::WildDrawTwo));
    }
    sides
}

pub fn build_flip_dark_sides() -> Vec<CardSide> {
    let mut sides = build_colored_sides(UnoColor::DARK, UnoFace::DrawFive, UnoFace::SkipEveryone);
    for _ in 0..4 {
        sides.push(CardSide::wild(UnoFace::DarkWild));
        sides.push(CardSide::wild(UnoFace::WildDrawColor));
    }
    sides
}

pub fn pair_flip_deck(dark_sides: Vec<CardSide>) -> Option<Vec<UnoCard>> {
    let light_sides = build_flip_light_sides();
    if dark_sides.len() != light_sides.len() {
        return None;
    }
    Some(
        light_sides
            .into_iter()
            .enumerate()
            .zip(dark_sides)
            .map(|((index, light), dark)| UnoCard::paired(light, dark, index as u8))
            .collect(),
    )
}

const FIXED_PAIRING: [usize; 112] = [
    17, 54, 91, 16, 53, 90, 15, 52, 89, 14, 51, 88, 13, 50, 87, 12, 49, 86, 11, 48, 85, 10, 47, 84,
    9, 46, 83, 8, 45, 82, 7, 44, 81, 6, 43, 80, 5, 42, 79, 4, 41, 78, 3, 40, 77, 2, 39, 76, 1, 38,
    75, 0, 37, 74, 111, 36, 73, 110, 35, 72, 109, 34, 71, 108, 33, 70, 107, 32, 69, 106, 31, 68,
    105, 30, 67, 104, 29, 66, 103, 28, 65, 102, 27, 64, 101, 26, 63, 100, 25, 62, 99, 24, 61, 98,
    23, 60, 97, 22, 59, 96, 21, 58, 95, 20, 57, 94, 19, 56, 93, 18, 55, 92,
];

pub fn build_flip_deck() -> Vec<UnoCard> {
    let dark_sides = build_flip_dark_sides();
    pair_flip_deck(
        FIXED_PAIRING
            .into_iter()
            .map(|index| dark_sides[index])
            .collect(),
    )
    .expect("fixed UNO FLIP pairing must contain 112 dark sides")
}
