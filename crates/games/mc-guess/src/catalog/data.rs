use super::readers::read_bundled;
use super::{McGuessEntry, McGuessEntryKind};
use crate::McGuessPool;
use std::sync::OnceLock;

#[derive(Debug)]
pub struct McGuessCatalogue {
    entries: Vec<McGuessEntry>,
}

impl McGuessCatalogue {
    pub fn bundled() -> &'static Self {
        static CATALOGUE: OnceLock<McGuessCatalogue> = OnceLock::new();
        CATALOGUE.get_or_init(|| Self {
            entries: read_bundled(),
        })
    }

    pub fn entries(&self, pool: McGuessPool) -> impl Iterator<Item = &McGuessEntry> {
        self.entries
            .iter()
            .filter(move |entry| pool.contains(entry.kind))
    }

    pub fn find(&self, kind: McGuessEntryKind, name_or_id: &str) -> Option<&McGuessEntry> {
        let name_or_id = name_or_id.trim();
        self.entries.iter().find(|entry| {
            entry.kind == kind && (entry.name == name_or_id || entry.id == name_or_id)
        })
    }

    pub fn search<'a>(
        &'a self,
        kind: McGuessEntryKind,
        query: &'a str,
    ) -> impl Iterator<Item = &'a McGuessEntry> {
        self.entries.iter().filter(move |entry| {
            entry.kind == kind
                && (entry.name.contains(query.trim()) || entry.id.contains(query.trim()))
        })
    }
}
