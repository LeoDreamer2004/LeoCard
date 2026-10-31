use super::{ClientResource, ClientUpdateSet};
use crate::model::event::EventInbox;
use bevy::ecs::component::Mutable;
use bevy::prelude::*;
use leocard_game_common::PlayerId;
use leocard_protocol::{GameEvent, GameKind, MatchId, ServerEvent};
use std::marker::PhantomData;

/// Each accepted snapshot and event, in network order, before frame-level presentation.
#[derive(Message)]
pub(crate) struct GameStateMessage(pub ServerEvent);

#[derive(Message)]
pub(crate) struct GameNotice(pub String);

pub(crate) trait GameProjection: Resource<Mutability = Mutable> + Default {
    fn apply(&mut self, update: &ServerEvent);
}

pub(crate) struct GameProjectionPlugin<P>(PhantomData<P>);

impl<P> Default for GameProjectionPlugin<P> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<P: GameProjection> Plugin for GameProjectionPlugin<P> {
    fn build(&self, app: &mut App) {
        app.init_resource::<P>()
            .add_systems(Update, project::<P>.in_set(ClientUpdateSet::Project));
    }
}

fn project<P: GameProjection>(
    client: Option<Res<ClientResource>>,
    mut updates: MessageReader<GameStateMessage>,
    mut state: ResMut<P>,
) {
    if client.is_none() {
        *state = P::default();
        updates.clear();
        return;
    }
    for update in updates.read() {
        state.apply(&update.0);
    }
}

pub(crate) trait GameEventSource: Clone + Send + Sync + 'static {
    const KIND: GameKind;
    fn select(event: &GameEvent) -> Option<&Self>;
}

#[derive(Resource)]
pub(crate) struct GameEvents<E: GameEventSource> {
    key: Option<(MatchId, PlayerId)>,
    events: EventInbox<E>,
}

impl<E: GameEventSource> Default for GameEvents<E> {
    fn default() -> Self {
        Self {
            key: None,
            events: EventInbox::default(),
        }
    }
}

impl<E: GameEventSource> GameEvents<E> {
    pub(crate) fn take(&mut self) -> Vec<E> {
        self.events.take()
    }
}

impl<E: GameEventSource> GameProjection for GameEvents<E> {
    fn apply(&mut self, update: &ServerEvent) {
        match update {
            ServerEvent::GameSnapshot(snapshot) if snapshot.kind() == E::KIND => {
                let key = (snapshot.match_id(), snapshot.you());
                if self.key != Some(key) {
                    self.events.clear();
                    self.key = Some(key);
                }
            }
            ServerEvent::GameEvent(event) if self.key.is_some() => {
                if let Some(event) = E::select(event) {
                    self.events.push(event.clone());
                }
            }
            ServerEvent::GameSnapshot(_)
            | ServerEvent::LobbySnapshot(_)
            | ServerEvent::RoomClosed
            | ServerEvent::LeftRoom
            | ServerEvent::KickedFromRoom => *self = Self::default(),
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use leocard_mahjong::MahjongDrawOrigin;
    use leocard_protocol::{MahjongEvent, UnoEvent};

    #[test]
    fn ordered_game_events_survive_multiple_updates_until_consumed_and_clear_on_exit() {
        let mut events = GameEvents::<UnoEvent> {
            key: Some((MatchId([1; 16]), PlayerId(0))),
            ..default()
        };
        let first = UnoEvent::UnoCalled {
            player: PlayerId(0),
        };
        let second = UnoEvent::UnoCalled {
            player: PlayerId(1),
        };
        events.apply(&ServerEvent::GameEvent(GameEvent::Uno(first.clone())));
        events.apply(&ServerEvent::Heartbeat);
        events.apply(&ServerEvent::GameEvent(GameEvent::Mahjong(
            MahjongEvent::TileDrawn {
                origin: MahjongDrawOrigin::Normal,
                player: PlayerId(0),
            },
        )));
        events.apply(&ServerEvent::GameEvent(GameEvent::Uno(second.clone())));
        assert_eq!(events.take(), vec![first.clone(), second]);
        assert!(events.take().is_empty());
        events.apply(&ServerEvent::GameEvent(GameEvent::Uno(first.clone())));
        events.apply(&ServerEvent::LeftRoom);
        assert!(events.take().is_empty());
        events.apply(&ServerEvent::GameEvent(GameEvent::Uno(first)));
        assert!(events.take().is_empty());
    }
}
