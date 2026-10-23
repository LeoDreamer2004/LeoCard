use bevy::prelude::*;
use leocard_protocol::{MatchId, PlayerId};
use leocard_shengji::Category;
use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct MissingSuitsUi {
    pub active: bool,
    pub hand: Option<(MatchId, u32)>,
    pub ages: HashMap<(PlayerId, Category), f32>,
}

#[derive(Component)]
pub(in super::super) struct MissingSuitMarker {
    pub player: PlayerId,
    pub door: Category,
}
