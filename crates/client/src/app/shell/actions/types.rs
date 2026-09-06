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
    FocusInput(InputField),
    OpenHostGamePicker,
    CloseHostGamePicker,
    CreateRoom(GameKind),
    JoinRoom,
    ChooseAvatar,
    ClearAvatar,
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
    SelectSeat(SeatId),
    ToggleReady,
    UpdateRules(QiGuiRuleSet),
    UpdateTexasRules(TexasHoldemRuleSet),
    UpdateShengjiRules(ShengjiRuleSet),
    UpdateUnoRules(UnoRuleSet),
    UpdateMahjongRules(MahjongRuleSet),
    MahjongDiscard(MahjongTile),
    MahjongRespond(MahjongClaim),
    MahjongSelfDraw,
    MahjongConcealedKong(MahjongTileKind),
    MahjongAddedKong(MahjongTile),
    ToggleUnoModeMenu,
    CloseUnoModeMenu,
    ToggleUnoExpansionSettings,
    ToggleUnoCard(UnoCard),
    SubmitUnoCard,
    CloseUnoColorChoice,
    UnoChooseInitialColor(UnoColor),
    UnoPlayCard(UnoCard, Option<UnoColor>),
    UnoJumpIn(UnoCard),
    ToggleUnoSwapTarget(PlayerId),
    ConfirmUnoSwapTargets,
    UnoDrawCard,
    UnoPassAfterDraw,
    UnoAcceptDrawPenalty,
    UnoChallengeDrawFour,
    UnoResolveSkip,
    UnoCall,
    UnoReport(PlayerId),
    SetTexasRaiseTo(u32),
    TexasAct(TexasHoldemAction),
    ShengjiDeclare(Vec<ShengjiCard>),
    ConfirmShengjiBidPass,
    ShengjiBottomCopy(Vec<ShengjiCard>),
    DeclineBottomCopy,
    ToggleShengjiCard,
    ShengjiHint,
    ShowShengjiPreviousTrick,
    ToggleShengjiBuried,
    SubmitShengjiCards,
    DeclineFiveTrumpCrossing,
    StartGame,
    ReturnToLobby,
    PlayAgain,
    LeaveRoom,
    #[cfg(feature = "developer")]
    FocusDeveloperHand,
    ToggleInteractionMenu(PlayerId),
    SendInteraction {
        target: PlayerId,
        kind: PlayerInteractionKind,
    },
    ToggleChatPanel,
    ToggleAutoPlay,
    FocusChatInput,
    ToggleQuickVoiceMenu,
    ToggleEmojiMenu,
    SendQuickVoice(u8),
    SendEmoji(ChatEmoji),
    Hint,
    ToggleCard,
    Play,
    Pass,
}
