//! 卡牌、按钮、面板和文本等基础控件。

use super::super::{
    ACCENT, AutoPlayOverlay, PanelSkin, TABLE_CARD_REVEAL, TABLE_SCORE_CARD_REVEAL, TEXT,
};
use super::sliced_image;
use crate::app::presentation::{CardSize, UiComposer, ui_component};
use crate::app::runtime::{CardFace, UiAssets};
use crate::app::shell::{SocialUiAction, UiAction};
use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy::text::FontSourceTemplate;
use bevy::ui_widgets::Button;

const SCORE_CARD_REVEAL: f32 = 12.0;
const FINISHED_HAND_CARD_REVEAL: f32 = 14.4;

#[expect(
    clippy::too_many_arguments,
    reason = "the card-image primitive keeps card identity, geometry, and assets explicit"
)]
pub(crate) fn add_card_image(
    commands: &mut UiComposer,
    parent: Entity,
    card: impl Into<CardFace>,
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
    let image = assets.playing_cards.face(card);
    commands.child(
        parent,
        (
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
        ),
    )
}

/// 托管时覆盖整条手牌与操作区。蒙版本身是唯一可点击目标，因此其后的牌、
/// 操作按钮和聊天抽屉在这个区域内都不会收到指针事件。
pub(crate) fn add_auto_play_overlay(commands: &mut UiComposer, parent: Entity, assets: &UiAssets) {
    let overlay = commands.child(
        parent,
        (
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
            Pickable::default(),
            AutoPlayOverlay,
        ),
    );
    let title = add_text(commands, overlay, "您已托管", 27.0, ACCENT, assets);
    commands.entity(title).insert((
        Pickable::IGNORE,
        TextShadow {
            offset: Vec2::new(1.2, 1.5),
            color: Color::BLACK.with_alpha(0.92),
        },
    ));
    let detail = add_text(commands, overlay, "点击此处取消", 14.0, TEXT, assets);
    commands.entity(detail).insert(Pickable::IGNORE);
}

/// Reuse the rounded card skin at both entry and compact popup sizes.
pub(crate) fn card_background_image(texture: Handle<Image>, corner_scale: f32) -> ImageNode {
    sliced_image(texture, 16.0, corner_scale)
}

/// 给任意布局节点叠加独立的九宫格面板皮肤。玩家框仍使用专用贴图；这里仅
/// 服务于主窗口、内容分区和小型提示框，避免随尺寸拉伸边角与描边。
pub(crate) fn decorate_panel_skin(
    commands: &mut UiComposer,
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
    let image = sliced_image(assets.home.panel.clone(), 80.0, corner_scale);
    commands.mount(
        panel,
        "skin",
        bsn! {
                Node {
                    position_type: PositionType::Absolute,
                    left: px(0),
                    right: px(0),
                    top: px(0),
                    bottom: px(0),
                }
                @ui_component(image)
                ZIndex(-1)
                Pickable::IGNORE
        },
    )
}

#[track_caller]
pub(crate) fn spawn_node(
    commands: &mut UiComposer,
    parent: Entity,
    node: Node,
    background: Option<Color>,
) -> Entity {
    let background = background.map(|color| bsn! { BackgroundColor(color) });
    commands.node(parent, bsn! { node @background })
}

pub(crate) fn add_section_title(
    commands: &mut UiComposer,
    parent: Entity,
    text: impl Into<String>,
    assets: &UiAssets,
) {
    add_text(commands, parent, text, 21.0, ACCENT, assets);
}

#[track_caller]
pub(crate) fn add_text(
    commands: &mut UiComposer,
    parent: Entity,
    text: impl Into<String>,
    size: f32,
    color: Color,
    assets: &UiAssets,
) -> Entity {
    commands.node(parent, text_scene(assets, text, size, color))
}

pub(crate) fn text_scene(
    assets: &UiAssets,
    text: impl Into<String>,
    size: f32,
    color: Color,
) -> impl Scene {
    let text = Text::new(text);
    let font = assets.font.clone();
    let font_source = FontSourceTemplate::Handle(font.into());
    bsn! {
            text
            TextFont { font: font_source, font_size: px(size) }
            TextColor(color)
    }
}
