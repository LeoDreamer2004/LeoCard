use crate::analysis::{GuessAnalysisInput, RoundAnalysis};
use crate::{
    EntryKind, GuessActionStatistics, GuessHandStatistics, GuessMode, GuessPlayStatistics,
    GuessResult,
};
use std::mem;

#[derive(Clone, Debug)]
pub(super) struct GraphAnalysis {
    nodes: Vec<NodeAnalysis>,
    players: Vec<u8>,
    unassigned_skips: Vec<u32>,
    actions: Vec<GuessActionStatistics>,
    hands: Vec<GuessHandStatistics>,
}

#[derive(Clone, Debug)]
struct NodeAnalysis {
    round: RoundAnalysis,
    turns: usize,
    guesses: usize,
    skipped: Vec<u8>,
    winner: Option<u8>,
}

impl GraphAnalysis {
    pub fn new(players: &[u8], nodes: usize) -> Self {
        Self {
            nodes: (0..nodes)
                .map(|_| NodeAnalysis {
                    round: RoundAnalysis::new(players),
                    turns: 0,
                    guesses: 0,
                    skipped: Vec::new(),
                    winner: None,
                })
                .collect(),
            players: players.to_vec(),
            unassigned_skips: vec![0; players.len()],
            actions: Vec::new(),
            hands: Vec::new(),
        }
    }

    pub fn guess(&mut self, index: usize, player: u8, result: &GuessResult, previous: Option<u32>) {
        let node = &mut self.nodes[index];
        node.round.guess(GuessAnalysisInput {
            player,
            mode: GuessMode::Graph,
            result,
            previous,
            turns: node.turns,
            guess_count: node.guesses,
            skipped: &node.skipped,
        });
        self.actions.extend(node.round.take_actions());
        node.turns += 1;
        node.guesses += 1;
        node.skipped.clear();
        if result.is_correct() {
            node.winner = Some(player);
            self.hands
                .extend(
                    node.round
                        .consistency()
                        .map(|(player, consistent)| GuessHandStatistics {
                            player,
                            consistent,
                            round_sweep: false,
                            match_sweep: false,
                            no_answers_top: false,
                        }),
                );
        }
    }

    pub fn skip(&mut self, index: Option<usize>, player: u8) {
        let Some(index) = index else {
            let slot = self
                .players
                .iter()
                .position(|id| *id == player)
                .expect("graph player");
            self.unassigned_skips[slot] += 1;
            return;
        };
        let node = &mut self.nodes[index];
        node.round.skip(player);
        self.actions.extend(node.round.take_actions());
        node.turns += 1;
        node.skipped.push(player);
        if node.skipped.len() == self.players.len() {
            node.skipped.clear();
        }
    }

    pub fn take_actions(&mut self) -> Vec<GuessActionStatistics> {
        mem::take(&mut self.actions)
    }

    pub fn take_hands(&mut self) -> Vec<GuessHandStatistics> {
        mem::take(&mut self.hands)
    }

    pub fn statistics(
        &self,
        entries: impl Iterator<Item = (usize, EntryKind)>,
        allow_unknown: bool,
    ) -> Vec<(u8, GuessPlayStatistics)> {
        let mut totals: Vec<_> = self
            .players
            .iter()
            .copied()
            .zip(&self.unassigned_skips)
            .map(|(player, &skips)| {
                (
                    player,
                    GuessPlayStatistics {
                        skips,
                        ..Default::default()
                    },
                )
            })
            .collect();
        for (index, kind) in entries {
            let node = &self.nodes[index];
            for ((_, total), (player, mut statistics)) in
                totals.iter_mut().zip(node.round.statistics())
            {
                statistics.finish_game(kind, node.winner == Some(player), allow_unknown);
                total.merge(statistics);
            }
        }
        totals
    }
}
