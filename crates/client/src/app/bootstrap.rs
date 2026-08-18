//! Application bootstrap, persistence, image normalization, and asset loading.

use super::*;

/// Cargo 运行工作区成员时，Bevy 以该成员的 `CARGO_MANIFEST_DIR` 为基准；发布后的
/// 独立程序则使用可执行文件目录。开发期指向工作区素材，发布期保留 Bevy 的标准
/// `assets` 旁置布局。显式的 `BEVY_ASSET_ROOT` 始终拥有最高优先级。
pub(super) fn asset_file_path() -> String {
    if std::env::var_os("BEVY_ASSET_ROOT").is_some() {
        return "assets".to_owned();
    }
    if std::env::var_os("CARGO_MANIFEST_DIR").is_some() {
        return "../../assets".to_owned();
    }
    if std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(ToOwned::to_owned))
        .is_some_and(|directory| directory.join("../../assets").is_dir())
    {
        "../../assets".to_owned()
    } else {
        "assets".to_owned()
    }
}

fn preferences_path() -> Option<PathBuf> {
    if let Some(directory) = std::env::var_os("LEOCARD_CONFIG_DIR") {
        return Some(PathBuf::from(directory).join("client.prefs"));
    }
    #[cfg(target_os = "windows")]
    let base = std::env::var_os("APPDATA").map(PathBuf::from);
    #[cfg(target_os = "macos")]
    let base = std::env::var_os("HOME")
        .map(PathBuf::from)
        .map(|home| home.join("Library/Application Support"));
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME")
                .map(PathBuf::from)
                .map(|home| home.join(".config"))
        });
    base.map(|base| base.join("leocard/client.prefs"))
}

fn player_profile_path() -> Option<PathBuf> {
    Some(preferences_path()?.parent()?.join("profile.dat"))
}

impl LocalPlayerProfile {
    pub(super) fn load_or_create() -> Result<Self, String> {
        let path = player_profile_path().ok_or_else(|| "无法确定玩家档案目录".to_owned())?;
        if path.is_file() {
            let bytes = fs::read(&path).map_err(|error| format!("无法读取玩家档案：{error}"))?;
            let stored = decode_player_profile(&bytes)?;
            return Ok(Self {
                identity: PlayerIdentity::from_secret_bytes(stored.secret_key),
                rating: PlayerRatingProfile {
                    reference_points: stored.games.qigui523.reference_points,
                    completed_games: stored.games.qigui523.completed_games,
                    applied_matches: stored.games.qigui523.applied_matches.into_iter().collect(),
                    last_change: None,
                },
                game_profiles: PlayerGameProfiles {
                    qigui523: stored.games.qigui523.qigui523_stats,
                    texas_holdem: stored.games.texas_holdem_stats,
                    shengji: stored.games.shengji_stats,
                    uno: stored.games.uno_stats,
                    interactions: stored.games.interaction_stats,
                },
            });
        }

        let identity =
            PlayerIdentity::generate().map_err(|error| format!("无法生成玩家身份：{error}"))?;
        let profile = Self {
            identity,
            rating: PlayerRatingProfile {
                reference_points: 0,
                completed_games: 0,
                applied_matches: HashSet::new(),
                last_change: None,
            },
            game_profiles: PlayerGameProfiles::default(),
        };
        profile.save()?;
        Ok(profile)
    }

    pub(super) fn save(&self) -> Result<(), String> {
        let path = player_profile_path().ok_or_else(|| "无法确定玩家档案目录".to_owned())?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|error| format!("无法创建档案目录：{error}"))?;
        }
        let stored = StoredPlayerProfile {
            secret_key: self.identity.secret_bytes(),
            games: StoredGameProfiles {
                qigui523: StoredRatingProfile {
                    reference_points: self.rating.reference_points,
                    completed_games: self.rating.completed_games,
                    applied_matches: self.rating.applied_matches.iter().copied().collect(),
                    qigui523_stats: self.game_profiles.qigui523.clone(),
                },
                texas_holdem_stats: self.game_profiles.texas_holdem.clone(),
                shengji_stats: self.game_profiles.shengji.clone(),
                uno_stats: self.game_profiles.uno.clone(),
                interaction_stats: self.game_profiles.interactions.clone(),
            },
        };
        let bytes =
            postcard::to_allocvec(&stored).map_err(|error| format!("玩家档案编码失败：{error}"))?;
        fs::write(&path, bytes).map_err(|error| format!("无法保存玩家档案：{error}"))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&path, fs::Permissions::from_mode(0o600))
                .map_err(|error| format!("无法保护玩家档案权限：{error}"))?;
        }
        Ok(())
    }

    pub(super) fn apply_finished_match(
        &mut self,
        match_id: MatchId,
        reference_changes: &[leocard_protocol::PlayerReferenceChange],
    ) -> bool {
        if self.rating.applied_matches.contains(&match_id) {
            return false;
        }
        let profile_id = self.identity.profile_id();
        let Some(change) = reference_changes
            .iter()
            .find(|change| change.profile_id == profile_id)
        else {
            return false;
        };
        self.rating.reference_points = self
            .rating
            .reference_points
            .saturating_add(i32::from(change.delta));
        self.rating.completed_games = self.rating.completed_games.saturating_add(1);
        self.rating.applied_matches.insert(match_id);
        self.rating.last_change = Some((match_id, change.delta));
        true
    }

    pub(super) fn reference_points(&self) -> i32 {
        self.rating.reference_points
    }

    pub(super) fn completed_games(&self) -> u32 {
        self.rating.completed_games
    }

    pub(super) const fn game_profiles(&self) -> &PlayerGameProfiles {
        &self.game_profiles
    }

    pub(super) fn sync_qigui523_profile(&mut self, stats: &QiGui523ProfileStats) -> bool {
        if self.game_profiles.qigui523.as_ref() == Some(stats) {
            return false;
        }
        self.game_profiles.qigui523 = Some(stats.clone());
        true
    }

    pub(super) fn sync_texas_holdem_profile(&mut self, stats: &TexasHoldemProfileStats) -> bool {
        if self.game_profiles.texas_holdem.as_ref() == Some(stats) {
            return false;
        }
        self.game_profiles.texas_holdem = Some(stats.clone());
        true
    }

    pub(super) fn sync_shengji_profile(&mut self, stats: &ShengjiProfileStats) -> bool {
        if self.game_profiles.shengji.as_ref() == Some(stats) {
            return false;
        }
        self.game_profiles.shengji = Some(stats.clone());
        true
    }

    pub(super) fn sync_uno_profile(&mut self, stats: &UnoProfileStats) -> bool {
        if self.game_profiles.uno.as_ref() == Some(stats) {
            return false;
        }
        self.game_profiles.uno = Some(stats.clone());
        true
    }

    pub(super) fn sync_interaction_profile(&mut self, stats: &PlayerInteractionStats) -> bool {
        if self.game_profiles.interactions.as_ref() == Some(stats) {
            return false;
        }
        self.game_profiles.interactions = Some(stats.clone());
        true
    }
}

pub(in crate::app) fn decode_player_profile(bytes: &[u8]) -> Result<StoredPlayerProfile, String> {
    match postcard::from_bytes(bytes) {
        Ok(stored) => Ok(stored),
        Err(current_error) => {
            if let Ok(previous) = postcard::from_bytes::<PreInteractionStoredPlayerProfile>(bytes) {
                return Ok(StoredPlayerProfile {
                    secret_key: previous.secret_key,
                    games: StoredGameProfiles {
                        qigui523: previous.games.qigui523,
                        texas_holdem_stats: previous.games.texas_holdem_stats,
                        shengji_stats: previous.games.shengji_stats,
                        uno_stats: previous.games.uno_stats,
                        interaction_stats: None,
                    },
                });
            }
            if let Ok(previous) = postcard::from_bytes::<PreUnoStoredPlayerProfile>(bytes) {
                return Ok(StoredPlayerProfile {
                    secret_key: previous.secret_key,
                    games: StoredGameProfiles {
                        qigui523: previous.games.qigui523,
                        texas_holdem_stats: previous.games.texas_holdem_stats,
                        shengji_stats: previous.games.shengji_stats,
                        uno_stats: None,
                        interaction_stats: None,
                    },
                });
            }
            if let Ok(previous) = postcard::from_bytes::<PreShengjiStoredPlayerProfile>(bytes) {
                return Ok(StoredPlayerProfile {
                    secret_key: previous.secret_key,
                    games: StoredGameProfiles {
                        qigui523: previous.games.qigui523,
                        texas_holdem_stats: previous.games.texas_holdem_stats,
                        shengji_stats: None,
                        uno_stats: None,
                        interaction_stats: None,
                    },
                });
            }
            if let Ok(previous) = postcard::from_bytes::<PreTexasStoredPlayerProfile>(bytes) {
                return Ok(StoredPlayerProfile {
                    secret_key: previous.secret_key,
                    games: StoredGameProfiles {
                        qigui523: previous.games.qigui523,
                        texas_holdem_stats: None,
                        shengji_stats: None,
                        uno_stats: None,
                        interaction_stats: None,
                    },
                });
            }
            let previous: PreDetailedStoredPlayerProfile = postcard::from_bytes(bytes)
                .map_err(|_| format!("玩家档案已损坏：{current_error}"))?;
            Ok(StoredPlayerProfile {
                secret_key: previous.secret_key,
                games: StoredGameProfiles {
                    qigui523: StoredRatingProfile {
                        reference_points: previous.games.qigui523.reference_points,
                        completed_games: previous.games.qigui523.completed_games,
                        applied_matches: previous.games.qigui523.applied_matches,
                        qigui523_stats: None,
                    },
                    texas_holdem_stats: None,
                    shengji_stats: None,
                    uno_stats: None,
                    interaction_stats: None,
                },
            })
        }
    }
}

pub(super) fn load_preferences() -> Option<SavedPreferences> {
    let bytes = fs::read(preferences_path()?).ok()?;
    decode_preferences(&bytes)
}

pub(super) fn decode_preferences(bytes: &[u8]) -> Option<SavedPreferences> {
    postcard::from_bytes(bytes)
        .ok()
        .or_else(|| {
            let previous: PreOmahaSavedPreferences = postcard::from_bytes(bytes).ok()?;
            Some(SavedPreferences {
                global: previous.global,
                games: GamePreferences {
                    qigui523: previous.games.qigui523,
                    texas_holdem: TexasHoldemPreferences {
                        host_rules: previous.games.texas_holdem.host_rules.into(),
                    },
                    shengji: previous.games.shengji,
                    uno: previous.games.uno,
                },
            })
        })
        .or_else(|| {
            let previous: PreJumpInSavedPreferences = postcard::from_bytes(bytes).ok()?;
            Some(SavedPreferences {
                global: previous.global,
                games: GamePreferences {
                    qigui523: previous.games.qigui523,
                    texas_holdem: TexasHoldemPreferences {
                        host_rules: previous.games.texas_holdem.host_rules.into(),
                    },
                    shengji: previous.games.shengji,
                    uno: UnoPreferences {
                        host_rules: previous.games.uno.host_rules.into(),
                    },
                },
            })
        })
        .or_else(|| {
            let previous: PreviousUnoSavedPreferences = postcard::from_bytes(bytes).ok()?;
            Some(SavedPreferences {
                global: previous.global,
                games: GamePreferences {
                    qigui523: previous.games.qigui523,
                    texas_holdem: TexasHoldemPreferences {
                        host_rules: previous.games.texas_holdem.host_rules.into(),
                    },
                    shengji: previous.games.shengji,
                    uno: UnoPreferences {
                        host_rules: previous.games.uno.host_rules.into(),
                    },
                },
            })
        })
        .or_else(|| {
            let previous: PreUnoSavedPreferences = postcard::from_bytes(bytes).ok()?;
            Some(SavedPreferences {
                global: previous.global,
                games: GamePreferences {
                    qigui523: previous.games.qigui523,
                    texas_holdem: TexasHoldemPreferences {
                        host_rules: previous.games.texas_holdem.host_rules.into(),
                    },
                    shengji: previous.games.shengji,
                    uno: UnoPreferences::default(),
                },
            })
        })
        .or_else(|| {
            let previous: PreviousSavedPreferences = postcard::from_bytes(bytes).ok()?;
            Some(SavedPreferences {
                global: previous.global,
                games: GamePreferences {
                    qigui523: previous.games.qigui523,
                    texas_holdem: TexasHoldemPreferences {
                        host_rules: previous.games.texas_holdem.host_rules.into(),
                    },
                    shengji: previous.games.shengji,
                    uno: UnoPreferences::default(),
                },
            })
        })
        .or_else(|| {
            let legacy: LegacySavedPreferences = postcard::from_bytes(bytes).ok()?;
            Some(SavedPreferences {
                global: legacy.global,
                games: GamePreferences {
                    qigui523: legacy.games.qigui523,
                    texas_holdem: TexasHoldemPreferences {
                        host_rules: legacy.games.texas_holdem.host_rules.into(),
                    },
                    shengji: ShengjiPreferences::default(),
                    uno: UnoPreferences::default(),
                },
            })
        })
}

pub(super) fn save_preferences(form: &ConnectionForm) -> Result<(), String> {
    let path = preferences_path().ok_or_else(|| "无法确定本机配置目录".to_owned())?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| format!("无法创建配置目录：{error}"))?;
    }
    let saved = SavedPreferences {
        global: GlobalPreferences {
            player_name: form.player_name.clone(),
            avatar_png: form.avatar_png.clone(),
            host_port: form.host_port.clone(),
            join_address: form.join_address.clone(),
            table_felt_path: form.table_felt_path.clone(),
            table_brightness: form.table_brightness,
            table_vignette: form.table_vignette,
            audio_volume: form.audio_volume,
        },
        games: GamePreferences {
            qigui523: QiGui523Preferences {
                host_rules: form.host_rules,
            },
            texas_holdem: TexasHoldemPreferences {
                host_rules: form.texas_holdem_rules,
            },
            shengji: ShengjiPreferences {
                host_rules: form.shengji_rules,
            },
            uno: UnoPreferences {
                host_rules: form.uno_rules,
            },
        },
    };
    let bytes = postcard::to_allocvec(&saved).map_err(|error| format!("配置编码失败：{error}"))?;
    fs::write(path, bytes).map_err(|error| format!("无法保存本地配置：{error}"))
}

pub(super) fn normalize_avatar(path: &Path) -> Result<Vec<u8>, String> {
    let source = fs::read(path).map_err(|error| format!("无法读取头像文件：{error}"))?;
    normalize_avatar_bytes(&source)
}

pub(super) fn normalize_avatar_bytes(source: &[u8]) -> Result<Vec<u8>, String> {
    // Input may be PNG or JPEG, but local persistence and the wire format remain
    // one bounded square PNG representation.
    let image = image::load_from_memory(source)
        .map_err(|_| "头像必须是有效的 PNG、JPG 或 JPEG 图片".to_owned())?;
    let image = image.resize_to_fill(
        AVATAR_DIMENSION,
        AVATAR_DIMENSION,
        image::imageops::FilterType::Lanczos3,
    );
    let mut output = Cursor::new(Vec::new());
    image
        .write_to(&mut output, image::ImageFormat::Png)
        .map_err(|error| format!("头像压缩失败：{error}"))?;
    let png = output.into_inner();
    if png.len() > MAX_AVATAR_BYTES {
        return Err(format!("压缩后的头像超过 {} KiB", MAX_AVATAR_BYTES / 1024));
    }
    if !valid_normalized_avatar(&png) {
        return Err("头像规范化结果无效".to_owned());
    }
    Ok(png)
}

pub(super) fn valid_normalized_avatar(png: &[u8]) -> bool {
    png.len() <= MAX_AVATAR_BYTES
        && image::load_from_memory_with_format(png, image::ImageFormat::Png).is_ok_and(|image| {
            image.width() == AVATAR_DIMENSION && image.height() == AVATAR_DIMENSION
        })
}

pub(super) fn image_handle_from_png(
    png: &[u8],
    images: &mut Assets<Image>,
) -> Option<Handle<Image>> {
    let dynamic = image::load_from_memory_with_format(png, image::ImageFormat::Png).ok()?;
    Some(images.add(Image::from_dynamic(
        dynamic,
        true,
        RenderAssetUsages::default(),
    )))
}

pub(super) fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.spawn((
        PlayerInteractionLayer,
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            right: px(0),
            top: px(0),
            bottom: px(0),
            ..default()
        },
        GlobalZIndex(1100),
        FocusPolicy::Pass,
    ));
}

pub(super) fn load_ui_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
) {
    let mut cards = HashMap::new();
    for card in build_deck(1) {
        cards.entry((card.rank(), card.suit())).or_insert_with(|| {
            asset_server.load::<Image>(card_asset_path(card.rank(), card.suit()))
        });
    }
    let mut uno_cards = HashMap::new();
    for card in build_uno_deck() {
        uno_cards
            .entry((card.color(), card.face()))
            .or_insert_with(|| asset_server.load::<Image>(uno_card_asset_path(card)));
    }
    let mut interaction_images = HashMap::new();
    let mut interaction_sounds = HashMap::new();
    for (kind, name) in [
        (PlayerInteractionKind::Flower, "flower"),
        (PlayerInteractionKind::Egg, "egg"),
        (PlayerInteractionKind::Wine, "wine"),
        (PlayerInteractionKind::Shoe, "shoe"),
    ] {
        for state in 1u8..=2 {
            interaction_images.insert(
                (kind, state == 2),
                asset_server.load(format!("vendor/noname/interactions/{name}{state}.png")),
            );
            interaction_sounds.insert(
                (kind, state - 1),
                asset_server.load(format!(
                    "vendor/noname/interactions/throw_{name}{state}.mp3"
                )),
            );
        }
    }
    let interaction_cooldown_masks = (0..=INTERACTION_COOLDOWN_MASK_FRAMES)
        .map(|frame| {
            let fraction = frame as f32 / INTERACTION_COOLDOWN_MASK_FRAMES as f32;
            images.add(interaction_cooldown_mask_image(72, 62, fraction))
        })
        .collect();
    let deal_sounds = (1..=8)
        .map(|index| {
            asset_server.load(format!(
                "vendor/kenney/casino-audio/Audio/card-slide-{index}.ogg"
            ))
        })
        .collect();
    let place_sounds = (1..=4)
        .map(|index| {
            asset_server.load(format!(
                "vendor/kenney/casino-audio/Audio/card-place-{index}.ogg"
            ))
        })
        .collect();
    let shove_sounds = vec![asset_server.load("vendor/noname/audio/effect/flappybird_start.ogg")];
    let button_click_sounds = ["click-a.ogg", "click-b.ogg"]
        .map(|name| asset_server.load(format!("vendor/kenney/ui/Sounds/{name}")))
        .to_vec();
    let quick_voice_sounds = (0..QUICK_VOICE_COUNT)
        .map(|index| asset_server.load(format!("vendor/noname/voice/male/{index}.mp3")))
        .collect();

    commands.insert_resource(ShengjiSoundAssets::load(&asset_server));
    commands.insert_resource(UiAssets {
        font: asset_server.load(UI_FONT_ASSET),
        cards,
        card_back: asset_server.load("vendor/kenney/boardgame/PNG/Cards/cardBack_blue4.png"),
        uno_cards,
        uno_card_back: asset_server.load("cards/uno/card_back.png"),
        table_felt: asset_server.load(TABLE_FELT_ASSET),
        primary_button: asset_server
            .load("vendor/kenney/ui/PNG/Green/Default/button_rectangle_depth_gradient.png"),
        secondary_button: asset_server
            .load("vendor/kenney/ui/PNG/Blue/Default/button_rectangle_depth_gradient.png"),
        warning_button: asset_server
            .load("vendor/kenney/ui/PNG/Yellow/Default/button_rectangle_depth_gradient.png"),
        danger_button: asset_server
            .load("vendor/kenney/ui/PNG/Red/Default/button_rectangle_depth_gradient.png"),
        disabled_button: asset_server
            .load("vendor/kenney/ui/PNG/Grey/Default/button_rectangle_depth_gradient.png"),
        panel_window: asset_server.load("ui/panel_window.png"),
        panel_section: asset_server.load("ui/panel_section.png"),
        panel_popup: asset_server.load("ui/panel_popup.png"),
        player_panel_wide: asset_server.load("ui/player_panel_wide.png"),
        player_panel_compact: asset_server.load("ui/player_panel_compact.png"),
        poker_chips: HashMap::from([
            (
                1,
                asset_server.load("vendor/kenney/boardgame/PNG/Chips/chipWhiteBlue.png"),
            ),
            (
                5,
                asset_server.load("vendor/kenney/boardgame/PNG/Chips/chipRedWhite.png"),
            ),
            (
                10,
                asset_server.load("vendor/kenney/boardgame/PNG/Chips/chipBlueWhite.png"),
            ),
            (
                25,
                asset_server.load("vendor/kenney/boardgame/PNG/Chips/chipGreenWhite.png"),
            ),
            (
                100,
                asset_server.load("vendor/kenney/boardgame/PNG/Chips/chipBlackWhite.png"),
            ),
        ]),
        interaction_images,
        interaction_sounds,
        interaction_cooldown_masks,
        deal_sounds,
        place_sounds,
        shove_sounds,
        button_click_sounds,
        error_popup_sound: asset_server.load("vendor/kenney/interface-sounds/Audio/error_007.ogg"),
        bomb_explosion_sound: asset_server.load("vendor/noname/damage_fire2.mp3"),
        summary_score_sound: asset_server.load("vendor/noname/audio/effect/flappybird_score.ogg"),
        summary_die_sound: asset_server.load("vendor/noname/audio/effect/flappybird_die.ogg"),
        quick_voice_sounds,
        chat_emojis: CHAT_EMOJI_ASSET_PATHS
            .iter()
            .map(|path| asset_server.load(*path))
            .collect(),
        chat_emoji_icon: asset_server.load("icons/chat-emoji-white.png"),
        texas_sounds: TexasSoundAssets::load(&asset_server),
        uno_sounds: UnoSoundAssets::load(&asset_server),
        sequence_airplane: asset_server.load("ui/effects/sequence_airplane.png"),
        shengji_target: asset_server.load("ui/effects/shengji_target.png"),
        shengji_dart: asset_server.load("ui/effects/shengji_dart.png"),
        chat_open_icon: asset_server.load("vendor/kenney/ui/PNG/Blue/Default/arrow_basic_w.png"),
        chat_close_icon: asset_server.load("vendor/kenney/ui/PNG/Blue/Default/arrow_basic_e.png"),
        quick_voice_icon: asset_server.load("icons/list-menu.png"),
        robot_icon: asset_server.load("icons/robot-2-fill.png"),
        host_crown: asset_server.load("icons/host-crown.png"),
        github_mark: asset_server.load("icons/github-mark.png"),
    });
}

fn uno_card_asset_path(card: UnoCard) -> String {
    let file = match card.color() {
        None => match card.face() {
            UnoFace::Wild => "wild".to_owned(),
            UnoFace::WildDrawFour => "wild_draw_four".to_owned(),
            _ => unreachable!("无颜色的 UNO 牌必须是万能牌"),
        },
        Some(color) => {
            let color = match color {
                UnoColor::Red => "red",
                UnoColor::Yellow => "yellow",
                UnoColor::Green => "green",
                UnoColor::Blue => "blue",
            };
            let face = match card.face() {
                UnoFace::Number(value) => value.to_string(),
                UnoFace::DrawTwo => "draw_two".to_owned(),
                UnoFace::Reverse => "reverse".to_owned(),
                UnoFace::Skip => "skip".to_owned(),
                UnoFace::Wild | UnoFace::WildDrawFour => unreachable!("万能牌没有颜色"),
            };
            format!("{color}_{face}")
        }
    };
    format!("cards/uno/{file}.png")
}

pub(super) fn interaction_cooldown_mask_image(width: u32, height: u32, fraction: f32) -> Image {
    let mut pixels = image::RgbaImage::new(width, height);
    let center = Vec2::new(width as f32 * 0.5, height as f32 * 0.5);
    let sweep = fraction.clamp(0.0, 1.0) * std::f32::consts::TAU;
    for (x, y, pixel) in pixels.enumerate_pixels_mut() {
        let offset = Vec2::new(x as f32 + 0.5, y as f32 + 0.5) - center;
        let angle = offset.x.atan2(-offset.y).rem_euclid(std::f32::consts::TAU);
        if fraction >= 1.0 || (fraction > 0.0 && angle <= sweep) {
            *pixel = image::Rgba([0, 0, 0, 158]);
        }
    }
    Image::from_dynamic(
        image::DynamicImage::ImageRgba8(pixels),
        true,
        RenderAssetUsages::default(),
    )
}
