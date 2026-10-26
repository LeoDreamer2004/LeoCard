use super::playback::{
    BlackjackCardSlot, BlackjackCardTiming, BlackjackPlayback, DEALER_FLIP_DURATION,
};
use crate::app::presentation::{PlayingCardFlip, TablePieceDeal};
use crate::app::runtime::UiAssets;
use bevy::picking::Pickable;
use bevy::prelude::*;
use leocard_blackjack::BlackjackCard;
use leocard_ui::UiComposer;

pub(super) struct BlackjackCardSpec {
    pub slot: BlackjackCardSlot,
    pub face: Option<BlackjackCard>,
    pub node: Node,
}

pub(super) fn add_card(
    commands: &mut UiComposer,
    parent: Entity,
    spec: BlackjackCardSpec,
    pile: Entity,
    playback: &BlackjackPlayback,
    assets: &UiAssets,
) -> Entity {
    let timing = playback.timing(spec.slot);
    let back = assets.playing_cards.card_back.clone();
    let face = spec.face.map(|card| assets.playing_cards.face(card));
    let image = if timing.flip_at.is_some() {
        back.clone()
    } else {
        face.clone().unwrap_or_else(|| back.clone())
    };
    let entity = commands.child(
        parent,
        (
            spec.node,
            ImageNode::new(image),
            Pickable::IGNORE,
            UiTransform::IDENTITY,
        ),
    );
    let previous = commands.declared::<BlackjackCardTiming>(entity).copied();
    if let Some(at) = timing.deal_at {
        let deal = TablePieceDeal::from_anchor(pile, at - playback.elapsed())
            .with_duration(spec.slot.deal_duration())
            .with_fade_in(spec.slot.deal_duration() * 0.2);
        if previous.is_some_and(|previous| previous.deal_at == timing.deal_at) {
            commands
                .entity(entity)
                .entry::<TablePieceDeal>()
                .or_insert(deal);
        } else {
            commands.entity(entity).insert(deal);
        }
    } else {
        commands.entity(entity).remove::<TablePieceDeal>();
    }
    if let Some(at) = timing.flip_at
        && let Some(face) = face
    {
        let flip = PlayingCardFlip::new(face, back, at - playback.elapsed())
            .with_duration(DEALER_FLIP_DURATION);
        if previous.is_some_and(|previous| previous.flip_at == timing.flip_at) {
            commands
                .entity(entity)
                .entry::<PlayingCardFlip>()
                .or_insert(flip);
        } else {
            commands.entity(entity).insert(flip);
        }
    } else {
        commands.entity(entity).remove::<PlayingCardFlip>();
    }
    commands.entity(entity).insert(timing);
    entity
}
