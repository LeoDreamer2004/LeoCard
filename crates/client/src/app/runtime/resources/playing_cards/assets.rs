use super::{CardFace, CardSuit};
use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct PlayingCardAssets {
    faces: HashMap<CardFace, Handle<Image>>,
    suits: HashMap<CardSuit, Handle<Image>>,
    pub card_back: Handle<Image>,
}

impl PlayingCardAssets {
    pub(crate) fn load(server: &AssetServer) -> Self {
        Self {
            faces: CardFace::all()
                .map(|face| (face, server.load(face.asset_path())))
                .collect(),
            suits: CardSuit::ALL
                .map(|suit| (suit, server.load(suit.icon_path())))
                .into(),
            card_back: server.load("vendor/kenney/boardgame/PNG/Cards/cardBack_blue4.png"),
        }
    }

    pub(crate) fn face(&self, card: impl Into<CardFace>) -> Handle<Image> {
        self.faces
            .get(&card.into())
            .expect("all standard playing-card faces are preloaded")
            .clone()
    }

    pub(crate) fn suit(&self, suit: impl Into<CardSuit>) -> Handle<Image> {
        self.suits
            .get(&suit.into())
            .expect("all playing-card suit icons are preloaded")
            .clone()
    }
}
