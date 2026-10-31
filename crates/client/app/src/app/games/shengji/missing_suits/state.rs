use bevy::prelude::*;
use leocard_game_common::PlayerId;
use leocard_protocol::MatchId;
use leocard_shengji::ShengjiCategory;
use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct MissingSuitsUi {
    pub active: bool,
    pub hand: Option<(MatchId, u32)>,
    pub ages: HashMap<(PlayerId, ShengjiCategory), f32>,
}

#[derive(Component)]
pub(in super::super) struct MissingSuitMarker {
    pub player: PlayerId,
    pub door: ShengjiCategory,
}
