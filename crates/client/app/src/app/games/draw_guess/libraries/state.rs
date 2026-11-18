use crate::player::LocalWordLibraries;
use bevy::prelude::*;
use leocard_draw_guess::WordLibrary;
use leocard_protocol::{WordLibraryInfo, WordLibraryOffer};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum LibrarySelection {
    Builtin,
    Local(String),
    New,
}

#[derive(Resource)]
pub(in super::super) struct WordLibraryManager {
    pub store: LocalWordLibraries,
    pub open: bool,
    pub progress: f32,
    pub selection: LibrarySelection,
    pub title: String,
    pub content: String,
    pub editor_revision: u64,
    pub status: String,
    pub prompted: Option<u64>,
}

impl Default for WordLibraryManager {
    fn default() -> Self {
        let builtin = WordLibrary::builtin();
        Self {
            store: LocalWordLibraries::default(),
            open: false,
            progress: 0.0,
            selection: LibrarySelection::Builtin,
            title: builtin.title.clone(),
            content: builtin.content(),
            editor_revision: 0,
            status: String::new(),
            prompted: None,
        }
    }
}

impl WordLibraryManager {
    pub fn selected_library(&self) -> Option<WordLibrary> {
        match &self.selection {
            LibrarySelection::Builtin => Some(WordLibrary::builtin()),
            LibrarySelection::Local(id) => self
                .store
                .entries
                .iter()
                .find(|entry| &entry.id == id)
                .map(|entry| entry.library.clone()),
            LibrarySelection::New => None,
        }
    }

    pub fn dirty(&self) -> bool {
        self.selected_library().map_or_else(
            || self.title != "新词库" || !self.content.is_empty(),
            |library| library.title != self.title || library.content() != self.content,
        )
    }

    pub fn word_count(&self) -> usize {
        self.content
            .lines()
            .filter(|line| !line.trim().is_empty())
            .count()
    }

    pub fn select(&mut self, selection: LibrarySelection) {
        self.selection = selection;
        let library = self.selected_library();
        self.title = library
            .as_ref()
            .map_or("新词库", |library| &library.title)
            .to_owned();
        self.content = library.map_or_else(String::new, |library| library.content());
        self.status.clear();
        self.editor_revision = self.editor_revision.wrapping_add(1);
    }
}

#[derive(Resource)]
pub(in super::super) struct WordLibraryRoom {
    pub selected: WordLibraryInfo,
    pub offers: Vec<WordLibraryOffer>,
}

impl Default for WordLibraryRoom {
    fn default() -> Self {
        Self {
            selected: (&WordLibrary::builtin()).into(),
            offers: Vec::new(),
        }
    }
}

#[derive(Clone, Copy, Component)]
pub(in super::super) enum LibraryField {
    Title,
    Content,
}

#[derive(Component)]
pub(in super::super) struct LibraryWordCount;
