use super::super::{BiomeRanges, McGuessAttribute};
use super::{CatalogueReader, DefaultReader, Record};

pub(super) struct LandscapeReader;

impl CatalogueReader for LandscapeReader {
    fn attribute(&self, label: &str, value: &str, record: &Record<'_>) -> Option<McGuessAttribute> {
        if label != "生成生物群系" {
            return DefaultReader.attribute(label, value, record);
        }
        let dimensions: Vec<_> = record.field("生成维度").split(['、', '；']).collect();
        Some(McGuessAttribute {
            label: label.to_owned(),
            values: BiomeRanges::bundled().expand(value, &dimensions),
        })
    }
}
