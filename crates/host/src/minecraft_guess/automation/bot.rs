use leocard_minecraft_guess::{Catalogue, GameState};

pub(super) struct Policy;

impl Policy {
    pub fn act(game: &mut GameState, current: u8) {
        // Bots use only public feedback; never peek at the chosen answer.
        let view = game.view();
        let full_feedback = cfg!(debug_assertions) || cfg!(feature = "developer");
        let candidates: Vec<_> = Catalogue::bundled()
            .search(view.kind, "")
            .filter(|entry| {
                entry.name.chars().count() == usize::from(view.character_count)
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
        } else {
            game.skip(current).expect("current player can skip");
        }
    }
}
