use super::MahjongBotObservation;
use super::search::{Position, Search};
use crate::{MahjongClaim, MahjongMeld, MahjongMeldKind, MahjongTile, MahjongTileKind};
use leocard_game_common::SearchBudget;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MahjongBotAction {
    Discard(MahjongTile),
    Respond(MahjongClaim),
    SelfDraw,
    ConcealedKong(MahjongTileKind),
    AddedKong(MahjongTile),
}

pub struct MahjongBot;

impl MahjongBot {
    pub fn choose(observation: &MahjongBotObservation, budget: &SearchBudget) -> MahjongBotAction {
        if let Some(action) = observation.immediate_action() {
            return action;
        }
        let mut search = Search::new(observation, budget);
        let mut root = Position {
            counts: [0; 34],
            melds: observation.melds.clone(),
            replacement: false,
        };
        for tile in &observation.hand {
            root.counts[tile.kind().index34().expect("concealed tile")] += 1;
        }
        let mut candidates = Vec::new();
        for &action in &observation.options {
            let Some((position, discard)) = after_action(observation, &root, action, &mut search)
            else {
                continue;
            };
            let risk = discard.map_or(0.0, |index| observation.danger(index));
            let value = search.evaluate(&position, &observation.unseen) - risk;
            candidates.push((action, position, risk, value));
        }
        candidates.sort_by(|left, right| right.3.total_cmp(&left.3));
        let mut best = candidates
            .first()
            .map_or(observation.fallback(), |candidate| candidate.0);
        candidates.truncate(5);
        for depth in 1..=3 {
            let mut choices = Vec::new();
            for (action, position, risk, _) in &candidates {
                let Some(value) = search.future(position, &observation.unseen, depth) else {
                    return best;
                };
                let call_cost = if matches!(action, MahjongBotAction::Respond(claim) if *claim != MahjongClaim::Pass)
                {
                    1.5
                } else {
                    0.0
                };
                let robbing = if matches!(action, MahjongBotAction::AddedKong(tile) if observation.danger(tile.kind().index34().expect("tile")) > 5.0)
                {
                    6.0
                } else {
                    0.0
                };
                choices.push((*action, value - risk - call_cost - robbing));
            }
            if let Some(choice) = choices
                .into_iter()
                .max_by(|left, right| left.1.total_cmp(&right.1))
            {
                best = choice.0;
            }
        }
        best
    }
}

fn after_action(
    observation: &MahjongBotObservation,
    root: &Position,
    action: MahjongBotAction,
    search: &mut Search<'_>,
) -> Option<(Position, Option<usize>)> {
    let mut position = root.clone();
    let mut discard = None;
    match action {
        MahjongBotAction::Discard(tile) => {
            let index = tile.kind().index34()?;
            position.counts[index] -= 1;
            discard = Some(index);
        }
        MahjongBotAction::ConcealedKong(tile) => {
            position.counts[tile.index34()?] -= 4;
            position.melds.push(MahjongMeld::concealed_kong(tile));
            position.replacement = true;
        }
        MahjongBotAction::AddedKong(tile) => {
            position.counts[tile.kind().index34()?] -= 1;
            let meld = position
                .melds
                .iter_mut()
                .find(|meld| meld.kind() == MahjongMeldKind::Pung && meld.tile() == tile.kind())?;
            *meld = MahjongMeld::melded_kong(tile.kind(), meld.claimed_from()?);
            position.replacement = true;
        }
        MahjongBotAction::Respond(MahjongClaim::Pass) => {}
        MahjongBotAction::Respond(claim) => {
            let (source, tile) = observation.claim?;
            let index = tile.index34()?;
            let meld = match claim {
                MahjongClaim::Pung => {
                    position.counts[index] -= 2;
                    MahjongMeld::pung(tile, source)
                }
                MahjongClaim::Kong => {
                    position.counts[index] -= 3;
                    position.replacement = true;
                    MahjongMeld::melded_kong(tile, source)
                }
                MahjongClaim::Chow { start } => {
                    let MahjongTileKind::Suited { suit, .. } = tile else {
                        return None;
                    };
                    let first = suit.index() * 9 + usize::from(start) - 1;
                    for held in first..first + 3 {
                        if held != index {
                            position.counts[held] -= 1;
                        }
                    }
                    MahjongMeld::chow(suit, start, source)
                }
                _ => return None,
            };
            position.melds.push(meld);
            if !position.replacement {
                let (index, _) = search
                    .best_discards(&position, &observation.unseen, 1)
                    .first()
                    .copied()?;
                position.counts[index] -= 1;
                discard = Some(index);
            }
        }
        _ => return None,
    }
    Some((position, discard))
}
