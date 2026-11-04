use super::actions::GuandanUiAction;
use super::state::GuandanUiState;
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
use leocard_guandan::GuandanCard;
use leocard_ui::{CustomButtonMotion, UiComposer};

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
    cards: &[GuandanCard],
    level: u8,
    received_tribute: Option<GuandanCard>,
    ui: &GuandanUiState,
    assets: &UiAssets,
) {
    let mut cards = cards.to_vec();
    cards.sort_by_key(|card| (card.rank == level, card.rank, card.suit, card.deck));
    cards.reverse();
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
                Button,
                UiAction::Guandan(GuandanUiAction::ToggleCard),
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
                ZIndex(index as i32),
                Pickable::IGNORE,
            ),
        );
        if received_tribute == Some(*card) {
            let tag = add_text(commands, face, "贡", 14.0, ACCENT, assets);
            commands.entity(tag).entry::<Node>().and_modify(|mut node| {
                node.position_type = PositionType::Absolute;
                node.right = px(3);
                node.top = px(1);
            });
            commands.entity(tag).insert(Pickable::IGNORE);
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
