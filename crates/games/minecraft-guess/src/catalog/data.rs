use super::{Attribute, Entry, EntryKind};
use crate::GuessPool;
use std::sync::OnceLock;

const BLOCKS: &str = include_str!("../../../../../assets/data/minecraft/blocks.csv");
const ITEMS: &str = include_str!("../../../../../assets/data/minecraft/items.csv");

#[derive(Debug)]
pub struct Catalogue {
    entries: Vec<Entry>,
}

impl Catalogue {
    pub fn bundled() -> &'static Self {
        static CATALOGUE: OnceLock<Catalogue> = OnceLock::new();
        CATALOGUE.get_or_init(|| Self {
            entries: Self::read(BLOCKS, EntryKind::Block)
                .chain(Self::read(ITEMS, EntryKind::Item))
                .collect(),
        })
    }

    fn read(data: &str, kind: EntryKind) -> impl Iterator<Item = Entry> {
        let mut reader = csv::Reader::from_reader(data.as_bytes());
        let headers = reader
            .headers()
            .expect("bundled catalogue has a header")
            .clone();
        reader
            .records()
            .map(move |record| {
                let record = record.expect("bundled catalogue is valid CSV");
                Entry {
                    name: record[0].to_owned(),
                    id: record[1].to_owned(),
                    kind,
                    attributes: headers
                        .iter()
                        .zip(record.iter())
                        .skip(2)
                        .filter(|(label, _)| *label != "主色调")
                        .map(|(label, value)| Attribute {
                            label: label.to_owned(),
                            values: value.split(['、', '；']).map(str::to_owned).collect(),
                        })
                        .collect(),
                }
            })
            .collect::<Vec<_>>()
            .into_iter()
    }

    pub fn entries(&self, pool: GuessPool) -> impl Iterator<Item = &Entry> {
        self.entries.iter().filter(move |entry| match pool {
            GuessPool::Blocks => entry.kind == EntryKind::Block,
            GuessPool::Items => entry.kind == EntryKind::Item,
            GuessPool::Mixed => true,
        })
    }

    pub fn find(&self, kind: EntryKind, name_or_id: &str) -> Option<&Entry> {
        let name_or_id = name_or_id.trim();
        self.entries.iter().find(|entry| {
            entry.kind == kind && (entry.name == name_or_id || entry.id == name_or_id)
        })
    }

    pub fn search<'a>(
        &'a self,
        kind: EntryKind,
        query: &'a str,
    ) -> impl Iterator<Item = &'a Entry> {
        self.entries.iter().filter(move |entry| {
            entry.kind == kind
                && (entry.name.contains(query.trim()) || entry.id.contains(query.trim()))
        })
    }
}
