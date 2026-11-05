use crate::app::presentation::{CardSize, TablePieceDeal};
use crate::app::runtime::UiAssets;
use bevy::picking::Pickable;
use bevy::prelude::*;
use leocard_game_common::PlayerId;
use leocard_protocol::GuandanSnapshot;
use leocard_ui::UiComposer;

pub(super) fn status(game: &GuandanSnapshot) -> Option<String> {
    let current = game.current?;
    let (verb, target) = game.tribute_pairs.iter().find_map(|&(giver, receiver)| {
        if current == giver {
            Some(("进贡", receiver))
        } else if current == receiver {
            Some(("还贡", giver))
        } else {
            None
        }
    })?;
    let name = |id| {
        game.players
            .iter()
            .find(|player| player.player.id == id)
            .map_or("玩家", |player| player.player.name.as_str())
    };
    Some(format!("{} 向 {} {verb}", name(current), name(target)))
}

pub(super) fn add_flights(
    commands: &mut UiComposer,
    portraits: [Option<Entity>; 4],
    flights: impl IntoIterator<Item = (PlayerId, PlayerId)>,
    assets: &UiAssets,
) {
    let (width, height) = CardSize::FinishedHand.dimensions();
    for (index, (from, to)) in flights.into_iter().enumerate() {
        let (Some(from), Some(to)) = (portraits[from.index()], portraits[to.index()]) else {
            continue;
        };
        commands.effect_child(
            to,
            (
                Node {
                    position_type: PositionType::Absolute,
                    left: percent(50),
                    top: percent(50),
                    margin: UiRect::axes(px(-width * 0.5), px(-height * 0.5)),
                    width: px(width),
                    height: px(height),
                    ..default()
                },
                ImageNode::new(assets.playing_cards.card_back.clone()).with_color(Color::NONE),
                UiTransform::IDENTITY,
                TablePieceDeal::from_anchor(from, index as f32 * 0.16)
                    .transient(0.48)
                    .with_fade_in(0.12)
                    .with_rotation(-0.22, 0.10),
                GlobalZIndex(1050),
                Pickable::IGNORE,
            ),
        );
    }
}
