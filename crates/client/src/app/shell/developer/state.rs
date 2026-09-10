use bevy::prelude::*;

#[derive(Resource, Default)]
pub(crate) struct DeveloperHandInput {
    pub value: String,
    pub focused: bool,
}

#[derive(Component)]
pub(crate) struct DeveloperHandInputText {
    pub placeholder: &'static str,
}

#[derive(Component)]
pub(crate) struct DeveloperHandInputField;
