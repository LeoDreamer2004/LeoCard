use super::{
    UNO_DISCARD_OFFSETS, UNO_FLYING_CARD_HEIGHT, UNO_FLYING_CARD_WIDTH, UNO_PLAY_CARD_DURATION,
    UnoAssets, UnoDiscardCard, UnoFlyingCard, UnoPresentationState, uno_card_handle,
};
use crate::app::runtime::ClientResource;
use crate::app::shell::PlayerAvatarAnchor;
use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use leocard_protocol::{PlayerId, UnoEvent};
use leocard_uno::{UnoCard, UnoColor, UnoFace};

pub(crate) fn uno_anchor_in_layer(
    node: &ComputedNode,
    transform: &UiGlobalTransform,
    layer_node: &ComputedNode,
    layer_transform: &UiGlobalTransform,
) -> Option<Vec2> {
    if node.size().min_element() <= 1.0 || layer_node.size().min_element() <= 1.0 {
        return None;
    }
    let inverse = layer_transform.try_inverse()?;
    Some(
        (inverse.transform_point2(transform.to_scale_angle_translation().2)
            + layer_node.size() * 0.5)
            * layer_node.inverse_scale_factor(),
    )
}

pub(super) fn uno_player_anchor_in_layer(
    player: PlayerId,
    layer_node: &ComputedNode,
    layer_transform: &UiGlobalTransform,
    anchors: &Query<(&PlayerAvatarAnchor, &ComputedNode, &UiGlobalTransform)>,
) -> Option<Vec2> {
    let (_, node, transform) = anchors.iter().find(|(anchor, _, _)| anchor.0 == player)?;
    uno_anchor_in_layer(node, transform, layer_node, layer_transform)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn spawn_uno_flying_card(
    commands: &mut Commands,
    layer: Entity,
    image: Handle<Image>,
    played_card: Option<UnoCard>,
    source: Vec2,
    target: Vec2,
    delay: f32,
    draw_animation: bool,
    index: usize,
    target_angle: f32,
) {
    let fan = (index as f32 % 7.0) - 3.0;
    let staging = source + Vec2::new(fan * 7.0, -32.0 - fan.abs() * 2.0);
    let midpoint = (source + target) * 0.5;
    let control = midpoint + Vec2::new(fan * 10.0, -95.0);
    let card = commands
        .spawn((
            UnoFlyingCard {
                elapsed: 0.0,
                delay,
                start: source,
                staging,
                control,
                target,
                duration: if draw_animation {
                    0.78
                } else {
                    UNO_PLAY_CARD_DURATION
                },
                draw_animation,
                played_card,
                start_angle: fan * 2.8,
                end_angle: if draw_animation {
                    fan * -1.4
                } else {
                    target_angle
                },
            },
            Node {
                position_type: PositionType::Absolute,
                left: px(source.x - UNO_FLYING_CARD_WIDTH * 0.5),
                top: px(source.y - UNO_FLYING_CARD_HEIGHT * 0.5),
                width: px(UNO_FLYING_CARD_WIDTH),
                height: px(UNO_FLYING_CARD_HEIGHT),
                ..default()
            },
            ImageNode::new(image).with_color(Color::WHITE.with_alpha(0.0)),
            UiTransform::from_scale(Vec2::splat(0.76)),
            BoxShadow::new(Color::BLACK.with_alpha(0.42), px(2), px(5), px(0), px(6)),
            GlobalZIndex(1450),
            FocusPolicy::Pass,
        ))
        .id();
    commands.entity(layer).add_child(card);
}

pub(super) fn spawn_uno_draw_cards(
    commands: &mut Commands,
    layer: Entity,
    source: Vec2,
    target: Vec2,
    count: u16,
    base_delay: f32,
    assets: &UnoAssets,
) {
    spawn_uno_draw_cards_with_interval(
        commands,
        layer,
        source,
        target,
        count,
        &[],
        base_delay,
        0.045,
        assets,
    );
}

#[allow(clippy::too_many_arguments)]
pub(super) fn spawn_uno_draw_cards_with_backs(
    commands: &mut Commands,
    layer: Entity,
    source: Vec2,
    target: Vec2,
    count: u16,
    card_backs: &[UnoCard],
    base_delay: f32,
    assets: &UnoAssets,
) {
    spawn_uno_draw_cards_with_interval(
        commands, layer, source, target, count, card_backs, base_delay, 0.045, assets,
    );
}

#[allow(clippy::too_many_arguments)]
pub(super) fn spawn_uno_draw_cards_with_interval(
    commands: &mut Commands,
    layer: Entity,
    source: Vec2,
    target: Vec2,
    count: u16,
    card_backs: &[UnoCard],
    base_delay: f32,
    interval: f32,
    assets: &UnoAssets,
) {
    for index in 0..usize::from(count.min(16)) {
        let image = card_backs
            .get(index)
            .copied()
            .map(|card| uno_card_handle(assets, card))
            .unwrap_or_else(|| assets.card_back.clone());
        spawn_uno_flying_card(
            commands,
            layer,
            image,
            None,
            source,
            target,
            base_delay + index as f32 * interval,
            true,
            index,
            0.0,
        );
    }
}

pub(super) fn spawn_uno_transfer_cards(
    commands: &mut Commands,
    layer: Entity,
    source: Vec2,
    target: Vec2,
    count: u16,
    base_delay: f32,
    assets: &UnoAssets,
) {
    for index in 0..usize::from(count.min(6)) {
        spawn_uno_flying_card(
            commands,
            layer,
            assets.card_back.clone(),
            None,
            source,
            target,
            base_delay + index as f32 * 0.045,
            true,
            index,
            0.0,
        );
    }
}

/// 权威快照会在出牌动画结束前把新牌放进弃牌堆。动画等待布局或正在飞行时，
/// 暂时隐藏对应实体牌；双牌会同时隐藏，落地后再一起恢复。
pub(crate) fn sync_uno_discard_reveal(
    client: Option<Res<ClientResource>>,
    presentation: Res<UnoPresentationState>,
    flights: Query<&UnoFlyingCard>,
    mut discards: Query<(&UnoDiscardCard, &mut Visibility)>,
) {
    let has_game = client
        .as_deref()
        .and_then(|client| client.0.model().uno_game())
        .is_some();
    let active_cards = flights
        .iter()
        .filter_map(|flight| flight.played_card)
        .collect::<Vec<_>>();
    for (discard, mut visibility) in &mut discards {
        let hidden = has_game
            && uno_discard_should_be_hidden(
                discard.0,
                &presentation,
                active_cards.iter().copied().map(Some),
            );
        let expected = if hidden {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
        if *visibility != expected {
            *visibility = expected;
        }
    }
}

pub(crate) fn uno_discard_should_be_hidden(
    top: UnoCard,
    presentation: &UnoPresentationState,
    mut active_cards: impl Iterator<Item = Option<UnoCard>>,
) -> bool {
    presentation
        .events
        .iter()
        .any(|event| matches!(event, UnoEvent::CardPlayed { card, .. } if *card == top))
        || active_cards.any(|card| card == Some(top))
}

/// 弃牌堆只同步末尾六张牌；第七张加入时窗口会整体向前滑动，因此不能用数组
/// 下标决定姿态。物理牌标识在整局内稳定，用它分配偏移可让仍在堆中的旧牌原地不动。
pub(crate) fn uno_discard_pose(card: UnoCard) -> (f32, f32, f32) {
    let color = match card.color() {
        Some(UnoColor::Red) => 0usize,
        Some(UnoColor::Yellow) => 1,
        Some(UnoColor::Green) => 2,
        Some(UnoColor::Blue) => 3,
        Some(UnoColor::Pink) => 4,
        Some(UnoColor::Teal) => 5,
        Some(UnoColor::Orange) => 6,
        Some(UnoColor::Purple) => 7,
        None => 8,
    };
    let face = match card.face() {
        UnoFace::Number(value) => usize::from(value),
        UnoFace::DrawTwo => 10,
        UnoFace::Reverse => 11,
        UnoFace::Skip => 12,
        UnoFace::Wild => 13,
        UnoFace::DarkWild => 39,
        UnoFace::WildDrawFour => 14,
        UnoFace::SwapOne => 15,
        UnoFace::RefreshHand => 16,
        UnoFace::WildForceTrade => 17,
        UnoFace::WildPassHands => 18,
        UnoFace::ReverseDrawTwo => 19,
        UnoFace::ReverseSkip => 20,
        UnoFace::WildPowerReverse => 21,
        UnoFace::WildNoU => 22,
        UnoFace::StackOne => 23,
        UnoFace::StackTwo => 24,
        UnoFace::WildStackThree => 25,
        UnoFace::WildStackNumber => 26,
        UnoFace::DrawFour => 27,
        UnoFace::SkipEveryone => 28,
        UnoFace::DiscardAll => 29,
        UnoFace::WildReverseDrawFour => 30,
        UnoFace::WildDrawSix => 31,
        UnoFace::WildDrawTen => 32,
        UnoFace::WildColorRoulette => 33,
        UnoFace::DrawOne => 34,
        UnoFace::DrawFive => 35,
        UnoFace::Flip => 36,
        UnoFace::WildDrawTwo => 37,
        UnoFace::WildDrawColor => 38,
    };
    let key = color * 47 + face * 19 + usize::from(card.copy()) * 31;
    UNO_DISCARD_OFFSETS[(key ^ (key >> 3)) % UNO_DISCARD_OFFSETS.len()]
}
