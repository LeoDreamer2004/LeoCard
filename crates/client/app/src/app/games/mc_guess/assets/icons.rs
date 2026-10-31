use bevy::image::{ImageLoaderSettings, ImageSampler};
use bevy::picking::Pickable;
use bevy::prelude::*;
use leocard_mc_guess::{McGuessEntry, McGuessEntryKind};
use leocard_ui::UiComposer;
use std::collections::HashMap;

const LAYOUTS: &str =
    include_str!("../../../../../../../../assets/ui/minecraft_guess/icons/layouts.tsv");
const INDEXES: [(&str, &str); 7] = [
    (
        "entries",
        include_str!("../../../../../../../../assets/ui/minecraft_guess/icons/entries.tsv"),
    ),
    (
        "mobs",
        include_str!("../../../../../../../../assets/ui/minecraft_guess/icons/mobs.tsv"),
    ),
    (
        "enchantments",
        include_str!("../../../../../../../../assets/ui/minecraft_guess/icons/enchantments.tsv"),
    ),
    (
        "effects",
        include_str!("../../../../../../../../assets/ui/minecraft_guess/icons/effects.tsv"),
    ),
    (
        "villager-professions",
        include_str!(
            "../../../../../../../../assets/ui/minecraft_guess/icons/villager-professions.tsv"
        ),
    ),
    (
        "landscapes",
        include_str!("../../../../../../../../assets/ui/minecraft_guess/icons/landscapes.tsv"),
    ),
    (
        "biomes",
        include_str!("../../../../../../../../assets/ui/minecraft_guess/icons/biomes.tsv"),
    ),
];

pub(in super::super) struct IconSprite {
    pub image: Handle<Image>,
    /// Atlas rectangle in source pixels, independent of the image loading state.
    pub region: Vec4,
}

pub(in super::super) struct EntryIcons {
    atlases: [EntryIconAtlas; 7],
}

struct EntryIconAtlas {
    image: Handle<Image>,
    layout: Handle<TextureAtlasLayout>,
    cell_size: u32,
    entries: HashMap<McGuessEntryKind, HashMap<String, usize>>,
}

impl EntryIconAtlas {
    fn load(
        server: &AssetServer,
        name: &str,
        index: &str,
        cell_size: u32,
        columns: u32,
        layouts: &mut Assets<TextureAtlasLayout>,
    ) -> Self {
        let mut entries: HashMap<McGuessEntryKind, HashMap<String, usize>> = HashMap::new();
        let mut count = 0;
        for line in index.lines().skip(1) {
            let mut fields = line.split('\t');
            let kind = match fields.next().expect("icon kind") {
                "block" => McGuessEntryKind::Block,
                "item" => McGuessEntryKind::Item,
                "mob" => McGuessEntryKind::Mob,
                "enchantment" => McGuessEntryKind::Enchantment,
                "effect" => McGuessEntryKind::Effect,
                "villager_profession" => McGuessEntryKind::VillagerProfession,
                "landscape" => McGuessEntryKind::Landscape,
                "biome" => McGuessEntryKind::Biome,
                _ => panic!("invalid bundled icon kind"),
            };
            let id = fields.next().expect("icon identifier");
            let index = fields
                .next()
                .expect("icon index")
                .parse::<usize>()
                .expect("valid bundled icon index");
            entries
                .entry(kind)
                .or_default()
                .insert(id.to_owned(), index);
            count = count.max(index as u32 + 1);
        }
        let layout = layouts.add(TextureAtlasLayout::from_grid(
            UVec2::splat(cell_size),
            columns,
            count.div_ceil(columns),
            None,
            None,
        ));
        let image = server
            .load_builder()
            .with_settings(|settings: &mut ImageLoaderSettings| {
                settings.sampler = ImageSampler::nearest();
            })
            .load(format!("ui/minecraft_guess/icons/{name}.png"));
        Self {
            image,
            layout,
            cell_size,
            entries,
        }
    }

    fn image(&self, entry: &McGuessEntry) -> Option<ImageNode> {
        let index = *self.entries.get(&entry.kind)?.get(&entry.id)?;
        let mut image = ImageNode::from_atlas_image(
            self.image.clone(),
            TextureAtlas {
                layout: self.layout.clone(),
                index,
            },
        );
        // Sample inside texel centers so fractional UI scaling cannot reach adjacent cells.
        image.rect = Some(Rect::from_corners(
            Vec2::splat(0.5),
            Vec2::splat(self.cell_size as f32 - 0.5),
        ));
        Some(image)
    }
}

impl EntryIcons {
    pub fn new(server: &AssetServer, layouts: &mut Assets<TextureAtlasLayout>) -> Self {
        let mut definitions: HashMap<_, _> = LAYOUTS
            .lines()
            .skip(1)
            .map(|line| {
                let mut fields = line.split('\t');
                let name = fields.next().expect("atlas name");
                let size = fields
                    .next()
                    .expect("cell size")
                    .parse()
                    .expect("valid cell size");
                let columns = fields
                    .next()
                    .expect("columns")
                    .parse()
                    .expect("valid columns");
                (name, (size, columns))
            })
            .collect();
        let atlases = INDEXES.map(|(name, index)| {
            let (size, columns) = definitions.remove(name).expect("bundled atlas layout");
            EntryIconAtlas::load(server, name, index, size, columns, layouts)
        });
        assert!(definitions.is_empty(), "every atlas has a bundled index");
        Self { atlases }
    }

    pub fn image(&self, entry: &McGuessEntry) -> ImageNode {
        self.atlases
            .iter()
            .find_map(|atlas| atlas.image(entry))
            .expect("catalogue entry has an icon")
    }

    pub fn sprite(&self, entry: &McGuessEntry, layouts: &Assets<TextureAtlasLayout>) -> IconSprite {
        let image = self.image(entry);
        let atlas = image.texture_atlas.expect("entry icon uses an atlas");
        let rect = layouts
            .get(&atlas.layout)
            .expect("bundled icon layout")
            .textures[atlas.index];
        IconSprite {
            image: image.image,
            region: rect
                .min
                .as_vec2()
                .extend(rect.width() as f32)
                .extend(rect.height() as f32),
        }
    }

    pub fn spawn(commands: &mut UiComposer, parent: Entity, image: ImageNode, size: f32) {
        commands.child(
            parent,
            (
                image,
                Node {
                    width: px(size),
                    height: px(size),
                    flex_shrink: 0.0,
                    ..default()
                },
                Pickable::IGNORE,
            ),
        );
    }
}
