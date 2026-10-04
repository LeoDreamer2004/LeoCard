use super::{QiGui523Session, from_core_player};
use crate::Delivery;
use leocard_protocol::{GameEvent, PublicPlay, QiGui523Event, QiGui523ProfileStats, ServerEvent};
use leocard_qigui523::{
    GameError, QiGuiActionContext, QiGuiCard, QiGuiPlayerId, QiGuiPlayerStatistics, classify,
};

impl QiGui523Session {
    /// One accepted-action path for manual play, timers, bots and disconnected players.
    pub(super) fn perform_action(
        &mut self,
        player: QiGuiPlayerId,
        cards: Option<&[QiGuiCard]>,
    ) -> Result<Vec<QiGui523Event>, GameError> {
        let game = self.game.as_mut().expect("actions require an active game");
        let before = QiGuiActionContext::capture(game, player);
        let outcome = match cards {
            Some(cards) => game.play_cards(player, cards)?,
            None => game.pass(player)?,
        };
        let play =
            cards.map(|cards| classify(cards, game.rules()).expect("accepted play is valid"));
        let mut events = Vec::new();
        if let Some(play) = &play {
            events.push(QiGui523Event::PlayEffect {
                player: from_core_player(player),
                play: PublicPlay {
                    kind: play.kind().clone(),
                    cards: play.cards().to_vec(),
                },
            });
        }
        let statistics = self
            .statistics
            .as_mut()
            .expect("an active game has statistics");
        events.extend(
            statistics
                .observe(
                    before.expect("an accepted action has a playing context"),
                    &outcome,
                    play.as_ref(),
                    game,
                )
                .into_iter()
                .map(|statistics| QiGui523Event::ActionAnalyzed {
                    player: from_core_player(statistics.player),
                    statistics: Box::new(statistics),
                }),
        );
        Ok(events)
    }

    pub(super) fn broadcast_action_events(&self, events: Vec<QiGui523Event>) -> Vec<Delivery> {
        self.room
            .broadcast_game_events(events, self.match_id.map(|id| (id, None)))
            .into_iter()
            .filter(|delivery| match &delivery.message.event {
                ServerEvent::GameEvent(GameEvent::QiGui523(QiGui523Event::ActionAnalyzed {
                    player,
                    ..
                })) => self.room.player_id(delivery.recipient) == Some(*player),
                _ => true,
            })
            .collect()
    }
}

pub(super) fn profile_statistics(progress: &QiGuiPlayerStatistics) -> QiGui523ProfileStats {
    QiGui523ProfileStats {
        straight_plays: progress.straight_plays,
        consecutive_pair_plays: progress.consecutive_pair_plays,
        airplane_plays: progress.airplane_plays,
        bomb_plays: progress.bomb_plays,
        heaven_bomb_plays: progress.heaven_bomb_plays,
        longest_straight: progress.longest_straight,
        longest_consecutive_pairs: progress.longest_consecutive_pairs,
        longest_airplane: progress.longest_airplane,
        ..Default::default()
    }
}
