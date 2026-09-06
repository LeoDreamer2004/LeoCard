use super::*;
use leocard_protocol::UnoSnapshot;
use leocard_uno::UnoFlipSide;

#[allow(clippy::too_many_arguments)]
pub(super) fn spawn_uno_flip_effect(
    commands: &mut Commands,
    layer: Entity,
    layer_size: Vec2,
    previous_game: Option<&UnoSnapshot>,
    side: UnoFlipSide,
    targets: &Query<(Entity, &UnoFlipTarget, &ImageNode, &UiTransform)>,
    assets: &UiAssets,
) {
    let overlay = commands
        .spawn((
            UnoFlipOverlay { elapsed: 0.0 },
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                top: px(0),
                width: px(layer_size.x),
                height: px(layer_size.y),
                ..default()
            },
            BackgroundColor(match side {
                UnoFlipSide::Light => Color::srgba(1.0, 0.91, 0.53, 0.0),
                UnoFlipSide::Dark => Color::srgba(0.20, 0.08, 0.48, 0.0),
            }),
            GlobalZIndex(1490),
            FocusPolicy::Pass,
        ))
        .id();
    commands.entity(layer).add_child(overlay);

    let title = spawn_node(
        commands,
        overlay,
        Node {
            position_type: PositionType::Absolute,
            left: px(layer_size.x * 0.5 - 150.0),
            top: px(layer_size.y * 0.5 - 38.0),
            width: px(300),
            height: px(76),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            border_radius: BorderRadius::all(px(38)),
            ..default()
        },
        Some(Color::BLACK.with_alpha(0.68)),
    );
    commands.entity(title).insert((
        Outline::new(px(2.0), px(1.0), Color::WHITE.with_alpha(0.52)),
        BoxShadow::new(Color::BLACK.with_alpha(0.58), px(2), px(5), px(0), px(10)),
        GlobalZIndex(1492),
        FocusPolicy::Pass,
    ));
    add_text(
        commands,
        title,
        match side {
            UnoFlipSide::Light => "翻至亮面",
            UnoFlipSide::Dark => "翻至暗面",
        },
        27.0,
        Color::WHITE,
        assets,
    );

    let Some(previous) = previous_game else {
        return;
    };
    for (entity, target, image, transform) in targets {
        let (old_card, delay, pile) = match *target {
            UnoFlipTarget::Own(index) => (
                previous.your_hand.get(index).copied(),
                UNO_PLAY_CARD_DURATION + index as f32 * 0.018,
                false,
            ),
            UnoFlipTarget::Opponent { player, index } => (
                previous
                    .players
                    .iter()
                    .find(|candidate| candidate.id == player)
                    .and_then(|candidate| candidate.inactive_hand.get(index))
                    .copied(),
                UNO_PLAY_CARD_DURATION + player.0 as f32 * 0.025 + index as f32 * 0.018,
                false,
            ),
            UnoFlipTarget::DrawPile(index) => {
                let old_index = previous
                    .draw_pile_inactive_cards
                    .len()
                    .saturating_sub(1 + index);
                (
                    previous.draw_pile_inactive_cards.get(old_index).copied(),
                    UNO_PLAY_CARD_DURATION + 0.10,
                    true,
                )
            }
            UnoFlipTarget::DiscardPile(index) => (
                previous.discard_pile.get(index).copied(),
                UNO_PLAY_CARD_DURATION + 0.10,
                true,
            ),
        };
        let Some(old_card) = old_card else {
            continue;
        };
        let old_face = uno_card_handle(assets, old_card);
        let mut old_image = image.clone();
        let new_face = std::mem::replace(&mut old_image.image, old_face.clone());
        commands.entity(entity).insert((
            UnoFlipCard {
                elapsed: 0.0,
                delay,
                old_face,
                new_face,
                swapped: false,
                base_transform: *transform,
                pile,
            },
            old_image,
        ));
    }
}
