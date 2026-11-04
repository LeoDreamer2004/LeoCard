use super::{ShopCategory, ShopUiAction, ShopUiState};
use crate::app::presentation::{TEXT, add_title};
use crate::app::runtime::UiAssets;
use crate::app::shell::{PageTransitionElement, UiAction};
use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;
use bevy::ui_widgets::Button;
use leocard_ui::{CustomButtonMotion, UiComposer, sliced_image, spawn_node};

#[derive(Component)]
pub(super) struct ShopCategories;

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
    for (index, category) in ShopCategory::ALL.into_iter().enumerate() {
        let position = if index == 0 {
            0
        } else if index + 1 == ShopCategory::ALL.len() {
            2
        } else {
            1
        };
        let selected = state.category == category;
        let texture = if selected {
            &assets.shop.tab_selected
        } else {
            &assets.shop.tabs[position]
        };
        let tab = commands.child(
            categories,
            (
                Button,
                CustomButtonMotion,
                UiAction::Shop(ShopUiAction::SelectCategory(category)),
                Node {
                    width: percent(100),
                    height: px(60),
                    min_height: px(60),
                    padding: UiRect::right(px(8)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                sliced_image(texture.clone(), 32.0, 0.5),
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
