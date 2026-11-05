use crate::app::runtime::ClientResource;
use crate::app::shell::UiState;
use crate::player::config_file;
use bevy::prelude::*;
use leocard_client::ClientPhaseRef;
use leocard_protocol::GameKind;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum HomeCategory {
    #[default]
    All,
    Cards,
    Mahjong,
    Other,
}

impl HomeCategory {
    pub(crate) const ALL: [Self; 4] = [Self::All, Self::Cards, Self::Mahjong, Self::Other];

    pub(crate) fn title(self) -> &'static str {
        match self {
            Self::All => "全部",
            Self::Cards => "扑克",
            Self::Mahjong => "麻将",
            Self::Other => "其他",
        }
    }

    pub(crate) fn includes(self, game: GameKind) -> bool {
        match self {
            Self::All => true,
            Self::Mahjong => matches!(game, GameKind::Mahjong | GameKind::MahjongLink),
            Self::Other => game == GameKind::McGuess,
            Self::Cards => !matches!(
                game,
                GameKind::Mahjong | GameKind::MahjongLink | GameKind::McGuess
            ),
        }
    }
}

#[derive(Default, Serialize, Deserialize)]
struct HomeHistory {
    recent: Vec<GameKind>,
}

pub(crate) struct HomeUiState {
    pub category: HomeCategory,
    pub identity_open: bool,
    pub identity_progress: f32,
    pub join_open: bool,
    pub join_progress: f32,
    history: HomeHistory,
}

impl Default for HomeUiState {
    fn default() -> Self {
        let history = config_file("home-history.json")
            .and_then(|path| fs::read(path).ok())
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_default();
        Self {
            category: HomeCategory::All,
            identity_open: false,
            identity_progress: 0.0,
            join_open: false,
            join_progress: 0.0,
            history,
        }
    }
}

impl HomeUiState {
    pub(crate) fn recent(&self) -> Vec<GameKind> {
        let mut games = self.history.recent.clone();
        for game in [GameKind::Mahjong, GameKind::Shengji, GameKind::Uno] {
            if !games.contains(&game) {
                games.push(game);
            }
        }
        games.truncate(3);
        games
    }

    pub(crate) fn record(&mut self, game: GameKind) {
        self.history.recent.retain(|previous| *previous != game);
        self.history.recent.insert(0, game);
        self.history.recent.truncate(3);
        self.identity_open = false;
        self.join_open = false;
        if let Some(path) = config_file("home-history.json") {
            let save = || -> Result<(), String> {
                if let Some(parent) = path.parent() {
                    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
                }
                let bytes = serde_json::to_vec(&self.history).map_err(|error| error.to_string())?;
                let temporary = path.with_extension("json.tmp");
                fs::write(&temporary, bytes).map_err(|error| error.to_string())?;
                fs::rename(temporary, path).map_err(|error| error.to_string())
            };
            if let Err(error) = save() {
                warn!("无法保存最近常玩：{error}");
            }
        }
    }
}

pub(super) fn track_recent_game(
    client: Option<Res<ClientResource>>,
    mut ui: ResMut<UiState>,
    mut last: Local<Option<GameKind>>,
) {
    let Some(client) = client else {
        *last = None;
        return;
    };
    if client.is_added() {
        *last = None;
    }
    let kind = client
        .0
        .model()
        .lobby()
        .map(|lobby| lobby.game)
        .or_else(|| client.0.model().game_snapshot().map(|game| game.kind()));
    if let Some(kind) = kind
        && *last != Some(kind)
    {
        ui.home.record(kind);
        *last = Some(kind);
        ui.dirty = true;
    }
}

pub(crate) fn can_edit_identity(client: Option<&ClientResource>) -> bool {
    client.is_none_or(|client| {
        matches!(
            client.0.model().phase(),
            ClientPhaseRef::Idle | ClientPhaseRef::Closed
        )
    })
}
