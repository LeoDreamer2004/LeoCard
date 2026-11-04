use super::rarity::add_rarity_backdrop;
use super::{ShopCategory, ShopUiAction, ShopUiState};
use crate::app::presentation::{TEXT, add_text, add_title};
use crate::app::runtime::UiAssets;
use crate::app::shell::{PageTransitionElement, UiAction};
use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;
use bevy::ui_widgets::Button;
use leocard_client::ShopRarity;
use leocard_ui::{CustomButtonMotion, UiComposer, decorate_panel, sliced_image, spawn_node};

pub(super) const PRODUCT_CARD_WIDTH: f32 = 216.0;

#[derive(Component)]
pub(super) struct ShopCategories;

pub(super) fn add_product_card(
    commands: &mut UiComposer,
    parent: Entity,
    assets: &UiAssets,
) -> Entity {
    let card = spawn_node(
        commands,
        parent,
        Node {
            width: px(PRODUCT_CARD_WIDTH),
            max_width: percent(100),
            flex_shrink: 0.0,
            padding: UiRect::all(px(14)),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: px(10),
            ..default()
        },
        None,
    );
    decorate_panel(
        commands,
        card,
        sliced_image(assets.shop.product_card.clone(), 24.0, 0.5),
    );
    card
}

pub(super) fn add_product_preview(
    commands: &mut UiComposer,
    parent: Entity,
    assets: &UiAssets,
    rarity: ShopRarity,
) -> Entity {
    let preview = spawn_node(
        commands,
        parent,
        Node {
            width: px(144),
            height: px(144),
            padding: UiRect::all(px(12)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        None,
    );
    let texture = match rarity {
        ShopRarity::Common => &assets.shop.product_preview_common,
        ShopRarity::Fine => &assets.shop.product_preview_fine,
        ShopRarity::Epic => &assets.shop.product_preview_epic,
    };
    decorate_panel(commands, preview, ImageNode::new(texture.clone()));
    add_rarity_backdrop(commands, preview, rarity);
    preview
}

pub(super) fn add_price(commands: &mut UiComposer, button: Entity, price: u32, assets: &UiAssets) {
    commands.entity(button).insert(Node {
        width: percent(100),
        height: px(44),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        column_gap: px(5),
        ..default()
    });
    add_product_icon(commands, button, assets.shop.coin.clone(), 21.0);
    add_text(commands, button, price.to_string(), 17.0, TEXT, assets);
}

pub(super) fn add_product_icon(
    commands: &mut UiComposer,
    parent: Entity,
    image: Handle<Image>,
    size: f32,
) {
    commands.child(
        parent,
        (
            Node {
                width: px(size),
                height: px(size),
                flex_shrink: 0.0,
                ..default()
            },
            ImageNode::new(image),
            Pickable::IGNORE,
        ),
    );
}

pub(super) fn add_categories(
    commands: &mut UiComposer,
    parent: Entity,
    assets: &UiAssets,
    state: &ShopUiState,
) {
    let categories = spawn_node(
        commands,
        parent,
        Node {
            width: px(116),
            flex_shrink: 0.0,
            flex_direction: FlexDirection::Column,
            padding: UiRect::top(px(12)),
            row_gap: px(4),
            overflow: Overflow::scroll_y(),
            ..default()
        },
        None,
    );
    commands.entity(categories).insert((
        PageTransitionElement::left(1),
        UiTransform::IDENTITY,
        ShopCategories,
        ScrollPosition(Vec2::ZERO),
        RelativeCursorPosition::default(),
        ZIndex(1),
    ));
    for category in ShopCategory::ALL {
        let selected = state.category == category;
        let (texture, tint) = if selected {
            (&assets.shop.tab_selected, Color::WHITE)
        } else {
            (&assets.shop.tab_unselected, Color::srgb(0.84, 0.84, 0.84))
        };
        let tab = commands.child(
            categories,
            (
                Button,
                CustomButtonMotion,
                UiAction::Shop(ShopUiAction::SelectCategory(category)),
                Node {
                    align_self: AlignSelf::Stretch,
                    // Align the unselected texture's right stroke with the page frame's left stroke.
                    margin: UiRect::right(px(if selected { 0 } else { 2 })),
                    height: px(60),
                    min_height: px(60),
                    padding: UiRect::right(px(8)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                sliced_image(texture.clone(), 32.0, 0.5).with_color(tint),
            ),
        );
        let label = add_title(
            commands,
            tab,
            category.title(),
            20.0,
            if selected {
                Color::srgb(0.13, 0.14, 0.17)
            } else {
                TEXT
            },
            assets,
        );
        commands.entity(label).insert(Pickable::IGNORE);
    }
}
