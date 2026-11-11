use crate::app::runtime::UiAssets;
use bevy::prelude::*;
use bevy::text::FontWeight;

#[derive(Clone, Copy)]
pub(crate) enum CasinoButtonTone {
    Fold,
    Call,
    Raise,
    AllIn,
}

#[derive(Clone, Copy)]
pub(crate) enum ColorCardButtonTone {
    Red,
    Blue,
    Yellow,
    Teal,
    Orange,
    Gray,
}

#[derive(Clone, Copy)]
pub(crate) enum PaperCardButtonTone {
    Rust,
    Stone,
    Jade,
    Indigo,
    Pine,
    Amber,
}

#[derive(Clone, Copy)]
pub(crate) enum TableButtonSkin {
    Casino(CasinoButtonTone),
    ColorCard(ColorCardButtonTone),
    PaperCard(PaperCardButtonTone),
}

pub(super) struct TableButtonVisual<'a> {
    pub image: &'a Handle<Image>,
    pub font: &'a Handle<Font>,
    pub text: Color,
    pub weight: FontWeight,
    pub border: BorderRect,
}

impl TableButtonSkin {
    pub(super) fn visual(self, assets: &UiAssets) -> TableButtonVisual<'_> {
        let (image, font, text, weight, border) = match self {
            Self::Casino(tone) => {
                let textures = &assets.controls.casino_buttons;
                let (image, text) = match tone {
                    CasinoButtonTone::Fold => (&textures.charcoal, [223, 216, 195]),
                    CasinoButtonTone::Call => (&textures.petrol, [198, 233, 247]),
                    CasinoButtonTone::Raise => (&textures.bronze, [35, 29, 25]),
                    CasinoButtonTone::AllIn => (&textures.oxblood, [255, 209, 192]),
                };
                (
                    image,
                    &assets.title_font,
                    text,
                    FontWeight::BOLD,
                    BorderRect {
                        min_inset: Vec2::new(72.0, 16.0),
                        max_inset: Vec2::new(36.0, 20.0),
                    },
                )
            }
            Self::ColorCard(tone) => {
                let textures = &assets.controls.color_card_buttons;
                let (image, text) = match tone {
                    ColorCardButtonTone::Red => (&textures.red, [255, 244, 210]),
                    ColorCardButtonTone::Blue => (&textures.blue, [222, 244, 255]),
                    ColorCardButtonTone::Yellow => (&textures.yellow, [16, 49, 53]),
                    ColorCardButtonTone::Teal => (&textures.teal, [232, 250, 226]),
                    ColorCardButtonTone::Orange => (&textures.orange, [26, 43, 44]),
                    ColorCardButtonTone::Gray => (&textures.gray, [221, 233, 242]),
                };
                (
                    image,
                    &textures.font,
                    text,
                    FontWeight::NORMAL,
                    BorderRect {
                        min_inset: Vec2::new(72.0, 24.0),
                        max_inset: Vec2::new(72.0, 24.0),
                    },
                )
            }
            Self::PaperCard(tone) => {
                let textures = &assets.controls.paper_card_buttons;
                let (image, text) = match tone {
                    PaperCardButtonTone::Rust => (&textures.rust, [255, 226, 167]),
                    PaperCardButtonTone::Stone => (&textures.stone, [233, 225, 205]),
                    PaperCardButtonTone::Jade => (&textures.jade, [215, 244, 212]),
                    PaperCardButtonTone::Indigo => (&textures.indigo, [241, 231, 206]),
                    PaperCardButtonTone::Pine => (&textures.pine, [255, 229, 169]),
                    PaperCardButtonTone::Amber => (&textures.amber, [51, 36, 25]),
                };
                (
                    image,
                    &textures.font,
                    text,
                    FontWeight::NORMAL,
                    BorderRect {
                        min_inset: Vec2::new(80.0, 18.0),
                        max_inset: Vec2::new(28.0, 20.0),
                    },
                )
            }
        };
        TableButtonVisual {
            image,
            font,
            text: Color::srgb_u8(text[0], text[1], text[2]),
            weight,
            border,
        }
    }
}
