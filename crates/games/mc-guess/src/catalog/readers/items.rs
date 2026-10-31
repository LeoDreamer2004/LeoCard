use super::super::McGuessAttribute;
use super::{CatalogueReader, DefaultReader, Record};

pub(super) struct ItemReader;

impl CatalogueReader for ItemReader {
    fn attribute(&self, label: &str, value: &str, record: &Record<'_>) -> Option<McGuessAttribute> {
        if label == "主色调" {
            None
        } else {
            DefaultReader.attribute(label, value, record)
        }
    }
}
