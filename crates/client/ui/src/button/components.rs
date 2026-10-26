use bevy::prelude::*;

/// The feature animates this control itself.
#[derive(Component, Default)]
pub struct CustomButtonMotion;

#[derive(Component, FromTemplate)]
pub struct ButtonHighlight {
    pub overlay: Entity,
    pub arrows: Option<[Entity; 2]>,
}

#[derive(Component)]
pub struct ButtonTint {
    pub normal: Color,
    pub hovered: Color,
    pub pressed: Color,
}

#[derive(Component)]
pub struct BackgroundButtonTint;
