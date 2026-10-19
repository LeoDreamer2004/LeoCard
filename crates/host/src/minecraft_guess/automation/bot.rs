use leocard_minecraft_guess::{Catalogue, GameState};

pub(super) struct Policy;

impl Policy {
    pub fn act(game: &mut GameState, current: u8) -> bool {
        // Bots use only public feedback; never peek at the chosen answer.
        if let Some(graph) = game.graph_view() {
            let node = graph.selected_node();
            if node.kind.is_none() || node.revealed() {
                if let Some(next) = graph
                    .nodes
                    .iter()
                    .find(|node| node.kind.is_some() && !node.revealed())
                {
                    if game.select_node(current, next.number).is_err() {
                        return false;
                    }
                } else {
                    return false;
                }
            }
        }
        let view = game.view();
        let full_feedback = cfg!(debug_assertions) || cfg!(feature = "developer");
        let candidates: Vec<_> = Catalogue::bundled()
            .search(view.kind, "")
            .filter(|entry| {
                (view.character_count == 0
                    || entry.name.chars().count() == usize::from(view.character_count))
                    && view.guesses.iter().all(|guess| {
                        guess.result.accepts_candidate_matching(entry, |label| {
                            full_feedback || !matches!(label, "硬度" | "爆炸抗性")
                        })
                    })
            })
            .collect();
        if let Some(entry) = candidates.get(fastrand::usize(..candidates.len().max(1))) {
            game.guess(current, &entry.id)
                .expect("a public candidate is valid");
            true
        } else if view.current_player == Some(current) {
            game.skip(current).expect("current player can skip");
            true
        } else {
            false
        }
    }
}
