//! 卡牌、按钮、面板和文本等基础控件。

use super::super::{
    ACCENT, AutoPlayOverlay, PanelSkin, TABLE_CARD_REVEAL, TABLE_SCORE_CARD_REVEAL, TEXT,
};
use crate::app::presentation::CardSize;
use crate::app::runtime::UiAssets;
use crate::app::shell::{SocialUiAction, UiAction};
use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use leocard_qigui523::QiGuiCard;

const SCORE_CARD_REVEAL: f32 = 12.0;
const FINISHED_HAND_CARD_REVEAL: f32 = 14.4;

#[expect(
    clippy::too_many_arguments,
    reason = "the card-image primitive keeps card identity, geometry, and assets explicit"
)]
pub(crate) fn add_card_image(
    commands: &mut Commands,
    parent: Entity,
    card: QiGuiCard,
    size: CardSize,
    index: usize,
    is_last: bool,
    initially_hidden: bool,
    assets: &UiAssets,
) -> Entity {
    let (width, height) = size.dimensions();
    let reveal = match size {
        CardSize::Score => SCORE_CARD_REVEAL,
        CardSize::TableScore => TABLE_SCORE_CARD_REVEAL,
        CardSize::FinishedHand => FINISHED_HAND_CARD_REVEAL,
        CardSize::ShengjiHand | CardSize::QiGuiHand | CardSize::QiGuiSeat => TABLE_CARD_REVEAL,
    };
    let image = assets
        .playing_cards
        .cards
        .get(&(card.rank(), card.suit()))
        .expect("all valid card faces are preloaded")
        .clone();
    let entity = commands
        .spawn((
            Node {
                width: px(width),
                height: px(height),
                margin: UiRect::right(px(if is_last { 0.0 } else { reveal - width })),
                border_radius: BorderRadius::all(px(4)),
                ..default()
            },
            ImageNode::new(image).with_color(if initially_hidden {
                Color::NONE
            } else {
                Color::WHITE
            }),
            ZIndex(index as i32),
        ))
        .id();
    commands.entity(parent).add_child(entity);
    entity
}

/// 托管时覆盖整条手牌与操作区。蒙版本身是唯一可点击目标，因此其后的牌、
/// 操作按钮和聊天抽屉在这个区域内都不会收到指针事件。
pub(crate) fn add_auto_play_overlay(commands: &mut Commands, parent: Entity, assets: &UiAssets) {
    let overlay = commands
        .spawn((
            Button,
            UiAction::Social(SocialUiAction::ToggleAutoPlay),
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                right: px(0),
                bottom: px(0),
                height: px(190),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: px(5),
                ..default()
            },
            BackgroundColor(Color::BLACK.with_alpha(0.82)),
            GlobalZIndex(1900),
            FocusPolicy::Block,
            AutoPlayOverlay,
        ))
        .id();
    commands.entity(parent).add_child(overlay);
    let title = add_text(commands, overlay, "您已托管", 27.0, ACCENT, assets);
    commands.entity(title).insert((
        FocusPolicy::Pass,
        TextShadow {
            offset: Vec2::new(1.2, 1.5),
            color: Color::BLACK.with_alpha(0.92),
        },
    ));
    let detail = add_text(commands, overlay, "点击此处取消", 14.0, TEXT, assets);
    commands.entity(detail).insert(FocusPolicy::Pass);
}

pub(crate) fn add_panel(
    commands: &mut Commands,
    parent: Entity,
    mut node: Node,
    color: Color,
    skin: PanelSkin,
    assets: &UiAssets,
) -> Entity {
    node.padding = UiRect::all(px(match skin {
        PanelSkin::Window => 28.0,
        PanelSkin::Section => 24.0,
        PanelSkin::Popup => 22.0,
    }));
    node.border = UiRect::all(px(1));
    node.border_radius = BorderRadius::all(px(8));
    let entity = spawn_node(commands, parent, node, Some(color));
    decorate_panel_skin(commands, entity, skin, assets);
    entity
}

/// 给任意布局节点叠加独立的九宫格面板皮肤。玩家框仍使用专用贴图；这里仅
/// 服务于主窗口、内容分区和小型提示框，避免随尺寸拉伸边角与描边。
pub(crate) fn decorate_panel_skin(
    commands: &mut Commands,
    panel: Entity,
    skin: PanelSkin,
    assets: &UiAssets,
) -> Entity {
    let corner_scale = match skin {
        PanelSkin::Window => 0.42,
        PanelSkin::Section => 0.35,
        PanelSkin::Popup => 0.30,
    };
    commands
        .entity(panel)
        .insert((BackgroundColor(Color::NONE), BorderColor::all(Color::NONE)));
    let texture = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                right: px(0),
                top: px(0),
                bottom: px(0),
                ..default()
            },
            {
                let mut image = ImageNode::new(assets.home.panel.clone())
                    .with_mode(NodeImageMode::Sliced(TextureSlicer {
                        border: BorderRect::all(80.0),
                        center_scale_mode: SliceScaleMode::Stretch,
                        sides_scale_mode: SliceScaleMode::Stretch,
                        max_corner_scale: corner_scale,
                    }))
                    .with_color(Color::WHITE);
                image.visual_box = bevy::ui::VisualBox::BorderBox;
                image
            },
            ZIndex(-1),
            FocusPolicy::Pass,
        ))
        .id();
    commands.entity(panel).add_child(texture);
    texture
}

pub(crate) fn spawn_node(
    commands: &mut Commands,
    parent: Entity,
    node: Node,
    background: Option<Color>,
) -> Entity {
    let mut entity = commands.spawn(node);
    if let Some(color) = background {
        entity.insert(BackgroundColor(color));
    }
    let entity = entity.id();
    commands.entity(parent).add_child(entity);
    entity
}

pub(crate) fn add_section_title(
    commands: &mut Commands,
    parent: Entity,
    text: impl Into<String>,
    assets: &UiAssets,
) {
    add_text(commands, parent, text, 21.0, ACCENT, assets);
}

pub(crate) fn add_text(
    commands: &mut Commands,
    parent: Entity,
    text: impl Into<String>,
    size: f32,
    color: Color,
    assets: &UiAssets,
) -> Entity {
    let entity = commands
        .spawn((
            Text::new(text),
            TextFont::from_font_size(size).with_font(assets.font.clone()),
            TextColor(color),
        ))
        .id();
    commands.entity(parent).add_child(entity);
    entity
}
