use bevy::prelude::*;

#[derive(Resource, Default)]
pub(crate) struct DeveloperHandInput {
    #[cfg(feature = "developer")]
    pub value: String,
}

#[cfg(feature = "developer")]
#[derive(Component)]
pub(crate) struct DeveloperHandInputField;
