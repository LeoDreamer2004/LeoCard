//! 麻将房间、牌桌、动作按钮和结算视图。

use super::tiles::queue_mahjong_deal_sound;
use super::{
    MahjongAssets, MahjongOwnHandVisuals, MahjongPlayerPanelVisuals, MahjongPlayerTileVisuals,
    MahjongTileMaterial, MahjongUiState, MahjongWinVisuals, MahjongWinningHandVisual,
    mahjong_major_fan_impact_times, render_action_bar, render_discard_rivers,
    render_mahjong_claim_presentation, render_mahjong_flower_presentations,
    render_mahjong_player_panel, render_mahjong_player_tiles, render_mahjong_settlement,
    render_mahjong_wall, render_mahjong_win_effects, render_own_hand, render_round_status,
};
use crate::app::presentation::{
    DESIGN_WIDTH, GameSummaryAnimation, Observed, TableBackground, TableBackgroundMaterial,
    spawn_node, table_material_params,
};
use crate::app::runtime::{AvatarImages, ClientResource, TableAppearance, UiAssets};
#[cfg(feature = "developer")]
use crate::app::shell::add_developer_hand_input;
use crate::app::shell::{ChatPanelState, DeveloperHandInput, UiState, add_chat_panel};
use bevy::prelude::*;
use leocard_mahjong::{
    MahjongClaim, MahjongDragon, MahjongFlower, MahjongSuit, MahjongTile, MahjongTileKind,
    MahjongWind,
};
use leocard_protocol::{MahjongEvent, MahjongPhaseView, MahjongSnapshot, MatchId, PlayerId};
use leocard_protocol::{MahjongHandResultView, MahjongWinView};
use std::collections::VecDeque;

pub(super) const MAHJONG_CLAIM_FLIGHT_DELAY: f32 = 0.18;
pub(super) const MAHJONG_CLAIM_FLIGHT_DURATION: f32 = 0.52;
pub(super) const MAHJONG_CLAIM_HAND_SHIFT_DURATION: f32 = 0.34;
pub(super) const MAHJONG_CLAIM_PRESENTATION_DURATION: f32 = 1.42;
pub(super) const MAHJONG_FLOWER_PRESENTATION_DURATION: f32 = 1.18;
pub(super) const MAHJONG_OWN_HAND_LEFT: f32 = 315.0;
pub(super) const MAHJONG_OWN_MELD_WIDTH: f32 = 140.0;
pub(super) const MAHJONG_REMOTE_MELD_WIDTH: f32 = 78.0;

pub(super) const fn mahjong_claim_landing_time() -> f32 {
    MAHJONG_CLAIM_FLIGHT_DELAY + MAHJONG_CLAIM_FLIGHT_DURATION
}

#[derive(Component)]
pub(crate) struct MahjongHandTile {
    pub lift: f32,
    pub base_rotation: f32,
    pub index: i32,
}

#[derive(Component)]
pub(super) struct MahjongDealTile {
    pub elapsed: f32,
    pub start_offset: Vec2,
    pub start_rotation: f32,
    pub final_offset: Vec2,
    pub final_rotation: f32,
    pub final_shadow_alpha: f32,
}

#[derive(Component)]
pub(super) struct MahjongTurnArrow {
    pub slot: f32,
}

#[derive(Component)]
pub(crate) struct MahjongWinningHand {
    pub relative: u8,
    pub base_rotation: f32,
    pub reveal_duration: f32,
    pub start: f32,
}

#[derive(Clone, Debug)]
pub(crate) struct ActiveMahjongClaimPresentation {
    pub match_id: MatchId,
    pub player: PlayerId,
    pub source: Option<PlayerId>,
    pub tile: Option<MahjongTile>,
    pub claim: MahjongClaim,
    pub shift_hand: bool,
    pub elapsed: f32,
}

#[derive(Clone, Debug)]
pub(crate) struct ActiveMahjongFlowerPresentation {
    pub match_id: MatchId,
    pub player: PlayerId,
    pub elapsed: f32,
}

#[derive(Resource, Default)]
pub(crate) struct MahjongClaimPresentationState {
    pub active: Option<ActiveMahjongClaimPresentation>,
    pub queued: VecDeque<ActiveMahjongClaimPresentation>,
    pub flowers: Vec<ActiveMahjongFlowerPresentation>,
    pub observed_match: Observed<MatchId, ()>,
}

#[derive(Component)]
pub(crate) struct MahjongClaimFlight {
    pub player: PlayerId,
    pub tile: MahjongTile,
    pub start: Vec2,
    pub control: Vec2,
    pub target: Vec2,
    pub start_angle: f32,
    pub target_angle: f32,
    pub target_scale: f32,
}

#[derive(Component)]
pub(crate) struct MahjongClaimLabel {
    pub player: PlayerId,
    pub text: Entity,
}

#[derive(Component)]
pub(crate) struct MahjongFlowerLabel {
    pub player: PlayerId,
    pub text: Entity,
}

#[derive(Component)]
pub(crate) struct MahjongClaimHandShift {
    pub player: PlayerId,
    pub distance: f32,
}

#[derive(Component)]
pub(crate) struct MahjongClaimHeldTile {
    pub player: PlayerId,
}

#[derive(Component)]
pub(crate) struct MahjongWinEffect {
    pub tier: MahjongWinEffectTier,
    pub reveal_duration: f32,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum MahjongWinEffectTier {
    Normal,
    HighTotal,
    MajorFan,
}

impl MahjongWinEffectTier {
    fn from_score(total_points: u16, has_major_fan: bool) -> Self {
        if has_major_fan {
            Self::MajorFan
        } else if total_points > 20 {
            Self::HighTotal
        } else {
            Self::Normal
        }
    }

    pub(crate) const fn duration(self) -> f32 {
        match self {
            Self::Normal => 0.86,
            Self::HighTotal => 0.95,
            Self::MajorFan => 1.0,
        }
    }

    pub(super) const fn presentation_duration(self) -> f32 {
        match self {
            Self::Normal => 0.90,
            Self::HighTotal => 2.45,
            Self::MajorFan => 4.80,
        }
    }
}

pub(super) fn mahjong_win_effect_tier(winner: &MahjongWinView) -> MahjongWinEffectTier {
    MahjongWinEffectTier::from_score(
        winner.score.total_points,
        winner.score.fans.iter().any(|fan| fan.fan.points() >= 48),
    )
}

pub(super) fn mahjong_win_reveal_duration(result: &MahjongHandResultView) -> f32 {
    result.winners.iter().fold(1.0, |duration, winner| {
        duration + mahjong_win_effect_tier(winner).presentation_duration()
    })
}

pub(super) fn mahjong_win_stage_start(result: &MahjongHandResultView, winner_index: usize) -> f32 {
    1.0 + result.winners[..winner_index]
        .iter()
        .map(mahjong_win_effect_tier)
        .map(MahjongWinEffectTier::presentation_duration)
        .sum::<f32>()
}

fn mahjong_win_hand_start(result: &MahjongHandResultView, winner_index: usize) -> f32 {
    mahjong_win_stage_start(result, winner_index)
        + if mahjong_win_effect_tier(&result.winners[winner_index])
            == MahjongWinEffectTier::MajorFan
        {
            0.78
        } else {
            0.10
        }
}

#[derive(Component)]
pub(crate) struct MahjongWinEffectText {
    pub tier: MahjongWinEffectTier,
    pub reveal_duration: f32,
}

#[derive(Clone, Copy)]
pub(crate) enum MahjongWinDecorationKind {
    Halo,
    Ring {
        delay: f32,
        start_scale: f32,
        end_scale: f32,
        max_alpha: f32,
    },
    Ray {
        direction: Vec2,
        distance: f32,
        delay: f32,
        secondary: bool,
    },
}

#[derive(Component)]
pub(crate) struct MahjongWinDecoration {
    pub tier: MahjongWinEffectTier,
    pub reveal_duration: f32,
    pub kind: MahjongWinDecorationKind,
}

#[derive(Clone, Copy)]
pub(crate) enum MahjongWinStageKind {
    Backdrop,
    Hand,
    WinningTile,
    FocusRay {
        delay: f32,
        direction: Vec2,
        phase: f32,
    },
    MajorFrame,
    MajorSweep {
        delay: f32,
    },
    MajorSpark {
        delay: f32,
        drift: Vec2,
        phase: f32,
    },
    ImpactFlash {
        delay: f32,
    },
}

#[derive(Component)]
pub(crate) struct MahjongWinStagePart {
    pub tier: MahjongWinEffectTier,
    pub reveal_duration: f32,
    pub start: f32,
    pub duration: f32,
    pub kind: MahjongWinStageKind,
}

#[derive(Component)]
pub(crate) struct MahjongWinFanGlyph {
    pub reveal_duration: f32,
    pub start: f32,
    pub delay: f32,
}

#[derive(Component)]
pub(crate) struct MahjongWinScreenShake {
    pub reveal_duration: f32,
    pub impacts: Vec<f32>,
}

#[derive(Clone, Copy)]
pub(crate) struct MahjongDealSpec {
    pub start_offset: Vec2,
    pub start_rotation: f32,
}

pub(crate) struct MahjongTableVisuals<'a> {
    pub assets: &'a UiAssets,
    pub game_assets: &'a MahjongAssets,
    pub avatars: &'a AvatarImages,
    pub developer_hand: &'a DeveloperHandInput,
    pub appearance: &'a TableAppearance,
    pub brightness: f32,
    pub vignette: f32,
    pub table_materials: &'a mut Assets<TableBackgroundMaterial>,
    pub tile_materials: &'a mut Assets<MahjongTileMaterial>,
    pub game_summary: &'a GameSummaryAnimation,
    pub claim_presentation: &'a MahjongClaimPresentationState,
}

pub(super) fn sync_mahjong_claim_presentation(
    mut client: Option<ResMut<ClientResource>>,
    mut presentation: ResMut<MahjongClaimPresentationState>,
) {
    let Some(client) = client.as_deref_mut() else {
        *presentation = MahjongClaimPresentationState::default();
        return;
    };
    let events = client.0.model_mut().take_mahjong_events();
    let Some(match_id) = client.0.model().mahjong_game().map(|game| game.match_id) else {
        *presentation = MahjongClaimPresentationState::default();
        return;
    };
    if presentation.observed_match.observe(match_id) {
        presentation.active = None;
        presentation.queued.clear();
        presentation.flowers.clear();
    }
    for event in events {
        if let MahjongEvent::FlowerReplaced { player } = &event {
            if let Some(active) = presentation
                .flowers
                .iter_mut()
                .find(|active| active.player == *player)
            {
                active.elapsed = 0.0;
            } else {
                presentation.flowers.push(ActiveMahjongFlowerPresentation {
                    match_id,
                    player: *player,
                    elapsed: 0.0,
                });
            }
            continue;
        }
        let active = match event {
            MahjongEvent::ClaimResolved {
                player,
                source,
                tile,
                claim,
            } if matches!(
                claim,
                MahjongClaim::Chow { .. } | MahjongClaim::Pung | MahjongClaim::Kong
            ) =>
            {
                ActiveMahjongClaimPresentation {
                    match_id,
                    player,
                    source: Some(source),
                    tile: Some(tile),
                    claim,
                    shift_hand: true,
                    elapsed: 0.0,
                }
            }
            MahjongEvent::KongDeclared { player, added, .. } => ActiveMahjongClaimPresentation {
                match_id,
                player,
                source: None,
                tile: None,
                claim: MahjongClaim::Kong,
                shift_hand: !added,
                elapsed: 0.0,
            },
            _ => continue,
        };
        if presentation.active.is_none() {
            presentation.active = Some(active);
        } else {
            presentation.queued.push_back(active);
        }
    }
}

pub(super) fn advance_mahjong_claim_presentation(
    time: Res<Time>,
    mut presentation: ResMut<MahjongClaimPresentationState>,
    mut ui: ResMut<UiState>,
) {
    for flower in &mut presentation.flowers {
        flower.elapsed += time.delta_secs();
    }
    let flower_count = presentation.flowers.len();
    presentation
        .flowers
        .retain(|flower| flower.elapsed < MAHJONG_FLOWER_PRESENTATION_DURATION);
    if presentation.flowers.len() != flower_count {
        ui.dirty = true;
    }
    let Some(active) = presentation.active.as_mut() else {
        return;
    };
    let previous = active.elapsed;
    active.elapsed += time.delta_secs();
    if active.source.is_some()
        && previous < mahjong_claim_landing_time()
        && active.elapsed >= mahjong_claim_landing_time()
    {
        ui.dirty = true;
    }
    if active.elapsed >= MAHJONG_CLAIM_PRESENTATION_DURATION {
        presentation.active = presentation.queued.pop_front();
        ui.dirty = true;
    }
}

pub(crate) fn mahjong_tile_asset_path(kind: MahjongTileKind) -> String {
    let name = match kind {
        MahjongTileKind::Suited {
            suit: MahjongSuit::Characters,
            rank,
        } => format!("characters-{rank}"),
        MahjongTileKind::Suited {
            suit: MahjongSuit::Bamboo,
            rank,
        } => format!("bamboo-{rank}"),
        MahjongTileKind::Suited {
            suit: MahjongSuit::Dots,
            rank,
        } => format!("dots-{rank}"),
        MahjongTileKind::Wind(MahjongWind::East) => "wind-east".to_owned(),
        MahjongTileKind::Wind(MahjongWind::South) => "wind-south".to_owned(),
        MahjongTileKind::Wind(MahjongWind::West) => "wind-west".to_owned(),
        MahjongTileKind::Wind(MahjongWind::North) => "wind-north".to_owned(),
        MahjongTileKind::Dragon(MahjongDragon::Red) => "dragon-red".to_owned(),
        MahjongTileKind::Dragon(MahjongDragon::Green) => "dragon-green".to_owned(),
        MahjongTileKind::Dragon(MahjongDragon::White) => "dragon-white".to_owned(),
        MahjongTileKind::Flower(MahjongFlower::Spring) => "season-spring".to_owned(),
        MahjongTileKind::Flower(MahjongFlower::Summer) => "season-summer".to_owned(),
        MahjongTileKind::Flower(MahjongFlower::Autumn) => "season-autumn".to_owned(),
        MahjongTileKind::Flower(MahjongFlower::Winter) => "season-winter".to_owned(),
        MahjongTileKind::Flower(MahjongFlower::Plum) => "flower-plum".to_owned(),
        MahjongTileKind::Flower(MahjongFlower::Orchid) => "flower-orchid".to_owned(),
        MahjongTileKind::Flower(MahjongFlower::Bamboo) => "flower-bamboo".to_owned(),
        MahjongTileKind::Flower(MahjongFlower::Chrysanthemum) => "flower-chrysanthemum".to_owned(),
    };
    format!("cards/mahjong/hong-kong/{name}.png")
}

pub(crate) fn mahjong_tile_height_asset_path(kind: MahjongTileKind) -> String {
    mahjong_tile_asset_path(kind).replace("/hong-kong/", "/hong-kong-height/")
}

#[expect(
    clippy::too_many_arguments,
    reason = "the game-screen adapter passes common screen state plus grouped visuals"
)]
pub(crate) fn render_mahjong_table(
    commands: &mut Commands,
    root: Entity,
    client: &ClientResource,
    game: &MahjongSnapshot,
    ui: &mut MahjongUiState,
    chat: &ChatPanelState,
    interaction_menu_open: Option<PlayerId>,
    visuals: MahjongTableVisuals<'_>,
) {
    let MahjongTableVisuals {
        assets,
        game_assets,
        avatars,
        developer_hand,
        appearance,
        brightness,
        vignette,
        table_materials,
        tile_materials,
        game_summary,
        claim_presentation,
    } = visuals;
    let content = spawn_node(
        commands,
        root,
        Node {
            width: percent(100),
            flex_grow: 1.0,
            min_height: px(0),
            position_type: PositionType::Relative,
            ..default()
        },
        None,
    );
    let felt = appearance
        .custom_felt
        .as_ref()
        .unwrap_or(&assets.table_felt)
        .clone();
    let material = table_materials.add(TableBackgroundMaterial {
        params: table_material_params(brightness, vignette, appearance.custom_felt.is_none()),
        texture: felt,
    });
    commands
        .entity(content)
        .insert((MaterialNode(material), TableBackground));
    let table = spawn_node(
        commands,
        content,
        Node {
            position_type: PositionType::Absolute,
            left: percent(50),
            top: px(0),
            bottom: px(0),
            width: px(DESIGN_WIDTH),
            min_height: px(430),
            ..default()
        },
        None,
    );
    commands.entity(table).insert(UiTransform {
        translation: Val2::px(-DESIGN_WIDTH / 2.0, 0.0),
        ..default()
    });

    let own_seat = game
        .players
        .iter()
        .find(|player| player.id == game.you)
        .map(|player| player.seat.0)
        .unwrap_or_default();
    ui.observed_table
        .observe((game.match_id, game.sequence_index));
    let received_batch = game.players.iter().any(|player| {
        let index = player.id.0 as usize;
        player.concealed_count > ui.observed_table.state.counts[index]
            || player.flowers.len() > usize::from(ui.observed_table.state.flowers[index])
    });
    let dealing = matches!(
        game.phase,
        MahjongPhaseView::Dealing { .. } | MahjongPhaseView::ReplacingFlower { .. }
    );
    let finished_result = match &game.phase {
        MahjongPhaseView::Finished { result } => Some(result),
        _ => None,
    };
    let win_reveal_duration = finished_result.map_or(0.0, mahjong_win_reveal_duration);
    if let Some(result) = finished_result {
        let impacts = mahjong_major_fan_impact_times(result);
        if !impacts.is_empty() {
            commands.entity(table).insert(MahjongWinScreenShake {
                reveal_duration: win_reveal_duration,
                impacts,
            });
        }
    }
    let all_hands_start = finished_result.and_then(|result| {
        result
            .winners
            .iter()
            .position(|winner| mahjong_win_effect_tier(winner) == MahjongWinEffectTier::MajorFan)
            .map(|index| mahjong_win_stage_start(result, index) + 0.78)
    });
    render_mahjong_wall(commands, table, game.wall_len, game_assets, tile_materials);
    render_discard_rivers(commands, table, game, own_seat, game_assets, tile_materials);
    for player in &game.players {
        let winner_start = finished_result.and_then(|result| {
            result
                .winners
                .iter()
                .position(|winner| winner.player == player.id)
                .map(|index| mahjong_win_hand_start(result, index))
        });
        let winning_hand_start = match (winner_start, all_hands_start) {
            (Some(winner), Some(all)) => Some(winner.min(all)),
            (winner, all) => winner.or(all),
        };
        let winning_hand = winning_hand_start.is_some();
        let flower_replaced = player.flowers.len()
            > usize::from(ui.observed_table.state.flowers[player.id.0 as usize]);
        let separate_last_concealed = matches!(
            game.phase,
            MahjongPhaseView::Playing | MahjongPhaseView::ReplacingFlower { .. }
        ) && game.current_player == player.id
            && player.concealed_count % 3 == 2;
        let active_claim = claim_presentation
            .active
            .as_ref()
            .filter(|claim| claim.match_id == game.match_id && claim.player == player.id);
        render_mahjong_player_tiles(
            commands,
            table,
            player,
            MahjongPlayerTileVisuals {
                own_seat,
                observed_count: ui.observed_table.state.counts[player.id.0 as usize],
                flower_replaced,
                dealing: dealing || flower_replaced,
                winning_hand: winning_hand.then_some(MahjongWinningHandVisual {
                    start: winning_hand_start.unwrap_or_default(),
                    reveal_duration: win_reveal_duration,
                }),
                separate_last_concealed,
                animation: game_summary,
                active_claim,
                game_assets,
                materials: tile_materials,
            },
        );
        render_mahjong_player_panel(
            commands,
            table,
            game,
            player,
            MahjongPlayerPanelVisuals {
                own_seat,
                interaction_menu_open,
                assets,
                avatars,
            },
        );
    }
    render_round_status(commands, table, game, own_seat, assets, game_assets);
    let own_flower_replaced = game
        .players
        .iter()
        .find(|player| player.id == game.you)
        .is_some_and(|player| {
            player.flowers.len()
                > usize::from(ui.observed_table.state.flowers[player.id.0 as usize])
        });
    let own_winning_hand = finished_result
        .is_some_and(|result| {
            result
                .winners
                .iter()
                .any(|winner| winner.player == game.you)
                || all_hands_start.is_some()
        })
        .then(|| MahjongWinningHandVisual {
            start: finished_result
                .and_then(|result| {
                    result
                        .winners
                        .iter()
                        .position(|winner| winner.player == game.you)
                        .map(|index| mahjong_win_hand_start(result, index))
                })
                .into_iter()
                .chain(all_hands_start)
                .min_by(f32::total_cmp)
                .unwrap_or_default(),
            reveal_duration: win_reveal_duration,
        });
    render_own_hand(
        commands,
        table,
        game,
        MahjongOwnHandVisuals {
            observed_hand: &ui.observed_table.state.hand,
            dealing: dealing || own_flower_replaced,
            winning_hand: own_winning_hand,
            animation: game_summary,
            active_claim: claim_presentation.active.as_ref().filter(|claim| {
                claim.match_id == game.match_id && claim.player == game.you && claim.shift_hand
            }),
            assets: game_assets,
            materials: tile_materials,
        },
    );
    #[cfg(feature = "developer")]
    if matches!(game.phase, MahjongPhaseView::Playing) {
        add_developer_hand_input(
            commands,
            table,
            developer_hand,
            "编辑手牌，如 123M456P789S1234Z",
            Vec2::new(8.0, 66.0),
            assets,
        );
    }
    #[cfg(not(feature = "developer"))]
    let _ = developer_hand;
    if received_batch {
        queue_mahjong_deal_sound(commands, assets);
    }
    render_action_bar(commands, table, game, assets);
    render_mahjong_claim_presentation(
        commands,
        table,
        game,
        own_seat,
        claim_presentation,
        assets,
        game_assets,
        tile_materials,
    );
    render_mahjong_flower_presentations(
        commands,
        table,
        game,
        own_seat,
        claim_presentation,
        assets,
    );
    render_mahjong_win_effects(
        commands,
        table,
        game,
        own_seat,
        game_summary,
        MahjongWinVisuals {
            assets,
            game_assets,
            materials: tile_materials,
        },
    );
    add_chat_panel(commands, content, chat, assets, None, &[]);
    if let MahjongPhaseView::Finished { result } = &game.phase {
        render_mahjong_settlement(commands, table, game, result, assets, avatars, game_summary);
    }
    ui.observed_table.state.hand.clone_from(&game.your_hand);
    for player in &game.players {
        ui.observed_table.state.counts[player.id.0 as usize] = player.concealed_count;
        ui.observed_table.state.flowers[player.id.0 as usize] = player.flowers.len() as u8;
    }
    let _ = client;
}
