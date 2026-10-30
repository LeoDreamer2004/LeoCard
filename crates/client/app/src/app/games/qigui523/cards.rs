use super::{HandCardSlot, HandCardVisual, QiGui523UiAction};
use crate::app::presentation::{
    CardAnimationState, CardSize, HandCardHighlight, HandCardSelectionOverlay, hand_card_pose,
    qigui_hand_card_reveal,
};
use crate::app::runtime::UiAssets;
use crate::app::shell::UiAction;
use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;
use bevy::ui_widgets::Button;
use leocard_qigui523::QiGuiCard;
use leocard_ui::UiComposer;

pub(crate) fn sort_cards_high_to_low(cards: &mut [QiGuiCard]) {
    cards.sort_by(|left, right| {
        right
            .rank()
            .strength()
            .cmp(&left.rank().strength())
            .then_with(|| right.suit().strength().cmp(&left.suit().strength()))
            .then_with(|| right.deck().cmp(&left.deck()))
    });
}

pub(super) struct HandCardSpec {
    pub(super) card: QiGuiCard,
    pub(super) selected: bool,
    pub(super) animation: CardAnimationState,
    pub(super) index: usize,
    pub(super) hand_len: usize,
    pub(super) is_last: bool,
}

pub(super) fn add_card_button(
    commands: &mut UiComposer,
    parent: Entity,
    spec: HandCardSpec,
    assets: &UiAssets,
) {
    let HandCardSpec {
        card,
        selected,
        animation,
        index,
        hand_len,
        is_last,
    } = spec;
    let (width, height) = CardSize::QiGuiHand.dimensions();
    let image = assets.playing_cards.face(card);
    let initial_pose = hand_card_pose(
        index,
        hand_len,
        animation.face_hover_amount,
        animation.selected_amount,
        animation.deal_elapsed,
        animation.dealing,
        CardSize::QiGuiHand,
    );
    let highlight = HandCardHighlight::new(
        animation.face_hover_amount,
        animation.selected_amount,
        selected,
        0.72,
    );
    let button = commands.keyed_child(
        parent,
        format!("card:{card:?}"),
        (
            Button,
            UiAction::QiGui523(QiGui523UiAction::ToggleCard),
            HandCardSlot {
                card,
                index,
                is_last,
                hover_amount: animation.slot_hover_amount,
            },
            RelativeCursorPosition::default(),
            Node {
                width: px(if is_last {
                    width
                } else {
                    if cfg!(target_os = "android") {
                        qigui_hand_card_reveal(hand_len)
                    } else {
                        32.0
                    }
                }),
                height: px(height),
                ..default()
            },
        ),
    );

    let card_face = commands.child(
        button,
        (
            HandCardVisual {
                button,
                card,
                index,
                selected,
                hover_amount: animation.face_hover_amount,
                selected_amount: animation.selected_amount,
                deal_elapsed: animation.deal_elapsed,
                dealing: animation.dealing,
                hand_len,
            },
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                bottom: px(0),
                width: px(width),
                height: px(height),
                border: UiRect::all(px(2)),
                border_radius: BorderRadius::all(px(6)),
                ..default()
            },
            UiTransform {
                translation: initial_pose.translation,
                rotation: initial_pose.rotation,
                ..UiTransform::IDENTITY
            },
            ImageNode::new(image).with_color(highlight.tint()),
            BorderColor::all(highlight.border()),
            highlight.outline(),
            highlight.shadow(),
            GlobalZIndex(index as i32 + 1),
            Pickable::IGNORE,
        ),
    );
    commands.child(
        card_face,
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
