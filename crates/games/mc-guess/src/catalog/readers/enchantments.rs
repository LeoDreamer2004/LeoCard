use super::super::{EnchantmentItems, McGuessAttribute};
use super::{CatalogueReader, DefaultReader, Record};

pub(super) struct EnchantmentReader;

impl CatalogueReader for EnchantmentReader {
    fn attribute(&self, label: &str, value: &str, record: &Record<'_>) -> Option<McGuessAttribute> {
        if label != "适用物品" {
            return DefaultReader.attribute(label, value, record);
        }
        Some(McGuessAttribute {
            label: label.to_owned(),
            values: EnchantmentItems::bundled().expand(record.field("附魔标识")),
        })
    }
}
