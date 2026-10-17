use super::readers::read_bundled;
use super::{Entry, EntryKind};
use crate::GuessPool;
use std::sync::OnceLock;

#[derive(Debug)]
pub struct Catalogue {
    entries: Vec<Entry>,
}

impl Catalogue {
    pub fn bundled() -> &'static Self {
        static CATALOGUE: OnceLock<Catalogue> = OnceLock::new();
        CATALOGUE.get_or_init(|| Self {
            entries: read_bundled(),
        })
    }

    pub fn entries(&self, pool: GuessPool) -> impl Iterator<Item = &Entry> {
        self.entries
            .iter()
            .filter(move |entry| pool.contains(entry.kind))
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
