use super::actions::GuandanUiAction;
use super::cards::add_level_marker;
use super::state::{GuandanUiState, local_turn, tribute_choice};
use crate::app::presentation::{
    ACCENT, CardAnimationState, CardSize, GAME_SIDE_CONTROL_SCALE, HandCardHighlight,
    HandCardLayout, SilentButton, add_text, hand_card_pose_for_layout,
};
use crate::app::runtime::UiAssets;
use crate::app::shell::UiAction;
use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;
use bevy::ui_widgets::Button;
use leocard_guandan::{GuandanCard, rank_strength};
use leocard_protocol::GuandanSnapshot;
use leocard_ui::{CustomButtonMotion, UiComposer};
use std::cmp::Reverse;

#[derive(Component)]
#[require(CustomButtonMotion, SilentButton)]
pub(super) struct HandCardSlot {
    pub card: GuandanCard,
    pub index: usize,
}

#[derive(Component)]
pub(super) struct HandCardSelectionOverlay {
    pub index: usize,
}

#[derive(Component)]
pub(super) struct HandCardVisual {
    pub card: GuandanCard,
    pub index: usize,
    pub hand_len: usize,
    pub button: Entity,
    pub animation: CardAnimationState,
}

pub(super) fn hand_layout(index: usize, hand_len: usize) -> HandCardLayout {
    HandCardLayout {
        index,
        hand_len,
        card_width: CardSize::ShengjiHand.dimensions().0,
        reveal: (760.0 / hand_len.max(1) as f32).clamp(18.0, 30.0 * GAME_SIDE_CONTROL_SCALE),
    }
}

pub(super) fn render_hand(
    commands: &mut UiComposer,
    parent: Entity,
    game: &GuandanSnapshot,
    ui: &GuandanUiState,
    can_control: bool,
    assets: &UiAssets,
) {
    let cards = sorted_hand(game);
    let choosing_tribute = local_turn(game) && game.tribute_action.is_some();
    let (width, height) = CardSize::ShengjiHand.dimensions();
    let reveal = hand_layout(0, cards.len()).reveal;
    for (index, card) in cards.iter().enumerate() {
        let animation = ui.card_animations.get(card).copied().unwrap_or_default();
        let highlight = HandCardHighlight::new(
            animation.face_hover_amount,
            animation.selected_amount,
            ui.selected.contains(card),
            0.72,
        );
        let pose = hand_card_pose_for_layout(hand_layout(index, cards.len()), animation);
        let slot = commands.keyed_child(
            parent,
            format!("hand:{}:{}:{}", card.deck, card.rank, card.suit as u8),
            (
                HandCardSlot { card: *card, index },
                RelativeCursorPosition::default(),
                Node {
                    width: px(if index + 1 == cards.len() {
                        width
                    } else {
                        reveal
                    }),
                    height: px(height + 24.0),
                    flex_shrink: 0.0,
                    position_type: PositionType::Relative,
                    ..default()
                },
            ),
        );
        if can_control {
            commands
                .entity(slot)
                .insert((Button, UiAction::Guandan(GuandanUiAction::ToggleCard)));
        } else {
            commands.entity(slot).insert(Pickable::IGNORE);
        }
        let face = commands.child(
            slot,
            (
                Node {
                    position_type: PositionType::Absolute,
                    width: px(width),
                    height: px(height),
                    bottom: px(0),
                    border: UiRect::all(px(2)),
                    border_radius: BorderRadius::all(px(6)),
                    ..default()
                },
                ImageNode::new(assets.playing_cards.face(*card)).with_color(highlight.tint()),
                BorderColor::all(highlight.border()),
                HandCardVisual {
                    card: *card,
                    index,
                    hand_len: cards.len(),
                    button: slot,
                    animation,
                },
                UiTransform {
                    translation: pose.translation,
                    rotation: pose.rotation,
                    ..default()
                },
                highlight.outline(),
                highlight.shadow(),
                if animation.dealing && animation.deal_elapsed < 0.0 {
                    Visibility::Hidden
                } else {
                    Visibility::Inherited
                },
                ZIndex(index as i32),
                Pickable::IGNORE,
            ),
        );
        add_level_marker(commands, face, *card, game.level, assets);
        let tribute_label = if game.received_tribute == Some(*card) {
            Some("贡")
        } else if choosing_tribute && tribute_choice(game, *card).is_some() {
            Some("可选")
        } else {
            None
        };
        if let Some(label) = tribute_label {
            let tag = add_text(commands, face, label, 12.0, ACCENT, assets);
            commands.entity(tag).entry::<Node>().and_modify(|mut node| {
                node.position_type = PositionType::Absolute;
                node.left = px(3);
                node.bottom = px(24);
            });
            commands.entity(tag).insert((
                TextLayout::no_wrap(),
                BackgroundColor(Color::BLACK.with_alpha(0.86)),
                Pickable::IGNORE,
            ));
        }
        commands.child(
            face,
            (
                HandCardSelectionOverlay { index },
                Node {
                    position_type: PositionType::Absolute,
                    left: px(0),
                    top: px(0),
                    width: percent(100),
                    height: percent(100),
                    border_radius: BorderRadius::all(px(5)),
                    ..default()
                },
                BackgroundColor(Color::NONE),
                Pickable::IGNORE,
            ),
        );
    }
}

pub(super) fn sorted_hand(game: &GuandanSnapshot) -> Vec<GuandanCard> {
    let mut cards = game.your_hand.clone();
    cards.sort_by_key(|card| {
        Reverse((
            rank_strength(card.rank, game.level),
            card.is_wild(game.level),
            card.suit,
            card.deck,
        ))
    });
    cards
}
