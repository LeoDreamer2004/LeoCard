use crate::analysis::{GuessAnalysisInput, RoundAnalysis};
use crate::{
    McGuessActionStatistics, McGuessEntryKind, McGuessGraphActionStatistics, McGuessHandStatistics,
    McGuessMode, McGuessPlayStatistics, McGuessResult,
};
use leocard_game_common::PlayerId;
use std::mem;

#[derive(Clone, Debug)]
pub(super) struct GraphAnalysis {
    nodes: Vec<NodeAnalysis>,
    players: Vec<PlayerId>,
    unassigned_skips: Vec<u32>,
    actions: Vec<McGuessActionStatistics>,
    hands: Vec<McGuessHandStatistics>,
    scores: Vec<u32>,
    turns: usize,
}

#[derive(Clone, Debug)]
struct NodeAnalysis {
    round: RoundAnalysis,
    turns: usize,
    guesses: usize,
    skipped: Vec<PlayerId>,
    winner: Option<PlayerId>,
    first_green: Option<PlayerId>,
    assisted: bool,
}

impl GraphAnalysis {
    pub fn new(players: &[PlayerId], nodes: usize) -> Self {
        Self {
            nodes: (0..nodes)
                .map(|_| NodeAnalysis {
                    round: RoundAnalysis::new(players),
                    turns: 0,
                    guesses: 0,
                    skipped: Vec::new(),
                    winner: None,
                    first_green: None,
                    assisted: false,
                })
                .collect(),
            players: players.to_vec(),
            unassigned_skips: vec![0; players.len()],
            actions: Vec::new(),
            hands: Vec::new(),
            scores: vec![0; players.len()],
            turns: 0,
        }
    }

    pub fn guess(&mut self, input: GraphGuessInput<'_>) {
        let index = input.index;
        let player = input.player;
        let slot = self
            .players
            .iter()
            .position(|&id| id == player)
            .expect("graph player");
        self.scores[slot] = self.scores[slot].saturating_add(input.points);
        let node = &mut self.nodes[index];
        node.round.guess(GuessAnalysisInput {
            turns: node.turns,
            guess_count: node.guesses,
            skipped: &node.skipped,
            player,
            mode: McGuessMode::Graph,
            result: input.result,
            previous: input.previous,
            character_count: input.character_count,
        });
        let mut actions = node.round.take_actions();
        let action = actions.first_mut().expect("accepted graph guess");
        if action.first_matching_wrong {
            node.first_green = Some(player);
        }
        if input.result.is_correct() {
            node.winner = Some(player);
            node.assisted = action.won_after_others_matching;
            self.hands
                .extend(node.round.consistency().map(|(player, consistent)| {
                    McGuessHandStatistics {
                        player,
                        consistent,
                        round_sweep: false,
                        match_sweep: false,
                        no_answers_top: false,
                    }
                }));
        }
        node.turns += 1;
        node.guesses += 1;
        node.skipped.clear();
        action.graph = Some(McGuessGraphActionStatistics {
            combo: input.combo,
            only_tag_neighbours: input.result.is_correct() && input.only_tag_neighbours,
            opening_win: input.result.is_correct() && self.turns == 0,
            first_green_nodes: self
                .nodes
                .iter()
                .filter(|node| node.first_green == Some(player))
                .count() as u32,
            assisted_nodes: self
                .nodes
                .iter()
                .filter(|node| node.winner == Some(player) && node.assisted)
                .count() as u32,
            combo_sweep: input.result.is_correct() && usize::from(input.combo) == input.entry_count,
            ..Default::default()
        });
        self.actions.extend(actions);
        self.turns += 1;
    }

    pub fn finish(&mut self) {
        let highest = self.scores.iter().copied().max().expect("graph players");
        let winners = self
            .players
            .iter()
            .zip(&self.scores)
            .filter_map(|(&player, &score)| {
                (score == highest).then_some(McGuessActionStatistics {
                    player,
                    graph: Some(McGuessGraphActionStatistics {
                        round_win: true,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
            })
            .collect::<Vec<_>>();
        self.actions.splice(0..0, winners);
    }

    pub fn skip(&mut self, index: Option<usize>, player: PlayerId) {
        self.turns += 1;
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

    pub fn take_actions(&mut self) -> Vec<McGuessActionStatistics> {
        mem::take(&mut self.actions)
    }

    pub fn take_hands(&mut self) -> Vec<McGuessHandStatistics> {
        mem::take(&mut self.hands)
    }

    pub fn statistics(
        &self,
        entries: impl Iterator<Item = (usize, McGuessEntryKind)>,
        allow_unknown: bool,
    ) -> Vec<(PlayerId, McGuessPlayStatistics)> {
        let mut totals: Vec<_> = self
            .players
            .iter()
            .copied()
            .zip(&self.unassigned_skips)
            .map(|(player, &skips)| {
                (
                    player,
                    McGuessPlayStatistics {
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

pub(super) struct GraphGuessInput<'a> {
    pub index: usize,
    pub player: PlayerId,
    pub result: &'a McGuessResult,
    pub previous: Option<u32>,
    pub character_count: Option<usize>,
    pub combo: u16,
    pub points: u32,
    pub only_tag_neighbours: bool,
    pub entry_count: usize,
}
