use super::ClientModel;
use super::types::GameEventInbox;
use leocard_protocol::{GameRules, PlayerId, UnoEvent, UnoPhaseView, UnoSnapshot};
use leocard_uno::UnoChallengeResult;

#[derive(Clone, Debug, Default)]
pub(super) struct UnoClientState {
    events: GameEventInbox<UnoEvent>,
}

impl ClientModel {
    pub fn take_uno_events(&mut self) -> Vec<UnoEvent> {
        self.games.uno.events.take()
    }

    pub(super) fn apply_uno_snapshot(&mut self, snapshot: UnoSnapshot) {
        if self.prepare_game_snapshot(snapshot.match_id, snapshot.host_port, snapshot.you) {
            self.games.uno.events.clear();
        }
        if let UnoPhaseView::Finished {
            reference_changes, ..
        } = &snapshot.phase
        {
            self.last_finished_match = Some((snapshot.match_id, reference_changes.clone()));
        }
        self.rules = Some(GameRules::Uno(snapshot.rules));
        self.store_game_snapshot(snapshot);
    }

    pub(super) fn apply_uno_event(&mut self, event: UnoEvent) {
        if let Some(notice) = uno_event_notice(self.uno_game(), &event) {
            self.notice.publish(notice);
        }
        self.games.uno.events.push(event);
    }
}

pub(super) fn uno_event_notice(snapshot: Option<&UnoSnapshot>, event: &UnoEvent) -> Option<String> {
    let player_name = |player: PlayerId| {
        snapshot
            .and_then(|snapshot| snapshot.players.iter().find(|item| item.id == player))
            .map(|player| player.name.clone())
            .unwrap_or_else(|| format!("玩家 {}", player.0 + 1))
    };
    match event {
        UnoEvent::ChallengeResolved {
            challenger,
            offender,
            result,
            penalized,
            count,
            ..
        } => Some(match result {
            UnoChallengeResult::Successful => format!(
                "{} 质疑成功，{} 摸 {count} 张",
                player_name(*challenger),
                player_name(*offender)
            ),
            UnoChallengeResult::Failed => format!(
                "{} 质疑失败，{} 摸 {count} 张",
                player_name(*challenger),
                player_name(*penalized)
            ),
        }),
        UnoEvent::UnoCalled { player } => Some(format!("{}：UNO!", player_name(*player))),
        UnoEvent::UnoReported {
            reporter, target, ..
        } => Some(format!(
            "{} 检举了 {}，罚摸 2 张",
            player_name(*reporter),
            player_name(*target)
        )),
        UnoEvent::DrawPenaltyReflected {
            player,
            target,
            count,
            ..
        } => Some(format!(
            "{} 将累计罚牌反弹给 {}，摸 {count} 张",
            player_name(*player),
            player_name(*target)
        )),
        UnoEvent::StackNumberRevealed { player, value, .. } => Some(format!(
            "{} 的随机堆叠翻出数字 {value}",
            player_name(*player)
        )),
        UnoEvent::SkipResolved { .. } => None,
        UnoEvent::ColorChosen { .. }
        | UnoEvent::CardPlayed { .. }
        | UnoEvent::CardsDrawn { .. }
        | UnoEvent::HandRefreshed { .. }
        | UnoEvent::SwapOneCardTaken { .. }
        | UnoEvent::SwapOneCompleted { .. }
        | UnoEvent::HandsTraded { .. }
        | UnoEvent::HandsPassed { .. }
        | UnoEvent::CardsDiscarded { .. }
        | UnoEvent::Flipped { .. }
        | UnoEvent::ColorRouletteResolved { .. }
        | UnoEvent::GameFinished { .. } => None,
    }
}
