use super::super::data::GraphData;
use std::collections::HashMap;

pub(super) struct DiversityIndex {
    features: Vec<NodeFeatures>,
}

struct NodeFeatures {
    version: Option<usize>,
    families: Vec<usize>,
}

/// Whole-puzzle counts, including nodes nowhere near the current expansion path.
#[derive(Default)]
pub(super) struct Diversity {
    versions: HashMap<usize, usize>,
    families: HashMap<usize, usize>,
    version_total: usize,
    version_mass: f32,
    family_pairs: usize,
}

impl DiversityIndex {
    pub fn new(data: &GraphData) -> Self {
        let mut versions = HashMap::new();
        let mut families = HashMap::new();
        let features = data
            .nodes
            .iter()
            .map(|node| {
                let version = node
                    .entry
                    .and_then(|entry| {
                        entry
                            .attributes
                            .iter()
                            .find(|attribute| attribute.label == "首次加入版本")
                            .and_then(|attribute| attribute.values.first())
                    })
                    .map(|version| {
                        let next = versions.len();
                        *versions.entry(version.as_str()).or_insert(next)
                    });
                let mut words: Vec<_> = node
                    .entry
                    .into_iter()
                    .flat_map(|entry| {
                        entry
                            .id
                            .split_once(':')
                            .expect("namespaced entry")
                            .1
                            .split(|c: char| !c.is_ascii_alphanumeric())
                            .filter(|word| !word.is_empty() && !variant_modifier(word))
                    })
                    .collect();
                words.sort_unstable();
                words.dedup();
                let families = words
                    .into_iter()
                    .map(|word| {
                        let next = families.len();
                        *families.entry(word).or_insert(next)
                    })
                    .collect();
                NodeFeatures { version, families }
            })
            .collect();
        Self { features }
    }

    pub fn census(&self, nodes: impl IntoIterator<Item = usize>) -> Diversity {
        let mut census = Diversity::default();
        for node in nodes {
            let feature = &self.features[node];
            if let Some(version) = feature.version {
                let count = census.versions.entry(version).or_default();
                census.version_mass += mass(*count + 1) - mass(*count);
                *count += 1;
                census.version_total += 1;
            }
            for &family in &feature.families {
                let count = census.families.entry(family).or_default();
                census.family_pairs += *count;
                *count += 1;
            }
        }
        census
    }
}

impl Diversity {
    pub fn score(&self) -> f32 {
        self.versions.len() as f32 * 2.0 + entropy(self.version_total, self.version_mass) * 8.0
            - self.family_pairs as f32 * 1.5
    }

    pub fn gain(&self, index: &DiversityIndex, nodes: impl IntoIterator<Item = usize>) -> f32 {
        let mut versions = HashMap::<usize, usize>::new();
        let mut families = HashMap::<usize, usize>::new();
        let mut total = self.version_total;
        let mut version_mass = self.version_mass;
        let mut new_versions = 0;
        let mut pairs = 0;
        let mut added = 0;
        for node in nodes {
            let feature = &index.features[node];
            if let Some(version) = feature.version {
                let increment = versions.entry(version).or_default();
                let count = self.versions.get(&version).copied().unwrap_or(0) + *increment;
                new_versions += usize::from(count == 0);
                version_mass += mass(count + 1) - mass(count);
                *increment += 1;
                total += 1;
                added += 1;
            }
            for &family in &feature.families {
                let increment = families.entry(family).or_default();
                pairs += self.families.get(&family).copied().unwrap_or(0) + *increment;
                *increment += 1;
            }
        }
        // Normalize path length; long paths do not win just by contributing more nodes.
        (new_versions as f32 * 2.0
            + (entropy(total, version_mass) - entropy(self.version_total, self.version_mass)) * 8.0
            - pairs as f32 * 1.5)
            / added.max(1) as f32
    }
}

fn mass(count: usize) -> f32 {
    if count == 0 {
        0.0
    } else {
        count as f32 * (count as f32).ln()
    }
}

fn entropy(total: usize, mass: f32) -> f32 {
    if total == 0 {
        0.0
    } else {
        (total as f32).ln() - mass / total as f32
    }
}

fn variant_modifier(word: &str) -> bool {
    // Retain material and shape words, so both dyed wool and a material's cut forms share counts.
    matches!(
        word,
        "white"
            | "orange"
            | "magenta"
            | "light"
            | "blue"
            | "yellow"
            | "lime"
            | "pink"
            | "gray"
            | "cyan"
            | "purple"
            | "brown"
            | "green"
            | "red"
            | "black"
            | "block"
            | "of"
            | "the"
            | "waxed"
            | "weathered"
            | "exposed"
            | "oxidized"
            | "polished"
            | "chiseled"
            | "cracked"
            | "smooth"
            | "cut"
            | "stripped"
    )
}
