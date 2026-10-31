use super::super::McGuessAssets;
use super::material::{GraphCurveMaterial, GraphNodeMaterial};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use leocard_mc_guess::{McGuessEntryKind, McGuessGraphRelation, McGuessGraphView};
use leocard_protocol::MatchId;

#[derive(Resource, Default)]
pub(crate) struct GraphArt {
    pub(super) key: Option<(MatchId, u8)>,
    pub(super) curves: Vec<Handle<GraphCurveMaterial>>,
    pub(super) nodes: Vec<Handle<GraphNodeMaterial>>,
    pub(super) revision: u64,
    revealed: Vec<bool>,
}

#[derive(SystemParam)]
pub(in super::super) struct GraphMaterials<'w> {
    pub curves: ResMut<'w, Assets<GraphCurveMaterial>>,
    pub nodes: ResMut<'w, Assets<GraphNodeMaterial>>,
    pub assets: Res<'w, McGuessAssets>,
    pub layouts: Res<'w, Assets<TextureAtlasLayout>>,
}

impl GraphArt {
    pub(super) fn begin(
        &mut self,
        key: (MatchId, u8),
        graph: &McGuessGraphView,
        materials: &mut GraphMaterials,
    ) {
        self.key = Some(key);
        self.revision = 0;
        self.curves = graph
            .edges
            .iter()
            .map(|_| {
                materials.curves.add(GraphCurveMaterial {
                    start_control: Vec4::ZERO,
                    end_size: Vec4::ONE,
                    color: Color::WHITE.to_linear(),
                    appearance: Vec4::ZERO,
                })
            })
            .collect();
        self.nodes = graph
            .nodes
            .iter()
            .map(|node| {
                let sprite = materials.assets.node_sprite(node, &materials.layouts);
                materials.nodes.add(GraphNodeMaterial {
                    color: Color::WHITE.to_linear(),
                    state: Vec4::ZERO,
                    sprite: sprite.region,
                    activity: Vec4::ZERO,
                    icon: sprite.image,
                })
            })
            .collect();
        self.revealed = graph.nodes.iter().map(|node| node.revealed()).collect();
    }

    pub(super) fn sync_sprites(
        &mut self,
        graph: &McGuessGraphView,
        materials: &mut GraphMaterials,
    ) {
        for ((node, revealed), handle) in
            graph.nodes.iter().zip(&mut self.revealed).zip(&self.nodes)
        {
            if *revealed == node.revealed() {
                continue;
            }
            let sprite = materials.assets.node_sprite(node, &materials.layouts);
            if let Some(mut material) = materials.nodes.get_mut(handle) {
                material.sprite = sprite.region;
                material.icon = sprite.image;
            }
            *revealed = node.revealed();
            self.revision += 1;
        }
    }
}

pub(super) fn node_color(kind: Option<McGuessEntryKind>) -> Color {
    rgb(match kind {
        Some(McGuessEntryKind::Block) => 0x7da858,
        Some(McGuessEntryKind::Item) => 0xedbf49,
        Some(McGuessEntryKind::Mob) => 0x6b944c,
        Some(McGuessEntryKind::Enchantment) => 0xb96eb6,
        Some(McGuessEntryKind::Effect) => 0xb77cc6,
        Some(McGuessEntryKind::VillagerProfession) => 0x4cd668,
        Some(McGuessEntryKind::Landscape) => 0x6ab0a4,
        Some(McGuessEntryKind::Biome) => 0x90b155,
        None => 0xa4a8b2,
    })
}

pub(super) fn relation_color(relation: McGuessGraphRelation) -> Color {
    rgb(match relation {
        McGuessGraphRelation::Crafting => 0x8fd18b,
        McGuessGraphRelation::Cooking => 0xed9a5e,
        McGuessGraphRelation::Brewing => 0xbd93ed,
        McGuessGraphRelation::Smithing => 0x70b9f4,
        McGuessGraphRelation::Stonecutting => 0x63d1c8,
        McGuessGraphRelation::Loot => 0xf0cb68,
        McGuessGraphRelation::MiningDrop => 0xbcb089,
        McGuessGraphRelation::Gifts => 0xe8a5c5,
        McGuessGraphRelation::GrantsEffect => 0xb8a8ff,
        McGuessGraphRelation::Equippable => 0xcf87b5,
        McGuessGraphRelation::SpawnWeapon => 0xdf876c,
        McGuessGraphRelation::ConvertsTo => 0xb1cae8,
        McGuessGraphRelation::Enchantable => 0x5297df,
        McGuessGraphRelation::ConflictsWith => 0xef7474,
        McGuessGraphRelation::Generates => 0x6ac9a5,
        McGuessGraphRelation::SpawnsMob => 0x5cbfd6,
        McGuessGraphRelation::LandscapeSpawns => 0x79a2b9,
        McGuessGraphRelation::GeneratesLandscape => 0xc49f5f,
        McGuessGraphRelation::MajorBlock => 0x96b58e,
        McGuessGraphRelation::Locates => 0xf29e9e,
        McGuessGraphRelation::LosesLife => 0x9f9f9f,
        McGuessGraphRelation::Infests => 0xa3a67e,
        McGuessGraphRelation::Potting => 0xb9a17d,
        McGuessGraphRelation::Strips => 0xd2aa74,
        McGuessGraphRelation::Harvest => 0xc5ce85,
        McGuessGraphRelation::Oxidizes => 0x65bbaf,
        McGuessGraphRelation::Scrapes => 0xb9d3df,
        McGuessGraphRelation::Waxes => 0xebad44,
        McGuessGraphRelation::Unwaxes => 0xdfb890,
        McGuessGraphRelation::Hardens => 0x9bbab7,
        McGuessGraphRelation::Hatches => 0xe4bd64,
        McGuessGraphRelation::SpawnsEntity => 0xa9c95d,
        McGuessGraphRelation::MemberOf => 0x8498a5,
        McGuessGraphRelation::TradeIngredient => 0xd8a88a,
        McGuessGraphRelation::TradeOutput => 0xe9e6ba,
        McGuessGraphRelation::Workstation => 0x6aaf8d,
        McGuessGraphRelation::Processing => 0xd5a273,
        McGuessGraphRelation::Natural => 0x87b69c,
        McGuessGraphRelation::Trading => 0xc2b089,
        McGuessGraphRelation::Other => 0x9b9eb2,
    })
}

fn rgb(value: u32) -> Color {
    Color::srgb_u8((value >> 16) as u8, (value >> 8) as u8, value as u8)
}
