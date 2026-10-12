use bevy::picking::Pickable;
use bevy::prelude::*;
use leocard_minecraft_guess::{Entry, EntryKind};
use std::collections::HashMap;

const INDEX: &str = include_str!("../../../../../../assets/data/minecraft/icons.tsv");
const CELL_SIZE: u32 = 64;
const COLUMNS: u32 = 64;

pub(super) struct EntryIcons {
    image: Handle<Image>,
    layout: Handle<TextureAtlasLayout>,
    blocks: HashMap<String, usize>,
    items: HashMap<String, usize>,
}

impl EntryIcons {
    pub fn new(image: Handle<Image>, layouts: &mut Assets<TextureAtlasLayout>) -> Self {
        let mut blocks = HashMap::new();
        let mut items = HashMap::new();
        let mut count = 0;
        for line in INDEX.lines().skip(1) {
            let mut fields = line.split('\t');
            let kind = fields.next().expect("icon kind");
            let id = fields.next().expect("icon identifier");
            let index = fields
                .next()
                .expect("icon index")
                .parse::<usize>()
                .expect("valid bundled icon index");
            let entries = match kind {
                "block" => &mut blocks,
                "item" => &mut items,
                _ => panic!("invalid bundled icon kind"),
            };
            entries.insert(id.to_owned(), index);
            count = count.max(index as u32 + 1);
        }
        let layout = layouts.add(TextureAtlasLayout::from_grid(
            UVec2::splat(CELL_SIZE),
            COLUMNS,
            count.div_ceil(COLUMNS),
            None,
            None,
        ));
        Self {
            image,
            layout,
            blocks,
            items,
        }
    }

    pub fn image(&self, entry: &Entry) -> ImageNode {
        let entries = match entry.kind {
            EntryKind::Block => &self.blocks,
            EntryKind::Item => &self.items,
        };
        let index = *entries.get(&entry.id).expect("catalogue entry has an icon");
        ImageNode::from_atlas_image(
            self.image.clone(),
            TextureAtlas {
                layout: self.layout.clone(),
                index,
            },
        )
    }

    pub fn spawn(commands: &mut Commands, parent: Entity, image: ImageNode, size: f32) {
        commands.spawn((
            image,
            Node {
                width: px(size),
                height: px(size),
                flex_shrink: 0.0,
                ..default()
            },
            Pickable::IGNORE,
            ChildOf(parent),
        ));
    }
}
