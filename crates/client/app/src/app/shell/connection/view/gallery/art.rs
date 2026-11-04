//! 图库中用已打包的扑克牌面组合掼蛋入口。

use crate::app::runtime::{CardFace, CardRank, CardSuit, UiAssets};
use bevy::picking::Pickable;
use bevy::prelude::*;
use leocard_ui::UiComposer;

pub(super) fn add_guandan_art(
    commands: &mut UiComposer,
    parent: Entity,
    size: f32,
    assets: &UiAssets,
) {
    let cards = [
        (
            CardFace::Suited {
                rank: CardRank::Two,
                suit: CardSuit::Heart,
            },
            0.07,
            0.20,
            -18.0,
            0,
        ),
        (
            CardFace::Suited {
                rank: CardRank::Two,
                suit: CardSuit::Spade,
            },
            0.32,
            0.10,
            0.0,
            1,
        ),
        (CardFace::BigJoker, 0.57, 0.20, 18.0, 2),
    ];
    for (face, left, top, rotation, layer) in cards {
        commands.child(
            parent,
            (
                Node {
                    position_type: PositionType::Absolute,
                    left: px(size * left),
                    top: px(size * top),
                    width: px(size * 0.43),
                    height: px(size * 0.60),
                    ..default()
                },
                ImageNode::new(assets.playing_cards.face(face)),
                UiTransform::from_rotation(Rot2::degrees(rotation)),
                ZIndex(layer),
                Pickable::IGNORE,
            ),
        );
    }
}
