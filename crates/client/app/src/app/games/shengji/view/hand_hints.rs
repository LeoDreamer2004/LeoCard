use crate::app::presentation::{CardSize, TEXT, add_text};
use crate::app::runtime::UiAssets;
use bevy::picking::Pickable;
use bevy::prelude::*;
use leocard_shengji::{ShengjiCard, ShengjiComponent, ShengjiTrump, hand_sequences};
use leocard_ui::{UiComposer, spawn_node};

pub(super) fn sequence_labels(
    cards: &[ShengjiCard],
    trump: Option<ShengjiTrump>,
) -> Vec<Option<&'static str>> {
    let mut labels = vec![None; cards.len()];
    let Some(trump) = trump else { return labels };
    for sequence in hand_sequences(cards, trump).unwrap_or_default() {
        let label = match &sequence {
            ShengjiComponent::Tractor { .. } => "拖拉机",
            ShengjiComponent::Titanic { .. } => "泰坦尼克",
            ShengjiComponent::Spaceship { .. } => "宇宙飞船",
            _ => continue,
        };
        let group = sequence.cards();
        if let Some(index) = cards.iter().rposition(|card| group.contains(card)) {
            labels[index] = Some(label);
        }
    }
    labels
}

pub(super) fn add_hand_sequence_hint(
    commands: &mut UiComposer,
    face: Entity,
    label: &str,
    visible_width: f32,
    assets: &UiAssets,
) {
    let width = CardSize::ShengjiHand.dimensions().0;
    let hint = spawn_node(
        commands,
        face,
        Node {
            position_type: PositionType::Absolute,
            right: px(width - visible_width + 3.0),
            bottom: px(7),
            width: px(72),
            height: px(22),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            border_radius: BorderRadius::all(px(3)),
            ..default()
        },
        Some(Color::srgba(0.18, 0.18, 0.20, 0.72)),
    );
    // The badge spans the exposed bottoms of preceding cards, above their faces.
    commands
        .entity(hint)
        .insert((GlobalZIndex(150), Pickable::IGNORE));
    let text = add_text(commands, hint, label, 13.0, TEXT, assets);
    commands.entity(text).insert(Pickable::IGNORE);
}
