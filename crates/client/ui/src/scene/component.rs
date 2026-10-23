use bevy::ecs::template::template;
use bevy::prelude::*;

/// Preloaded handles and explicit component values can be composed without
/// inventing defaults for business actions or custom material components.
pub fn ui_component<T: Component + Clone>(value: T) -> impl Scene {
    template(move |_| Ok(value.clone()))
}
