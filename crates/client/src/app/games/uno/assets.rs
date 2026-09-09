use super::UnoSoundAssets;
use bevy::prelude::*;
use leocard_uno::{Mode, UnoColor, UnoFace, UnoRuleSet, build_deck_for_rules};
use std::collections::HashMap;

#[derive(Resource, Default)]
pub(crate) struct UnoAssets {
    pub cards: HashMap<(Option<UnoColor>, UnoFace), Handle<Image>>,
    pub card_back: Handle<Image>,
    pub sounds: UnoSoundAssets,
}

impl UnoAssets {
    pub(super) fn load(asset_server: &AssetServer) -> Self {
        let mut cards = HashMap::new();
        for rules in [
            UnoRuleSet {
                swap_pack: true,
                reverse_pack: true,
                stack_pack: true,
                ..UnoRuleSet::default()
            },
            UnoRuleSet {
                mode: Mode::NoMercy,
                ..UnoRuleSet::default()
            },
            UnoRuleSet {
                mode: Mode::Flip,
                ..UnoRuleSet::default()
            },
        ] {
            for card in build_deck_for_rules(rules) {
                for face in [Some(card), card.opposite_public_face()] {
                    let Some(face) = face else { continue };
                    cards.entry((face.color(), face.face())).or_insert_with(|| {
                        asset_server.load(crate::app::uno_card_asset_path(face))
                    });
                }
            }
        }
        Self {
            cards,
            card_back: asset_server.load("cards/uno/card_back.png"),
            sounds: UnoSoundAssets::load(asset_server),
        }
    }
}

pub(super) fn load_uno_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(UnoAssets::load(&asset_server));
}
