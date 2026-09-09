use super::super::{InputField, PlayerProfilePage, ProfileGameTab};
use crate::app::games::mahjong::actions::MahjongUiAction;
use crate::app::games::qigui523::actions::QiGui523UiAction;
use crate::app::games::shengji::actions::ShengjiUiAction;
use crate::app::games::texas_holdem::actions::TexasHoldemUiAction;
use crate::app::games::uno::actions::UnoUiAction;
use crate::app::runtime::{AvatarImages, AvatarPicker, TableAppearance, TableFeltPicker};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use leocard_protocol::{ChatEmoji, GameKind, PlayerId, PlayerInteractionKind, SeatId};

pub(super) type ButtonInteractions<'w, 's> =
    Query<'w, 's, (&'static Interaction, &'static UiAction), (Changed<Interaction>, With<Button>)>;

#[derive(SystemParam)]
pub(super) struct AvatarUiResources<'w> {
    pub(super) avatar_images: ResMut<'w, AvatarImages>,
    pub(super) avatar_picker: ResMut<'w, AvatarPicker>,
}

#[derive(SystemParam)]
pub(super) struct AppearanceUiResources<'w> {
    pub(super) table_felt_picker: ResMut<'w, TableFeltPicker>,
    pub(super) table_appearance: ResMut<'w, TableAppearance>,
}

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
    Lobby(LobbyUiAction),
}

#[derive(Clone)]
pub(crate) enum SocialUiAction {
    ToggleInteractionMenu(PlayerId),
    ToggleAutoPlay,
    SendInteraction {
        target: PlayerId,
        kind: PlayerInteractionKind,
    },
}

#[derive(Clone)]
pub(crate) enum ChatUiAction {
    #[cfg(feature = "developer")]
    FocusDeveloperHand,
    TogglePanel,
    FocusInput,
    ToggleQuickVoiceMenu,
    ToggleEmojiMenu,
    SendQuickVoice(u8),
    SendEmoji(ChatEmoji),
}

#[derive(Clone)]
pub(crate) enum ConnectionUiAction {
    FocusInput(InputField),
    OpenHostGamePicker,
    CloseHostGamePicker,
    CreateRoom(GameKind),
    JoinRoom,
    ChooseAvatar,
    ClearAvatar,
}

#[derive(Clone)]
pub(crate) enum NavigationUiAction {
    ToggleProfile,
    OpenPlayerProfile(Box<PlayerProfilePage>),
    SelectProfileGameTab(ProfileGameTab),
    ToggleSettings,
    StartUpdate,
    OpenGitHubRepository,
    HideUpdateDialog,
    RestartToUpdate,
    ChooseTableFelt,
    UseDefaultTableFelt,
}

#[derive(Clone)]
pub(crate) enum LobbyUiAction {
    SelectSeat(SeatId),
    ToggleReady,
    StartGame,
    ReturnToLobby,
    PlayAgain,
    LeaveRoom,
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

macro_rules! impl_domain_action {
    ($action:ty, $variant:ident) => {
        impl DomainUiAction for $action {
            fn extract(action: &UiAction) -> Option<&Self> {
                let UiAction::$variant(action) = action else {
                    return None;
                };
                Some(action)
            }
        }
    };
}

impl_domain_action!(SocialUiAction, Social);
impl_domain_action!(ChatUiAction, Chat);
impl_domain_action!(ConnectionUiAction, Connection);
impl_domain_action!(NavigationUiAction, Navigation);
impl_domain_action!(LobbyUiAction, Lobby);

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
            Self::Connection(action) => action.rebuilds_ui(),
            Self::Lobby(action) => action.rebuilds_ui(),
        }
    }
}

impl ChatUiAction {
    fn rebuilds_ui(&self) -> bool {
        let rebuild = !matches!(
            self,
            Self::TogglePanel
                | Self::FocusInput
                | Self::ToggleQuickVoiceMenu
                | Self::SendQuickVoice(_)
        );
        #[cfg(feature = "developer")]
        let rebuild = rebuild && !matches!(self, Self::FocusDeveloperHand);
        rebuild
    }
}

impl NavigationUiAction {
    fn rebuilds_ui(&self) -> bool {
        !matches!(self, Self::OpenGitHubRepository)
    }
}
