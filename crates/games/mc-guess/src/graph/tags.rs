use super::data::GraphData;
use super::{McGuessGraphNodeView, McGuessGraphRelation};
use crate::McGuessEntry;

impl McGuessGraphNodeView {
    /// All catalogue members of an unlocked tag, including entries outside this puzzle.
    pub fn tag_members(&self) -> Vec<&'static McGuessEntry> {
        let Some(name) = &self.tag_name else {
            return Vec::new();
        };
        let data = GraphData::bundled();
        let Some(index) = data
            .nodes
            .iter()
            .position(|node| node.entry.is_none() && node.name == *name)
        else {
            return Vec::new();
        };
        let mut members: Vec<_> = data
            .edges
            .iter()
            .filter(|edge| edge.kind == McGuessGraphRelation::MemberOf && edge.to == index)
            .filter_map(|edge| data.nodes[edge.from].entry)
            .collect();
        members.sort_unstable_by(|a, b| {
            (a.kind as u8, &a.name, &a.id).cmp(&(b.kind as u8, &b.name, &b.id))
        });
        members.dedup_by(|a, b| a.kind == b.kind && a.id == b.id);
        members
    }
}
