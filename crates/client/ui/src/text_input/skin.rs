use bevy::prelude::*;

#[derive(Component, Clone)]
pub struct TextInputSkin {
    idle: ImageNode,
    focused: ImageNode,
}

impl TextInputSkin {
    pub fn new(idle: ImageNode, focused: ImageNode) -> Self {
        Self { idle, focused }
    }

    pub(super) fn image(&self, focused: bool) -> &ImageNode {
        if focused { &self.focused } else { &self.idle }
    }
}

/// Visual theme supplied by the application without coupling the editor to its assets.
pub struct TextInputStyle {
    pub font: Handle<Font>,
    pub text: Color,
    pub placeholder: Color,
    pub skin: TextInputSkin,
}
