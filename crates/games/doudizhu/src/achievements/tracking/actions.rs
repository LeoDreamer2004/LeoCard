use super::state::{DoudizhuAchievementTracker, DoudizhuActionContext, is_airplane, is_bomb};
use crate::achievements::{DoudizhuAchievementFacts, DoudizhuPlayFacts};
use crate::{DoudizhuAction, DoudizhuCard, DoudizhuGameState, DoudizhuPhase, classify};
use leocard_game_common::PlayerId;
use std::array;

impl DoudizhuAchievementTracker {
    /// Call only after the rules accept an action, so rejected attempts never affect streaks.
    pub fn observe_action(
        &mut self,
        player: PlayerId,
        action: &DoudizhuAction,
        before: DoudizhuActionContext,
        game: &DoudizhuGameState,
    ) -> Vec<(PlayerId, DoudizhuAchievementFacts)> {
        if before.phase == DoudizhuPhase::Bidding && game.phase == DoudizhuPhase::Playing {
            for (index, progress) in self.players.iter_mut().enumerate() {
                let hand = game
                    .hand(PlayerId::from_index(index))
                    .expect("seated player has a hand");
                let mut ranks: [Vec<DoudizhuCard>; 14] = array::from_fn(|_| Vec::new());
                for card in hand {
                    // Keep both joker ranks together so the rules validate a complete rocket.
                    ranks[usize::from(card.rank().min(13))].push(*card);
                }
                progress.hand.unavailable_exchange =
                    ranks[..4].iter().all(|cards| cards.len() == 1) && ranks[4].is_empty();
                progress.hand.all_bomb_hand = !hand.is_empty()
                    && ranks.iter().filter(|cards| !cards.is_empty()).all(|cards| {
                        classify(cards, game.rules).is_some_and(|play| is_bomb(play.kind))
                    });
            }
        }
        match action {
            DoudizhuAction::Play(_) => {
                let (_, play) = game
                    .table
                    .as_ref()
                    .expect("accepted play remains on the table");
                if before.previous.is_none() {
                    self.clear_trick_bombs();
                }
                let bomb = is_bomb(play.kind);
                let hand = &mut self.players[usize::from(player)].hand;
                hand.play_count += 1;
                hand.consecutive_bombs = if bomb { hand.consecutive_bombs + 1 } else { 0 };
                hand.trick_bombs += u32::from(bomb);
                let mut events = vec![(
                    player,
                    DoudizhuAchievementFacts::Play(DoudizhuPlayFacts {
                        mode: game.rules.mode,
                        kind: play.kind,
                        card_count: play.cards.len() as u8,
                        rank: play.rank,
                        is_lead: before.previous.is_none(),
                        beats_airplane: is_airplane(play.kind)
                            && before.previous.as_ref().is_some_and(|(other, previous)| {
                                *other != player && is_airplane(previous.kind)
                            }),
                        trick_bombs: hand.trick_bombs,
                        consecutive_bombs: hand.consecutive_bombs,
                    }),
                )];
                if bomb {
                    events.extend(
                        self.flower_senders
                            .drain(..)
                            .map(|source| (source, DoudizhuAchievementFacts::FlowerBomb)),
                    );
                }
                self.flower_senders.clear();
                events
            }
            DoudizhuAction::Pass => {
                self.players[usize::from(player)].hand.consecutive_bombs = 0;
                self.flower_senders.clear();
                if game.table.is_none() {
                    self.clear_trick_bombs();
                }
                Vec::new()
            }
            DoudizhuAction::Bid(_) => Vec::new(),
        }
    }

    fn clear_trick_bombs(&mut self) {
        for player in &mut self.players {
            player.hand.trick_bombs = 0;
        }
    }
}
