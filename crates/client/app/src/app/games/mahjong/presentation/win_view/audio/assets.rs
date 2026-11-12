use super::stage::ShowcaseClip;
use bevy::prelude::*;

pub(crate) struct MahjongShowcaseSounds {
    high: Handle<AudioSource>,
    background: Handle<AudioSource>,
    tiles: Handle<AudioSource>,
    glyph: Handle<AudioSource>,
}

impl MahjongShowcaseSounds {
    pub(crate) fn load(server: &AssetServer) -> Self {
        Self {
            high: server.load("audio/mahjong/showcase/high.mp3"),
            background: server.load("audio/mahjong/showcase/major-background.ogg"),
            tiles: server.load("audio/mahjong/showcase/tile-impact.ogg"),
            glyph: server.load("audio/mahjong/showcase/glyph-impact.ogg"),
        }
    }

    pub(super) fn clip(&self, clip: ShowcaseClip) -> Handle<AudioSource> {
        match clip {
            ShowcaseClip::High => &self.high,
            ShowcaseClip::Background => &self.background,
            ShowcaseClip::Tiles => &self.tiles,
            ShowcaseClip::Glyph => &self.glyph,
        }
        .clone()
    }
}
