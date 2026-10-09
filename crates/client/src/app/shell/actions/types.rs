use super::super::{
    AchievementUiAction, ChatUiAction, ConfirmationUiAction, ConnectionUiAction, LobbyUiAction,
    NavigationUiAction, ProfileUiAction, SettingsUiAction, ShopUiAction, SocialUiAction,
    UpdateUiAction,
};
use crate::app::games::mahjong::actions::MahjongUiAction;
use crate::app::games::qigui523::actions::QiGui523UiAction;
use crate::app::games::shengji::actions::ShengjiUiAction;
use crate::app::games::texas_holdem::actions::TexasHoldemUiAction;
use crate::app::games::uno::actions::UnoUiAction;
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
    Connection(ConnectionUiAction),
    Navigation(NavigationUiAction),
    Achievements(AchievementUiAction),
    Shop(ShopUiAction),
    Confirmation(ConfirmationUiAction),
    Profile(ProfileUiAction),
    Settings(SettingsUiAction),
    Update(UpdateUiAction),
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
            Self::Navigation(action) => action.rebuilds_ui(),
            Self::Achievements(action) => action.rebuilds_ui(),
            Self::Shop(action) => action.rebuilds_ui(),
            Self::Confirmation(_) => false,
            Self::Profile(action) => action.rebuilds_ui(),
            Self::Settings(action) => action.rebuilds_ui(),
            Self::Update(action) => action.rebuilds_ui(),
            Self::Connection(action) => action.rebuilds_ui(),
            Self::Lobby(action) => action.rebuilds_ui(),
        }
    }
}
