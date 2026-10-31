use super::ShengjiSession;
use crate::Delivery;
use leocard_protocol::{GameEvent, ServerEvent, ShengjiEvent};
use leocard_shengji::{ShengjiActionOutcome, ShengjiPhase};

impl ShengjiSession {
    pub(super) fn first_trick_cut_event(
        &self,
        outcome: &ShengjiActionOutcome,
    ) -> Option<ShengjiEvent> {
        let game = self.game.as_ref()?;
        let player = match outcome {
            ShengjiActionOutcome::Played { player, .. } if game.history().is_empty() => *player,
            ShengjiActionOutcome::TrickComplete(trick) if game.history().len() == 1 => {
                trick.plays.last()?.0
            }
            ShengjiActionOutcome::HandComplete(_) if game.history().len() == 1 => {
                game.history().last()?.plays.last()?.0
            }
            _ => return None,
        };
        game.first_trick_cut_dealer(player)
            .then_some(ShengjiEvent::FirstTrickDealerCut { player })
    }

    pub(super) fn hand_analysis_events(&self) -> Vec<ShengjiEvent> {
        let Some(game) = self
            .game
            .as_ref()
            .filter(|game| matches!(game.phase(), ShengjiPhase::Finished(_)))
        else {
            return Vec::new();
        };
        game.players()
            .iter()
            .map(|player| ShengjiEvent::HandAnalyzed {
                player: player.id,
                statistics: Box::new(
                    game.hand_statistics(player.id)
                        .expect("a settled hand has private statistics for each player"),
                ),
                match_statistics: self.statistics.match_statistics,
            })
            .collect()
    }

    pub(super) fn redeal_event(&self) -> ShengjiEvent {
        let ShengjiPhase::RedealRequired(reason) =
            self.game.as_ref().expect("redeal requires a game").phase()
        else {
            unreachable!("only the core redeal transition emits a redeal event");
        };
        ShengjiEvent::RedealRequired { reason: *reason }
    }

    pub(super) fn broadcast_events(&self, events: Vec<ShengjiEvent>) -> Vec<Delivery> {
        self.room
            .broadcast_game_events(events, self.match_id.map(|id| (id, Some(self.hand_number))))
            .into_iter()
            .filter(|delivery| match &delivery.message.event {
                ServerEvent::GameEvent(GameEvent::Shengji(ShengjiEvent::HandAnalyzed {
                    player,
                    ..
                })) => self.room.player_id(delivery.recipient) == Some(*player),
                _ => true,
            })
            .collect()
    }
}
