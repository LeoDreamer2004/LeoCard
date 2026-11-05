use super::mode::PlayMode;
use super::storage::MusicPreferences;
use super::task::Task;
use crate::app::presentation::GAME_SIDE_CONTROL_SCALE;
use bevy::prelude::*;
use leocard_music::{Account, LoginPoll, MusicClient, Playlist, QrLogin, SavedSession, Song};
use leocard_ui::{DraggableWindowOwner, DraggableWindowState};
use std::sync::Arc;
use std::time::{Duration, Instant};

pub(super) enum AccountResult {
    Qr(MusicClient, QrLogin),
    Poll(LoginPoll),
    LoggedIn(Account),
}

pub(super) enum CatalogResult {
    Playlists(Vec<Playlist>),
    Songs(u64, Vec<Song>),
}

pub(super) struct PreparedAudio {
    pub bytes: Arc<[u8]>,
    pub duration: Duration,
    pub trial: bool,
}

#[derive(Component)]
pub(super) struct MusicAudio;

#[derive(Component)]
pub(super) struct MusicClock;

#[derive(Resource)]
pub(crate) struct MusicState {
    pub(super) initialized: bool,
    pub(super) saved: Option<SavedSession>,
    pub(super) client: Option<MusicClient>,
    pub(super) account: Option<Account>,
    pub(super) remember: bool,
    pub(super) qr_key: Option<String>,
    pub(super) qr_image: Option<Handle<Image>>,
    pub(super) poll_at: Instant,
    pub(super) account_task: Option<Task<AccountResult>>,
    pub(super) catalog_task: Option<Task<CatalogResult>>,
    pub(super) audio_task: Option<Task<PreparedAudio>>,
    pub(super) status: String,
    pub(super) error: Option<String>,
    pub(super) playlists: Vec<Playlist>,
    pub(super) selected: Option<u64>,
    pub(super) songs: Arc<Vec<Song>>,
    pub(super) playlist_page: usize,
    pub(super) song_page: usize,
    pub(super) queue: Arc<Vec<Song>>,
    pub(super) queue_index: Option<usize>,
    pub(super) playing: Option<Song>,
    pub(super) audio_entity: Option<Entity>,
    pub(super) audio_source: Option<Handle<AudioSource>>,
    pub(super) paused: bool,
    pub(super) trial: bool,
    pub(super) duration: Duration,
    pub(super) volume: f32,
    pub(super) mode: PlayMode,
    pub(super) show_in_game: bool,
    pub(super) mini_window: DraggableWindowState,
}

impl Default for MusicState {
    fn default() -> Self {
        let (preferences, error) = match MusicPreferences::load() {
            Ok(preferences) => (preferences, None),
            Err(error) => (MusicPreferences::default(), Some(error)),
        };
        Self {
            initialized: false,
            remember: preferences.session.is_some(),
            saved: preferences.session,
            client: None,
            account: None,
            qr_key: None,
            qr_image: None,
            poll_at: Instant::now(),
            account_task: None,
            catalog_task: None,
            audio_task: None,
            status: "关联网易云账号，播放自己的歌单".to_owned(),
            error,
            playlists: Vec::new(),
            selected: None,
            songs: Arc::default(),
            playlist_page: 0,
            song_page: 0,
            queue: Arc::default(),
            queue_index: None,
            playing: None,
            audio_entity: None,
            audio_source: None,
            paused: false,
            trial: false,
            duration: Duration::ZERO,
            volume: preferences.volume,
            mode: preferences.mode,
            show_in_game: preferences.show_in_game,
            mini_window: DraggableWindowState {
                position: Vec2::new(
                    if cfg!(target_os = "android") {
                        36.0
                    } else {
                        12.0
                    },
                    8.0 * GAME_SIDE_CONTROL_SCALE,
                ),
                ..default()
            },
        }
    }
}

impl DraggableWindowOwner for MusicState {
    fn window_state(&mut self) -> &mut DraggableWindowState {
        &mut self.mini_window
    }
}

impl MusicState {
    pub(super) fn save(&mut self) {
        let result = (|| {
            let session = if self.remember {
                match (&self.client, &self.account) {
                    (Some(client), Some(account)) => Some(client.session(account.clone())?),
                    _ => self.saved.clone(),
                }
            } else {
                None
            };
            MusicPreferences {
                volume: self.volume,
                session,
                mode: self.mode,
                show_in_game: self.show_in_game,
            }
            .save()
        })();
        if let Err(error) = result {
            self.error = Some(error);
        }
    }

    pub(super) fn load_playlists(&mut self) -> Result<(), String> {
        let client = self.client.clone().ok_or("请先关联网易云账号")?;
        let account = self.account.as_ref().ok_or("请先关联网易云账号")?;
        let user_id = account.id;
        self.catalog_task = Some(Task::start(move || {
            client.playlists(user_id).map(CatalogResult::Playlists)
        })?);
        Ok(())
    }
}
