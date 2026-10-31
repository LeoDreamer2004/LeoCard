use crate::app::runtime::{ClientResource, GameNotice, GameStateMessage};
use bevy::prelude::*;
use leocard_game_common::PlayerId;
use leocard_protocol::{GameEvent, GameSnapshot, ServerEvent, UnoEvent, UnoSnapshot};
use leocard_uno::UnoChallengeResult;

pub(super) fn sync_notices(
    client: Option<Res<ClientResource>>,
    mut updates: MessageReader<GameStateMessage>,
    mut notices: MessageWriter<GameNotice>,
    mut snapshot: Local<Option<UnoSnapshot>>,
) {
    if client.is_none() {
        *snapshot = None;
        updates.clear();
        return;
    }
    for update in updates.read() {
        match &update.0 {
            ServerEvent::GameSnapshot(GameSnapshot::Uno(game)) => *snapshot = Some(game.clone()),
            ServerEvent::GameEvent(GameEvent::Uno(event)) => {
                if let Some(notice) = uno_event_notice(snapshot.as_ref(), event) {
                    notices.write(GameNotice(notice));
                }
            }
            ServerEvent::GameSnapshot(_)
            | ServerEvent::LobbySnapshot(_)
            | ServerEvent::LeftRoom
            | ServerEvent::RoomClosed
            | ServerEvent::KickedFromRoom => *snapshot = None,
            _ => {}
        }
    }
}

pub(crate) fn uno_event_notice(snapshot: Option<&UnoSnapshot>, event: &UnoEvent) -> Option<String> {
    let player_name = |player: PlayerId| {
        snapshot
            .and_then(|snapshot| {
                snapshot
                    .players
                    .iter()
                    .find(|item| item.player.id == player)
            })
            .map(|player| player.player.name.clone())
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
        UnoEvent::ActionAnalyzed { .. }
        | UnoEvent::ColorChosen { .. }
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
