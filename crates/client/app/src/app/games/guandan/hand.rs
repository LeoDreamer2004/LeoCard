use super::actions::GuandanUiAction;
use super::cards::add_level_marker;
use super::hand_layout::{GuandanHandLayout, HandCardPlacement};
use super::state::{GuandanUiState, local_turn, tribute_choice};
use crate::app::presentation::{
    ACCENT, CardAnimationState, HandCardHighlight, SilentButton, add_text,
};
use crate::app::runtime::UiAssets;
use crate::app::shell::UiAction;
use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;
use bevy::ui_widgets::Button;
use leocard_guandan::GuandanCard;
use leocard_protocol::GuandanSnapshot;
use leocard_ui::{CustomButtonMotion, UiComposer};

#[derive(Component)]
#[require(CustomButtonMotion, SilentButton)]
pub(super) struct HandCardSlot {
    pub card: GuandanCard,
    pub index: usize,
    pub column: usize,
}

#[derive(Component)]
pub(super) struct HandCardSelectionOverlay {
    pub index: usize,
}

#[derive(Component)]
pub(super) struct HandCardVisual {
    pub card: GuandanCard,
    pub placement: HandCardPlacement,
    pub button: Entity,
    pub animation: CardAnimationState,
}

pub(super) fn render_hand(
    commands: &mut UiComposer,
    parent: Entity,
    game: &GuandanSnapshot,
    ui: &GuandanUiState,
    layout: &GuandanHandLayout,
    can_control: bool,
    assets: &UiAssets,
) {
    let choosing_tribute = local_turn(game) && game.tribute_action.is_some();
    for &(card, placement) in &layout.cards {
        let animation = ui.card_animations.get(&card).copied().unwrap_or_default();
        let highlight = HandCardHighlight::new(
            animation.face_hover_amount,
            animation.selected_amount,
            ui.selected.contains(&card),
            0.72,
        );
        let pose = placement.pose(animation);
        let hit = placement.hit_rect();
        let slot = commands.keyed_child(
            parent,
            format!("hand:{}:{}:{}", card.deck, card.rank, card.suit as u8),
            (
                HandCardSlot {
                    card,
                    index: placement.index,
                    column: placement.column,
                },
                RelativeCursorPosition::default(),
                Node {
                    position_type: PositionType::Absolute,
                    left: px(hit.min.x),
                    top: px(hit.min.y),
                    width: px(hit.width()),
                    height: px(hit.height()),
                    overflow: Overflow::visible(),
                    ..default()
                },
                ZIndex(placement.index as i32),
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
                    left: px(placement.position.x - hit.min.x),
                    top: px(placement.position.y - hit.min.y),
                    width: px(placement.size.x),
                    height: px(placement.size.y),
                    border: UiRect::all(px(2)),
                    border_radius: BorderRadius::all(px(6)),
                    ..default()
                },
                ImageNode::new(assets.playing_cards.face(card)).with_color(highlight.tint()),
                BorderColor::all(highlight.border()),
                HandCardVisual {
                    card,
                    placement,
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
                Pickable::IGNORE,
            ),
        );
        add_level_marker(commands, face, card, game.level, assets);
        let tribute_label = if game.received_tribute == Some(card) {
            Some("贡")
        } else if choosing_tribute && tribute_choice(game, card).is_some() {
            Some("可选")
        } else {
            None
        };
        if let Some(label) = tribute_label {
            let tag = add_text(commands, face, label, 12.0, ACCENT, assets);
            commands.entity(tag).insert((
                Node {
                    position_type: PositionType::Absolute,
                    right: px(3),
                    top: px(3),
                    ..default()
                },
                TextLayout::no_wrap(),
                BackgroundColor(Color::BLACK.with_alpha(0.86)),
                Pickable::IGNORE,
            ));
        }
        commands.child(
            face,
            (
                HandCardSelectionOverlay {
                    index: placement.index,
                },
                Node {
                    position_type: PositionType::Absolute,
                    width: percent(100),
                    height: percent(100),
                    border_radius: BorderRadius::all(px(5)),
                    ..default()
                },
                BackgroundColor(Color::NONE),
                Pickable::IGNORE,
            ),
        );
        if placement.arranged {
            commands.child(
                slot,
                (
                    Node {
                        position_type: PositionType::Absolute,
                        top: px(placement.position.y - hit.min.y + placement.size.y + 3.0),
                        width: px(placement.size.x),
                        height: px(2),
                        ..default()
                    },
                    BackgroundColor(ACCENT.with_alpha(0.65)),
                    Pickable::IGNORE,
                ),
            );
        }
    }
}
