use super::sort_cards_high_to_low;
use super::state::ScoreCardsPopupPlacement;
use crate::app::presentation::CardSize;
use crate::app::presentation::{
    ACCENT, MUTED, add_card_image, add_text, position_opponent_popup, spawn_node,
};
use crate::app::runtime::UiAssets;
use crate::app::shell::{PlayerGameScoreText, ScoreCaptureEffectState, displayed_captured_score};
use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use leocard_protocol::PlayerPublicState;
use leocard_qigui523::QiGuiCard;

pub(super) fn add_score_cards_popup(
    commands: &mut Commands,
    parent: Entity,
    player: &PlayerPublicState,
    cards: &[QiGuiCard],
    placement: ScoreCardsPopupPlacement,
    score_capture: &ScoreCaptureEffectState,
    assets: &UiAssets,
) -> Entity {
    let mut node = Node {
        position_type: PositionType::Absolute,
        width: px(330),
        padding: UiRect::all(px(6)),
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Stretch,
        row_gap: px(3),
        border: UiRect::all(px(1)),
        border_radius: BorderRadius::all(px(8)),
        ..default()
    };
    match placement {
        ScoreCardsPopupPlacement::Opponent(side) => position_opponent_popup(&mut node, side),
        ScoreCardsPopupPlacement::Own => {
            node.left = px(10);
            node.bottom = px(64);
            node.width = px(410);
            node.min_height = px(58);
            node.padding = UiRect::new(px(6), px(76), px(6), px(6));
        }
    }
    let popup = spawn_node(commands, parent, node, Some(Color::BLACK.with_alpha(0.30)));
    commands.entity(popup).insert((
        BorderColor::all(ACCENT.with_alpha(0.72)),
        GlobalZIndex(1500),
        FocusPolicy::Pass,
    ));
    let displayed_score = displayed_captured_score(score_capture, player.id, player.score);
    match placement {
        ScoreCardsPopupPlacement::Own => {
            let score_area = spawn_node(
                commands,
                popup,
                Node {
                    position_type: PositionType::Absolute,
                    right: px(5),
                    top: px(4),
                    bottom: px(4),
                    width: px(66),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    row_gap: px(-2),
                    ..default()
                },
                None,
            );
            commands
                .entity(score_area)
                .insert((ZIndex(3), FocusPolicy::Pass));
            add_text(commands, score_area, "得分", 10.0, MUTED, assets);
            let score = add_text(
                commands,
                score_area,
                displayed_score.to_string(),
                30.0,
                ACCENT,
                assets,
            );
            commands.entity(score).insert((
                PlayerGameScoreText::Own(player.id),
                TextShadow {
                    offset: Vec2::new(1.5, 2.0),
                    color: Color::BLACK.with_alpha(0.82),
                },
            ));
        }
        ScoreCardsPopupPlacement::Opponent(_) => {
            let title = format!(
                "{} 的分牌 · {} 分 · {} 张",
                player.name,
                displayed_score,
                cards.len()
            );
            add_text(commands, popup, title, 12.0, ACCENT, assets);
        }
    }
    if cards.is_empty() {
        if matches!(placement, ScoreCardsPopupPlacement::Opponent(_)) {
            add_text(commands, popup, "尚未获得分牌", 12.0, MUTED, assets);
        }
        return popup;
    }

    let mut displayed_cards = cards.to_vec();
    sort_cards_high_to_low(&mut displayed_cards);
    for chunk in displayed_cards.chunks(24) {
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
    popup
}
