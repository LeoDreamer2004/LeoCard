use bevy::prelude::*;

#[derive(Default)]
pub(crate) struct PokerChipSounds {
    lay: Vec<Handle<AudioSource>>,
    handle: Vec<Handle<AudioSource>>,
    collide: Vec<Handle<AudioSource>>,
    stack: Vec<Handle<AudioSource>>,
}

#[derive(Clone, Copy)]
pub(crate) enum PokerChipSoundKind {
    Lay,
    Handle,
    Collide,
    Stack,
}

impl PokerChipSounds {
    pub(crate) fn load(server: &AssetServer) -> Self {
        let directory = "vendor/kenney/casino-audio/Audio";
        Self {
            lay: numbered_table_sounds(server, directory, "chip-lay", 3),
            handle: numbered_table_sounds(server, directory, "chips-handle", 6),
            collide: numbered_table_sounds(server, directory, "chips-collide", 4),
            stack: numbered_table_sounds(server, directory, "chips-stack", 6),
        }
    }

    pub(crate) fn variants(&self, kind: PokerChipSoundKind) -> &[Handle<AudioSource>] {
        match kind {
            PokerChipSoundKind::Lay => &self.lay,
            PokerChipSoundKind::Handle => &self.handle,
            PokerChipSoundKind::Collide => &self.collide,
            PokerChipSoundKind::Stack => &self.stack,
        }
    }
}

pub(crate) fn numbered_table_sounds(
    server: &AssetServer,
    directory: &str,
    prefix: &str,
    count: usize,
) -> Vec<Handle<AudioSource>> {
    (1..=count)
        .map(|index| server.load(format!("{directory}/{prefix}-{index}.ogg")))
        .collect()
}
