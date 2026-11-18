use crate::{MatchId, PlayerPublicInfo, PlayerReferenceChange, SeatId};
use leocard_draw_guess::{DrawAction, DrawGuessRuleSet, DrawGuessView, WordLibrary};
use leocard_game_common::PlayerId;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct DrawGuessSnapshot {
    pub match_id: MatchId,
    pub host_port: u16,
    pub you: PlayerId,
    pub host: PlayerId,
    pub rules: DrawGuessRuleSet,
    pub players: Vec<DrawGuessPlayerView>,
    pub state: DrawGuessView,
    pub match_complete: bool,
    pub reference_changes: Vec<PlayerReferenceChange>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct DrawGuessPlayerView {
    pub player: PlayerPublicInfo,
    pub seat: SeatId,
    pub ready: bool,
    pub connected: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum DrawGuessCommand {
    UpdateRules { rules: DrawGuessRuleSet },
    ChooseWord { turn: u32, choice: u8 },
    Draw { turn: u32, action: DrawAction },
    Guess { turn: u32, text: String },
    SelectLibrary { library: Option<WordLibrary> },
    UploadLibrary { library: WordLibrary },
    ResolveLibraryUpload { offer_id: u64, accept: bool },
}

/// Drawing deltas are independent of the private, recipient-specific round view.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct DrawGuessEvent {
    pub match_id: MatchId,
    pub turn: u32,
    pub revision: u64,
    pub action: DrawAction,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct WordLibraryInfo {
    pub title: String,
    pub word_count: u32,
    pub fingerprint: u64,
}

impl From<&WordLibrary> for WordLibraryInfo {
    fn from(library: &WordLibrary) -> Self {
        Self {
            title: library.title.clone(),
            word_count: library.words.len() as u32,
            fingerprint: library.fingerprint(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct WordLibraryOffer {
    pub id: u64,
    pub sender: PlayerId,
    pub sender_name: String,
    pub library: WordLibrary,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum DrawGuessLibraryEvent {
    State {
        selected: WordLibraryInfo,
        offers: Vec<WordLibraryOffer>,
    },
    UploadResolved {
        title: String,
        accepted: bool,
    },
}
