use leocard_protocol::GameKind;

#[derive(Clone, Copy)]
pub(crate) struct GameDescriptor {
    pub kind: GameKind,
    pub title: &'static str,
    pub description: &'static str,
}

/// Game-owned identity used by the gallery, LAN list and shop.
pub(crate) trait GameMetadata {
    const DESCRIPTOR: GameDescriptor;
}
