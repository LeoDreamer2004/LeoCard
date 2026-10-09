use bevy::prelude::*;

#[derive(Clone, Copy, Component, Eq, PartialEq)]
pub(crate) enum CozyModalKind {
    Settings,
    Profile,
    MahjongFanGuide,
    TexasHandGuide,
    UnoExpansionSettings,
    Confirmation,
    UpdateDialog,
}

impl CozyModalKind {
    const COUNT: usize = Self::UpdateDialog as usize + 1;
}

#[derive(Component)]
pub(crate) struct CozyModalBackdrop(pub CozyModalKind);

#[derive(Component)]
pub(crate) struct CozyModalPanel(pub CozyModalKind);

#[derive(Resource, Default)]
pub(crate) struct ModalAnimations {
    progress: [f32; CozyModalKind::COUNT],
}

impl ModalAnimations {
    pub(crate) fn set(&mut self, kind: CozyModalKind, progress: f32) {
        self.progress[kind as usize] = progress;
    }

    pub(super) fn progress(&self, kind: CozyModalKind) -> f32 {
        self.progress[kind as usize]
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, SystemSet)]
pub(crate) enum ModalAnimationSet {
    Progress,
    Visuals,
}
