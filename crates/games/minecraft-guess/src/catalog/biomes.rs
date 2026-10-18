use std::collections::{BTreeMap, BTreeSet};
use std::sync::OnceLock;

const ANY: &str = "任意群系";
const WITH_VOID: &str = "（含虚空）";

struct Biome {
    dimension: String,
    natural: bool,
}

pub(super) struct BiomeRanges {
    biomes: BTreeMap<String, Biome>,
}

impl BiomeRanges {
    pub fn bundled() -> &'static Self {
        static RANGES: OnceLock<BiomeRanges> = OnceLock::new();
        RANGES.get_or_init(|| {
            let data = include_str!("../../../../../assets/data/minecraft/biomes.csv");
            let mut reader = csv::Reader::from_reader(data.as_bytes());
            let biomes = reader
                .records()
                .map(|row| {
                    let row = row.expect("bundled biome is valid CSV");
                    (
                        row[0].to_owned(),
                        Biome {
                            dimension: row[2].to_owned(),
                            natural: &row[1] != "minecraft:the_void",
                        },
                    )
                })
                .collect();
            Self { biomes }
        })
    }

    pub fn expand(&self, text: &str, dimensions: &[&str]) -> Vec<String> {
        let (rule, include_void) = text
            .strip_suffix(WITH_VOID)
            .map_or((text, false), |rule| (rule, true));
        let (any, excluded) = if rule == ANY {
            (true, BTreeSet::new())
        } else if let Some(excluded) = rule
            .strip_prefix('除')
            .and_then(|rule| rule.strip_suffix("外的任意群系"))
        {
            (true, excluded.split('、').collect())
        } else {
            (false, BTreeSet::new())
        };
        let values: BTreeSet<_> = if any {
            assert!(excluded.iter().all(|name| self.biomes.contains_key(*name)));
            self.biomes
                .iter()
                .filter(|(name, biome)| {
                    dimensions.contains(&biome.dimension.as_str())
                        && (biome.natural || include_void)
                        && !excluded.contains(name.as_str())
                })
                .map(|(name, _)| name.clone())
                .collect()
        } else {
            text.split(['、', '；']).map(str::to_owned).collect()
        };
        assert!(
            !values.is_empty() && values.iter().all(|name| self.biomes.contains_key(name)),
            "bundled landscape has valid biome names"
        );
        values.into_iter().collect()
    }

    pub fn describe(&self, values: &[String]) -> Vec<String> {
        let dimensions: BTreeSet<_> = values
            .iter()
            .map(|name| self.biomes[name].dimension.as_str())
            .collect();
        let actual: BTreeSet<_> = values.iter().map(String::as_str).collect();
        let missing: Vec<_> = self
            .biomes
            .iter()
            .filter(|(name, biome)| {
                biome.natural
                    && dimensions.contains(biome.dimension.as_str())
                    && !actual.contains(name.as_str())
            })
            .map(|(name, _)| name.as_str())
            .collect();
        if missing.len() > 3 || missing.len() >= actual.len() {
            return values.to_vec();
        }
        let mut rule = if missing.is_empty() {
            ANY.to_owned()
        } else {
            format!("除{}外的任意群系", missing.join("、"))
        };
        if actual.contains("虚空") {
            rule.push_str(WITH_VOID);
        }
        vec![rule]
    }

    pub fn describe_by_dimension(&self, values: &[String]) -> Vec<String> {
        let mut groups: BTreeMap<&str, Vec<String>> = BTreeMap::new();
        for name in values {
            let Some(biome) = self.biomes.get(name) else {
                return values.to_vec();
            };
            groups
                .entry(biome.dimension.as_str())
                .or_default()
                .push(name.clone());
        }
        groups
            .into_iter()
            .map(|(dimension, biomes)| {
                format!("{dimension}：{}", self.describe(&biomes).join("、"))
            })
            .collect()
    }
}
