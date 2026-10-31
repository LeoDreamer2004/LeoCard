#[cfg(not(target_os = "android"))]
use super::super::SakuraUiAction;
use super::super::{
    AchievementUiAction, ChatUiAction, ConfirmationUiAction, ConnectionUiAction, LobbyUiAction,
    NavigationUiAction, ProfileUiAction, SettingsUiAction, ShopUiAction, SocialUiAction,
    UpdateUiAction,
};
use crate::app::games::blackjack::BlackjackUiAction;
use crate::app::games::doudizhu::DoudizhuUiAction;
use crate::app::games::mahjong::actions::MahjongUiAction;
use crate::app::games::mc_guess::McGuessUiAction;
use crate::app::games::qigui523::actions::QiGui523UiAction;
use crate::app::games::shengji::actions::ShengjiUiAction;
use crate::app::games::texas::actions::TexasUiAction;
use crate::app::games::uno::actions::UnoUiAction;
use bevy::prelude::*;
use bevy::ui_widgets::Button;

pub(super) type ButtonActions<'w, 's> = Query<'w, 's, &'static UiAction, With<Button>>;

#[derive(Clone, Component)]
pub(crate) enum UiAction {
    Blackjack(BlackjackUiAction),
    Doudizhu(DoudizhuUiAction),
    Mahjong(MahjongUiAction),
    McGuess(McGuessUiAction),
    Texas(TexasUiAction),
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
    #[cfg(not(target_os = "android"))]
    Sakura(SakuraUiAction),
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
            Self::Blackjack(action) => action.rebuilds_ui(),
            Self::Doudizhu(action) => action.rebuilds_ui(),
            Self::Mahjong(action) => action.rebuilds_ui(),
            Self::McGuess(action) => action.rebuilds_ui(),
            Self::Texas(action) => action.rebuilds_ui(),
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
            #[cfg(not(target_os = "android"))]
            Self::Sakura(action) => action.rebuilds_ui(),
        }
    }
}
