use leocard_minecraft_guess::EntryKind;

#[derive(Clone, Copy)]
pub(super) enum NodeShape {
    Tag,
    Symbol,
}

impl NodeShape {
    pub fn for_kind(kind: Option<EntryKind>) -> Self {
        if kind.is_none() {
            Self::Tag
        } else {
            Self::Symbol
        }
    }

    /// Leave room around the symbol's transparent source-image padding.
    pub fn radius(self) -> f32 {
        match self {
            Self::Tag => 20.0,
            Self::Symbol => 28.0,
        }
    }
}
