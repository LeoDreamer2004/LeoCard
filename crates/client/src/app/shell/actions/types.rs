use super::super::*;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use leocard_mahjong::{MahjongClaim, MahjongRuleSet, MahjongTile, MahjongTileKind};
use leocard_protocol::{ChatEmoji, GameKind, PlayerId, PlayerInteractionKind, SeatId};
use leocard_qigui523::QiGuiRuleSet;
use leocard_shengji::{ShengjiCard, ShengjiRuleSet};
use leocard_texas_holdem::{TexasHoldemAction, TexasHoldemRuleSet};
use leocard_uno::{UnoCard, UnoColor, UnoRuleSet};

pub(super) type ButtonInteractions<'w, 's> =
    Query<'w, 's, (&'static Interaction, &'static UiAction), (Changed<Interaction>, With<Button>)>;

#[derive(SystemParam)]
pub struct LocalUiResources<'w> {
    pub(super) avatar_images: ResMut<'w, AvatarImages>,
    pub(super) avatar_picker: ResMut<'w, AvatarPicker>,
    pub(super) table_felt_picker: ResMut<'w, TableFeltPicker>,
    pub(super) table_appearance: ResMut<'w, TableAppearance>,
}

#[derive(Clone, Component)]
pub enum UiAction {
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
pub enum MahjongUiAction {
    UpdateRules(MahjongRuleSet),
    Discard(MahjongTile),
    Respond(MahjongClaim),
    SelfDraw,
    ConcealedKong(MahjongTileKind),
    AddedKong(MahjongTile),
}

#[derive(Clone)]
pub enum TexasHoldemUiAction {
    UpdateRules(TexasHoldemRuleSet),
    SetRaiseTo(u32),
    Act(TexasHoldemAction),
}

#[derive(Clone)]
pub enum UnoUiAction {
    UpdateRules(UnoRuleSet),
    ToggleModeMenu,
    CloseModeMenu,
    ToggleExpansionSettings,
    ToggleCard(UnoCard),
    SubmitCard,
    CloseColorChoice,
    ChooseInitialColor(UnoColor),
    PlayCard(UnoCard, Option<UnoColor>),
    JumpIn(UnoCard),
    ToggleSwapTarget(PlayerId),
    ConfirmSwapTargets,
    DrawCard,
    PassAfterDraw,
    AcceptDrawPenalty,
    ChallengeDrawFour,
    ResolveSkip,
    Call,
    Report(PlayerId),
}

#[derive(Clone)]
pub enum ShengjiUiAction {
    UpdateRules(ShengjiRuleSet),
    Declare(Vec<ShengjiCard>),
    ConfirmBidPass,
    BottomCopy(Vec<ShengjiCard>),
    DeclineBottomCopy,
    ToggleCard,
    Hint,
    ShowPreviousTrick,
    ToggleBuried,
    SubmitCards,
    DeclineFiveTrumpCrossing,
}

#[derive(Clone)]
pub enum QiGui523UiAction {
    UpdateRules(QiGuiRuleSet),
    Hint,
    ToggleCard,
    Play,
    Pass,
}

#[derive(Clone)]
pub enum SocialUiAction {
    ToggleInteractionMenu(PlayerId),
    ToggleAutoPlay,
    SendInteraction {
        target: PlayerId,
        kind: PlayerInteractionKind,
    },
}

#[derive(Clone)]
pub enum ChatUiAction {
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
pub enum ConnectionUiAction {
    FocusInput(InputField),
    OpenHostGamePicker,
    CloseHostGamePicker,
    CreateRoom(GameKind),
    JoinRoom,
    ChooseAvatar,
    ClearAvatar,
}

#[derive(Clone)]
pub enum NavigationUiAction {
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
pub enum LobbyUiAction {
    SelectSeat(SeatId),
    ToggleReady,
    StartGame,
    ReturnToLobby,
    PlayAgain,
    LeaveRoom,
}

#[derive(Clone, Message)]
pub struct PressedUiAction(pub UiAction);

pub trait DomainUiAction: Sized {
    fn extract(action: &UiAction) -> Option<&Self>;
}

pub trait UiActionHandler<Context>: DomainUiAction {
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

impl_domain_action!(MahjongUiAction, Mahjong);
impl_domain_action!(TexasHoldemUiAction, TexasHoldem);
impl_domain_action!(UnoUiAction, Uno);
impl_domain_action!(ShengjiUiAction, Shengji);
impl_domain_action!(QiGui523UiAction, QiGui523);
impl_domain_action!(SocialUiAction, Social);
impl_domain_action!(ChatUiAction, Chat);
impl_domain_action!(ConnectionUiAction, Connection);
impl_domain_action!(NavigationUiAction, Navigation);
impl_domain_action!(LobbyUiAction, Lobby);

impl UiAction {
    pub(super) fn rebuilds_ui(&self) -> bool {
        match self {
            Self::Shengji(action) => action.rebuilds_ui(),
            Self::QiGui523(action) => action.rebuilds_ui(),
            Self::Social(_) => false,
            Self::Chat(action) => action.rebuilds_ui(),
            Self::Navigation(action) => action.rebuilds_ui(),
            Self::Mahjong(_)
            | Self::TexasHoldem(_)
            | Self::Uno(_)
            | Self::Connection(_)
            | Self::Lobby(_) => true,
        }
    }
}

impl ShengjiUiAction {
    fn rebuilds_ui(&self) -> bool {
        !matches!(self, Self::ToggleCard)
    }
}

impl QiGui523UiAction {
    fn rebuilds_ui(&self) -> bool {
        !matches!(self, Self::ToggleCard | Self::Pass)
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
