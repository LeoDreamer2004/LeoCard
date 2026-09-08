use crate::{GameKind, PlayerId, RequestId};
use leocard_mahjong::MahjongHandReplacementError;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum RejectReason {
    Request(RequestViolation),
    Player(PlayerViolation),
    Room(RoomViolation),
    Game(GameViolation),
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum RequestViolation {
    ProtocolMismatch { expected: u16, received: u16 },
    RoomMismatch,
    DuplicateRequest { last_seen: RequestId },
    StaleRequest { last_seen: RequestId },
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum PlayerViolation {
    AlreadyJoined,
    NotJoined,
    NameEmpty,
    NameTooLong { max_chars: u16 },
    InvalidIdentityProof,
    InvalidAvatar,
    AvatarAlreadySet,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum RoomViolation {
    RoomFull,
    InvalidSeat,
    SeatTaken,
    MustSelectSeat,
    OnlyHostCanConfigure,
    OnlyHostCanStart,
    OnlyHostCanReturnToLobby,
    OnlyHostCanCloseRoom,
    NotEnoughPlayers { minimum: u8, actual: u8 },
    WaitingForPlayers { expected: u8, actual: u8 },
    PlayersNotReady { players: Vec<PlayerId> },
    InvalidChatMessage,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum GameViolation {
    InvalidRuleConfiguration,
    DeveloperFeatureUnavailable,
    InvalidDeveloperHand,
    WrongGame {
        expected: GameKind,
        received: GameKind,
    },
    GameNotStarted,
    GameNotFinished,
    GameAlreadyStarted,
    QiGui523(RuleViolation),
    TexasHoldem(TexasHoldemViolation),
    Shengji(ShengjiViolation),
    Uno(UnoViolation),
    Mahjong(MahjongViolation),
}

impl From<RuleViolation> for GameViolation {
    fn from(value: RuleViolation) -> Self {
        Self::QiGui523(value)
    }
}

impl From<TexasHoldemViolation> for GameViolation {
    fn from(value: TexasHoldemViolation) -> Self {
        Self::TexasHoldem(value)
    }
}

impl From<ShengjiViolation> for GameViolation {
    fn from(value: ShengjiViolation) -> Self {
        Self::Shengji(value)
    }
}

impl From<UnoViolation> for GameViolation {
    fn from(value: UnoViolation) -> Self {
        Self::Uno(value)
    }
}

impl From<MahjongViolation> for GameViolation {
    fn from(value: MahjongViolation) -> Self {
        Self::Mahjong(value)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum MahjongViolation {
    InvalidPlayer,
    NotPlayersTurn,
    WrongPhase,
    TileNotInHand,
    InvalidClaim,
    AlreadyResponded,
    CannotWin,
    CannotKong,
    InvalidDeveloperHand(MahjongHandReplacementError),
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum UnoViolation {
    InvalidPlayer,
    PlayerEliminated,
    NotPlayersTurn,
    GameAlreadyFinished,
    InitialColorChoiceRequired,
    InitialColorAlreadyChosen,
    CardNotInHand,
    CardDoesNotMatch,
    ColorRequired,
    UnexpectedColor,
    MustPlayDrawnCard,
    MustResolveDrawPenalty,
    NoDrawPenalty,
    CannotStack,
    CannotChallenge,
    MustDrawBeforePassing,
    MustResolveSkip,
    NoSkipToResolve,
    UnoCalloutDisabled,
    CannotCallUno,
    MustPlayAfterUno,
    CannotReportSelf,
    PlayerNotReportable,
    CannotPlayTogether,
    CannotJumpIn,
    MustResolveSwapEffect,
    NoSwapEffect,
    InvalidSwapTargets,
    DrawPileExhausted,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum RuleViolation {
    InvalidPlayer,
    NotPlayersTurn,
    GameAlreadyFinished,
    MustLeadWithCards,
    CardNotInHand,
    InvalidPattern,
    PlayDoesNotBeatCurrent,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum TexasHoldemViolation {
    InvalidPlayer,
    NotPlayersTurn,
    HandAlreadyComplete,
    PlayerCannotAct,
    MustPostBlind,
    NoBlindToPost,
    CannotCheckWhileFacingBet { amount_to_call: u32 },
    NothingToCall,
    RaiseMustExceedCurrentBet { current_bet: u32, target: u32 },
    RaiseBelowMinimum { minimum_target: u32, target: u32 },
    RaiseExceedsStack { maximum_target: u32, target: u32 },
    RaiseNotReopened,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum ShengjiViolation {
    InvalidPlayer,
    WrongPhase,
    InvalidDeclaration,
    DeclarationCardsNotOwned,
    CounterRequiresPair,
    CounterNotStronger,
    ProtectedSuitCanOnlyBeCounteredByNoTrump,
    DeclarationRequiresJoker,
    NoTrumpCannotOpen,
    NotDealer,
    WrongBuryCount { expected: u8, actual: u8 },
    CrossingNotEligible,
    CrossingAlreadyDecided,
    WrongCrossingCount { expected: u8, actual: u8 },
    CrossingMustIncludeAllTrumps,
    CrossingReturnNotRequired,
    CrossingAlreadyReturned,
    NotBottomCopyPlayer,
    CardsNotOwned,
    NotPlayersTurn,
    MustLeadWithCards,
    ThrowDisabled,
    InvalidPattern,
    WrongCardCount { expected: u8, actual: u8 },
    MustFollowCategory,
    MustFollowStructure,
}
