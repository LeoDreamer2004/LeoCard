use super::super::{
    ChatUiAction, ConnectionUiAction, LobbyUiAction, NavigationUiAction, SocialUiAction,
};
use crate::app::games::mahjong::actions::MahjongUiAction;
use crate::app::games::qigui523::actions::QiGui523UiAction;
use crate::app::games::shengji::actions::ShengjiUiAction;
use crate::app::games::texas_holdem::actions::TexasHoldemUiAction;
use crate::app::games::uno::actions::UnoUiAction;
#[cfg(feature = "developer")]
use crate::app::shell::DeveloperUiAction;
use bevy::prelude::*;

pub(super) type ButtonInteractions<'w, 's> =
    Query<'w, 's, (&'static Interaction, &'static UiAction), (Changed<Interaction>, With<Button>)>;

#[derive(Clone, Component)]
pub(crate) enum UiAction {
    Mahjong(MahjongUiAction),
    TexasHoldem(TexasHoldemUiAction),
    Uno(UnoUiAction),
    Shengji(ShengjiUiAction),
    QiGui523(QiGui523UiAction),
    Social(SocialUiAction),
    Chat(ChatUiAction),
    #[cfg(feature = "developer")]
    Developer(DeveloperUiAction),
    Connection(ConnectionUiAction),
    Navigation(NavigationUiAction),
    Lobby(LobbyUiAction),
}

#[derive(Clone, Message)]
pub(crate) struct PressedUiAction(pub UiAction);

pub(crate) trait DomainUiAction: Sized {
    fn extract(action: &UiAction) -> Option<&Self>;

    fn rebuilds_ui(&self) -> bool {
        true
    }
}

pub(crate) trait UiActionHandler<Context>: DomainUiAction {
    fn handle(&self, context: &mut Context);
}

impl UiAction {
    pub(super) fn rebuilds_ui(&self) -> bool {
        match self {
            Self::Mahjong(action) => action.rebuilds_ui(),
            Self::TexasHoldem(action) => action.rebuilds_ui(),
            Self::Uno(action) => action.rebuilds_ui(),
            Self::Shengji(action) => action.rebuilds_ui(),
            Self::QiGui523(action) => action.rebuilds_ui(),
            Self::Social(_) => false,
            Self::Chat(action) => action.rebuilds_ui(),
            #[cfg(feature = "developer")]
            Self::Developer(action) => action.rebuilds_ui(),
            Self::Navigation(action) => action.rebuilds_ui(),
            Self::Connection(action) => action.rebuilds_ui(),
            Self::Lobby(action) => action.rebuilds_ui(),
        }
    }
}
