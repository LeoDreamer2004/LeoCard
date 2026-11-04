use super::state::{GraphLog, LogAction, LogMessage};
use crate::app::runtime::{AvatarImages, ClientResource, GameStateMessage};
use bevy::prelude::*;
use leocard_game_common::PlayerId;
use leocard_protocol::{GameEvent, GameSnapshot, McGuessEvent, McGuessSnapshot, ServerEvent};

pub(super) fn sync_messages(
    client: Option<Res<ClientResource>>,
    mut updates: MessageReader<GameStateMessage>,
    mut log: ResMut<GraphLog>,
    avatars: Res<AvatarImages>,
) {
    let Some(game) = client
        .as_deref()
        .and_then(|client| client.0.model().game_snapshot())
        .and_then(GameSnapshot::mc_guess)
        .filter(|game| game.graph.is_some())
    else {
        updates.clear();
        if log.key.is_some() {
            *log = GraphLog::default();
        }
        return;
    };
    // Consume snapshots and skips in network order, even when several arrive in one frame.
    for update in updates.read() {
        match &update.0 {
            ServerEvent::GameSnapshot(snapshot) => {
                if let Some(snapshot) = snapshot.mc_guess()
                    && snapshot.match_id == game.match_id
                    && snapshot.progress.game_index == game.progress.game_index
                {
                    log.observe(snapshot, &avatars);
                }
            }
            ServerEvent::GameEvent(GameEvent::McGuess(McGuessEvent::GuessSkipped { player }))
                if log.key == Some((game.match_id, game.progress.game_index)) =>
            {
                log.push(game, &avatars, *player, LogAction::Skip);
            }
            _ => {}
        }
    }
    log.observe(game, &avatars);
}

impl GraphLog {
    fn observe(&mut self, game: &McGuessSnapshot, avatars: &AvatarImages) {
        let Some(graph) = &game.graph else {
            return;
        };
        let key = (game.match_id, game.progress.game_index);
        if self.key != Some(key) {
            *self = Self {
                key: Some(key),
                ..default()
            };
        }
        for record in graph.guesses.iter().skip(self.guesses) {
            self.push(
                game,
                avatars,
                record.guess.player,
                LogAction::Guess {
                    node: record.node,
                    guess: record.guess.result.name().to_owned(),
                    correct: record.guess.result.is_correct(),
                    combo: record.combo,
                },
            );
        }
        self.guesses = graph.guesses.len();
    }

    fn push(
        &mut self,
        game: &McGuessSnapshot,
        avatars: &AvatarImages,
        actor: PlayerId,
        action: LogAction,
    ) {
        let player = game.players.iter().find(|player| player.player.id == actor);
        let avatar = player.and_then(|player| {
            if actor == game.you {
                avatars.local.clone()
            } else {
                player
                    .player
                    .avatar
                    .and_then(|id| avatars.remote.get(&id).cloned())
            }
        });
        self.messages.push(LogMessage {
            name: player
                .map(|player| player.player.name.clone())
                .unwrap_or_else(|| "离线玩家".to_owned()),
            avatar,
            action,
        });
        self.version += 1;
    }
}
