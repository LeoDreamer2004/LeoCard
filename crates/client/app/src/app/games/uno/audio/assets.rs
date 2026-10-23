use bevy::prelude::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum UnoSoundKind {
    CardThrow,
    CardLand,
    CardDraw,
    Penalty,
    Skip,
    Reverse,
    PaletteOpen,
    PaletteSelect,
    UnoCall,
    UnoAccent,
    Report,
    Challenge,
    Success,
    Failure,
}

impl UnoSoundKind {
    pub(super) fn is_card(self) -> bool {
        matches!(self, Self::CardThrow | Self::CardLand | Self::CardDraw)
    }

    pub(super) fn is_prominent(self) -> bool {
        matches!(
            self,
            Self::UnoCall
                | Self::UnoAccent
                | Self::Report
                | Self::Challenge
                | Self::Success
                | Self::Failure
        )
    }
}

#[derive(Default)]
pub(crate) struct UnoSoundAssets {
    pub(super) select: Vec<Handle<AudioSource>>,
    card_throw: Vec<Handle<AudioSource>>,
    card_land: Vec<Handle<AudioSource>>,
    card_draw: Vec<Handle<AudioSource>>,
    penalty: Vec<Handle<AudioSource>>,
    skip: Vec<Handle<AudioSource>>,
    reverse: Vec<Handle<AudioSource>>,
    palette_open: Vec<Handle<AudioSource>>,
    palette_select: Vec<Handle<AudioSource>>,
    uno_call: Vec<Handle<AudioSource>>,
    uno_accent: Vec<Handle<AudioSource>>,
    report: Vec<Handle<AudioSource>>,
    challenge: Vec<Handle<AudioSource>>,
    success: Vec<Handle<AudioSource>>,
    failure: Vec<Handle<AudioSource>>,
}

impl UnoSoundAssets {
    pub(crate) fn load(asset_server: &AssetServer) -> Self {
        let casino = "vendor/kenney/casino-audio/Audio";
        let interface = "vendor/kenney/interface-sounds/Audio";
        Self {
            select: numbered_sounds(asset_server, interface, "select_00", 8),
            card_throw: numbered_sounds(asset_server, casino, "card-shove-", 4),
            card_land: numbered_sounds(asset_server, casino, "card-place-", 4),
            card_draw: numbered_sounds(asset_server, casino, "card-slide-", 8),
            penalty: vec![asset_server.load(format!("{interface}/bong_001.ogg"))],
            skip: numbered_sounds(asset_server, interface, "close_00", 4),
            reverse: numbered_sounds(asset_server, interface, "scroll_00", 5),
            palette_open: numbered_sounds(asset_server, interface, "glass_00", 6),
            palette_select: numbered_sounds(asset_server, interface, "pluck_00", 2),
            uno_call: numbered_sounds(asset_server, interface, "confirmation_00", 4),
            uno_accent: numbered_sounds(asset_server, interface, "maximize_00", 9),
            report: numbered_sounds(asset_server, interface, "error_00", 8),
            challenge: numbered_sounds(asset_server, interface, "question_00", 4),
            success: numbered_sounds(asset_server, interface, "confirmation_00", 4),
            failure: numbered_sounds(asset_server, interface, "error_00", 8),
        }
    }

    pub(super) fn variants(&self, kind: UnoSoundKind) -> &[Handle<AudioSource>] {
        match kind {
            UnoSoundKind::CardThrow => &self.card_throw,
            UnoSoundKind::CardLand => &self.card_land,
            UnoSoundKind::CardDraw => &self.card_draw,
            UnoSoundKind::Penalty => &self.penalty,
            UnoSoundKind::Skip => &self.skip,
            UnoSoundKind::Reverse => &self.reverse,
            UnoSoundKind::PaletteOpen => &self.palette_open,
            UnoSoundKind::PaletteSelect => &self.palette_select,
            UnoSoundKind::UnoCall => &self.uno_call,
            UnoSoundKind::UnoAccent => &self.uno_accent,
            UnoSoundKind::Report => &self.report,
            UnoSoundKind::Challenge => &self.challenge,
            UnoSoundKind::Success => &self.success,
            UnoSoundKind::Failure => &self.failure,
        }
    }
}

fn numbered_sounds(
    asset_server: &AssetServer,
    directory: &str,
    prefix: &str,
    count: usize,
) -> Vec<Handle<AudioSource>> {
    (1..=count)
        .map(|index| asset_server.load(format!("{directory}/{prefix}{index}.ogg")))
        .collect()
}
