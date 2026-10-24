use super::super::MinecraftGuessAssets;
use super::material::{GraphCurveMaterial, GraphNodeMaterial};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use leocard_minecraft_guess::{EntryKind, GraphRelation, GraphView};
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
    pub assets: Res<'w, MinecraftGuessAssets>,
    pub layouts: Res<'w, Assets<TextureAtlasLayout>>,
}

impl GraphArt {
    pub(super) fn begin(
        &mut self,
        key: (MatchId, u8),
        graph: &GraphView,
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

    pub(super) fn sync_sprites(&mut self, graph: &GraphView, materials: &mut GraphMaterials) {
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

pub(super) fn node_color(kind: Option<EntryKind>) -> Color {
    rgb(match kind {
        Some(EntryKind::Block) => 0x7da858,
        Some(EntryKind::Item) => 0xedbf49,
        Some(EntryKind::Mob) => 0x6b944c,
        Some(EntryKind::Enchantment) => 0xb96eb6,
        Some(EntryKind::Effect) => 0xb77cc6,
        Some(EntryKind::VillagerProfession) => 0x4cd668,
        Some(EntryKind::Landscape) => 0x6ab0a4,
        Some(EntryKind::Biome) => 0x90b155,
        None => 0xa4a8b2,
    })
}

pub(super) fn relation_color(relation: GraphRelation) -> Color {
    rgb(match relation {
        GraphRelation::Crafting => 0x8fd18b,
        GraphRelation::Cooking => 0xed9a5e,
        GraphRelation::Brewing => 0xbd93ed,
        GraphRelation::Smithing => 0x70b9f4,
        GraphRelation::Stonecutting => 0x63d1c8,
        GraphRelation::Loot => 0xf0cb68,
        GraphRelation::MiningDrop => 0xbcb089,
        GraphRelation::Gifts => 0xe8a5c5,
        GraphRelation::GrantsEffect => 0xb8a8ff,
        GraphRelation::Equippable => 0xcf87b5,
        GraphRelation::SpawnWeapon => 0xdf876c,
        GraphRelation::ConvertsTo => 0xb1cae8,
        GraphRelation::Enchantable => 0x5297df,
        GraphRelation::ConflictsWith => 0xef7474,
        GraphRelation::Generates => 0x6ac9a5,
        GraphRelation::SpawnsMob => 0x5cbfd6,
        GraphRelation::LandscapeSpawns => 0x79a2b9,
        GraphRelation::GeneratesLandscape => 0xc49f5f,
        GraphRelation::MajorBlock => 0x96b58e,
        GraphRelation::Locates => 0xf29e9e,
        GraphRelation::LosesLife => 0x9f9f9f,
        GraphRelation::Infests => 0xa3a67e,
        GraphRelation::Potting => 0xb9a17d,
        GraphRelation::Strips => 0xd2aa74,
        GraphRelation::Harvest => 0xc5ce85,
        GraphRelation::Oxidizes => 0x65bbaf,
        GraphRelation::Scrapes => 0xb9d3df,
        GraphRelation::Waxes => 0xebad44,
        GraphRelation::Unwaxes => 0xdfb890,
        GraphRelation::Hardens => 0x9bbab7,
        GraphRelation::Hatches => 0xe4bd64,
        GraphRelation::SpawnsEntity => 0xa9c95d,
        GraphRelation::MemberOf => 0x8498a5,
        GraphRelation::TradeIngredient => 0xd8a88a,
        GraphRelation::TradeOutput => 0xe9e6ba,
        GraphRelation::Workstation => 0x6aaf8d,
        GraphRelation::Processing => 0xd5a273,
        GraphRelation::Natural => 0x87b69c,
        GraphRelation::Trading => 0xc2b089,
        GraphRelation::Other => 0x9b9eb2,
    })
}

fn rgb(value: u32) -> Color {
    Color::srgb_u8((value >> 16) as u8, (value >> 8) as u8, value as u8)
}
