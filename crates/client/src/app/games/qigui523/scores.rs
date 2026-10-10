use super::sort_cards_high_to_low;
use super::state::ScoreCardsPopupPlacement;
use super::{QIGUI_PORTRAIT_HEIGHT, QIGUI_PORTRAIT_WIDTH, qigui_plate_image};
use crate::app::presentation::{
    ACCENT, CardSize, MUTED, add_card_image, add_text, position_opponent_popup, spawn_node,
};
use crate::app::runtime::UiAssets;
use crate::app::shell::{PlayerGameScoreText, ScoreCaptureEffectState, displayed_captured_score};
use bevy::picking::Pickable;
use bevy::picking::hover::PickingInteraction;
use bevy::prelude::*;
use bevy::ui_widgets::Button;
use leocard_protocol::PlayerPublicState;
use leocard_qigui523::QiGuiCard;

#[derive(Component)]
pub(super) struct OwnScoreDetail(pub Entity);

pub(super) fn sync_own_score_detail(
    plaques: Query<(&PickingInteraction, &OwnScoreDetail), Changed<PickingInteraction>>,
    mut visibility: Query<&mut Visibility>,
) {
    for (interaction, detail) in &plaques {
        if let Ok(mut visible) = visibility.get_mut(detail.0) {
            *visible = if *interaction == PickingInteraction::None {
                Visibility::Hidden
            } else {
                Visibility::Visible
            };
        }
    }
}

pub(super) fn add_score_cards_popup(
    commands: &mut Commands,
    parent: Entity,
    player: &PlayerPublicState,
    cards: &[QiGuiCard],
    placement: ScoreCardsPopupPlacement,
    score_capture: &ScoreCaptureEffectState,
    assets: &UiAssets,
) -> Entity {
    let displayed_score = displayed_captured_score(score_capture, player.id, player.score);
    let ScoreCardsPopupPlacement::Opponent { side, above } = placement else {
        return add_own_score_plaque(commands, parent, player, cards, displayed_score, assets);
    };
    let mut node = Node {
        position_type: PositionType::Absolute,
        width: px(330),
        padding: UiRect::all(px(12)),
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Stretch,
        row_gap: px(6),
        ..default()
    };
    position_opponent_popup(&mut node, side);
    if above {
        node.top = Val::Auto;
        node.bottom = px(QIGUI_PORTRAIT_HEIGHT + 6.0);
    } else {
        node.top = px(QIGUI_PORTRAIT_HEIGHT + 6.0);
    }
    let popup = commands
        .spawn((
            node,
            qigui_plate_image(assets),
            GlobalZIndex(1500),
            Pickable::IGNORE,
        ))
        .id();
    commands.entity(parent).add_child(popup);
    if cards.is_empty() {
        add_text(commands, popup, "尚未获得分牌", 12.0, MUTED, assets);
    } else {
        add_score_card_rows(commands, popup, cards, assets);
    }
    popup
}

fn add_own_score_plaque(
    commands: &mut Commands,
    parent: Entity,
    player: &PlayerPublicState,
    cards: &[QiGuiCard],
    score: u32,
    assets: &UiAssets,
) -> Entity {
    let plaque = commands
        .spawn((
            Button,
            Node {
                position_type: PositionType::Absolute,
                left: px(10.0 + QIGUI_PORTRAIT_WIDTH + 8.0),
                bottom: px(25),
                width: px(140),
                height: px(64),
                padding: UiRect::axes(px(5), px(5)),
                align_items: AlignItems::Center,
                column_gap: px(11),
                ..default()
            },
        ))
        .id();
    commands.entity(parent).add_child(plaque);
    let icon = spawn_node(
        commands,
        plaque,
        Node {
            width: px(32),
            height: px(32),
            flex_shrink: 0.0,
            border: UiRect::all(px(1.5)),
            border_radius: BorderRadius::all(percent(50)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        None,
    );
    commands
        .entity(icon)
        .insert((BorderColor::all(ACCENT.with_alpha(0.75)), Pickable::IGNORE));
    add_text(commands, icon, "分", 18.0, ACCENT, assets);
    let values = spawn_node(
        commands,
        plaque,
        Node {
            flex_direction: FlexDirection::Column,
            row_gap: px(-3),
            ..default()
        },
        None,
    );
    add_text(commands, values, "得分", 11.0, MUTED, assets);
    let value = add_text(commands, values, score.to_string(), 30.0, ACCENT, assets);
    commands.entity(value).insert((
        PlayerGameScoreText::Own(player.id),
        TextShadow {
            offset: Vec2::new(1.5, 2.0),
            color: Color::BLACK.with_alpha(0.82),
        },
        Pickable::IGNORE,
    ));
    if !cards.is_empty() {
        let detail = commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: px(0),
                    bottom: px(68),
                    width: px(330),
                    padding: UiRect::all(px(12)),
                    flex_direction: FlexDirection::Column,
                    row_gap: px(6),
                    ..default()
                },
                qigui_plate_image(assets),
                Visibility::Hidden,
                GlobalZIndex(1500),
                Pickable::IGNORE,
            ))
            .id();
        commands.entity(plaque).add_child(detail);
        add_score_card_rows(commands, detail, cards, assets);
        commands.entity(plaque).insert(OwnScoreDetail(detail));
    }
    plaque
}

fn add_score_card_rows(
    commands: &mut Commands,
    popup: Entity,
    cards: &[QiGuiCard],
    assets: &UiAssets,
) {
    let mut displayed_cards = cards.to_vec();
    sort_cards_high_to_low(&mut displayed_cards);
    for chunk in displayed_cards.chunks(20) {
        let row = spawn_node(
            commands,
            popup,
            Node {
                width: percent(100),
                height: px(CardSize::Score.dimensions().1),
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::NoWrap,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::FlexStart,
                ..default()
            },
            None,
        );
        let last_card = chunk.len().saturating_sub(1);
        for (index, card) in chunk.iter().enumerate() {
            add_card_image(
                commands,
                row,
                *card,
                CardSize::Score,
                index,
                index == last_card,
                false,
                assets,
            );
        }
    }
}
