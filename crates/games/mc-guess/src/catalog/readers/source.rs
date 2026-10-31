use super::super::{McGuessAttribute, McGuessEntry, McGuessEntryKind};
use csv::{Reader, StringRecord};

pub(super) struct Record<'a> {
    headers: &'a StringRecord,
    values: &'a StringRecord,
}

impl Record<'_> {
    pub fn field(&self, label: &str) -> &str {
        self.headers
            .iter()
            .zip(self.values.iter())
            .find_map(|(header, value)| (header == label).then_some(value))
            .expect("bundled CSV has the requested field")
    }

    fn attributes(&self) -> impl Iterator<Item = (&str, &str)> {
        self.headers.iter().zip(self.values.iter()).skip(2)
    }
}

pub(super) trait CatalogueReader {
    fn attribute(
        &self,
        label: &str,
        value: &str,
        _record: &Record<'_>,
    ) -> Option<McGuessAttribute> {
        Some(McGuessAttribute {
            label: label.to_owned(),
            values: value.split(['、', '；']).map(str::to_owned).collect(),
        })
    }

    fn read(&self, kind: McGuessEntryKind, data: &str) -> Vec<McGuessEntry> {
        let mut reader = Reader::from_reader(data.as_bytes());
        let headers = reader
            .headers()
            .expect("bundled catalogue has a header")
            .clone();
        reader
            .records()
            .map(|values| {
                let values = values.expect("bundled catalogue is valid CSV");
                let record = Record {
                    headers: &headers,
                    values: &values,
                };
                McGuessEntry {
                    name: values[0].to_owned(),
                    id: values[1].to_owned(),
                    kind,
                    attributes: record
                        .attributes()
                        .filter_map(|(label, value)| self.attribute(label, value, &record))
                        .collect(),
                }
            })
            .collect()
    }
}

pub(super) struct DefaultReader;

impl CatalogueReader for DefaultReader {}
