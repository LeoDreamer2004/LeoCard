use super::{
    MahjongAssets, MahjongProjectionFrame, MahjongTurnSector, central_screen_offset, wind_label,
};
use crate::app::presentation::{TEXT, add_text};
use crate::app::runtime::UiAssets;
use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::ShaderRef;
use leocard_mahjong::MahjongWind;
use leocard_protocol::{MahjongPhaseView, MahjongSnapshot};
use leocard_ui::{UiComposer, spawn_node};
use std::f32::consts;

const REFERENCE_SCALE: f32 = 1280.0 / 2204.0;
const STATUS_LEFT: f32 = 900.0 * REFERENCE_SCALE;
const STATUS_TOP: f32 = 349.5 * REFERENCE_SCALE;
const STATUS_WIDTH: f32 = 390.0 * REFERENCE_SCALE;
const STATUS_HEIGHT: f32 = 254.5 * REFERENCE_SCALE;
const TOP_RATIO: f32 = 352.0 / 390.0;

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone, PartialEq)]
pub(super) struct MahjongTurnLightMaterial {
    /// x: bottom/right/top/left; y: breathing intensity.
    #[uniform(0)]
    pub(super) params: Vec4,
    #[uniform(0)]
    color: LinearRgba,
    #[texture(1)]
    #[sampler(2)]
    texture: Handle<Image>,
}

impl UiMaterial for MahjongTurnLightMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/mahjong_turn_light.wesl".into()
    }
}

impl MahjongTurnLightMaterial {
    pub(super) fn new(relative: u8, texture: Handle<Image>) -> Self {
        Self {
            params: Vec4::new(f32::from(relative), 0.7, 0.0, 0.0),
            color: Color::srgb(1.0, 0.55, 0.12).to_linear(),
            texture,
        }
    }
}

pub(super) fn render_round_status(
    commands: &mut UiComposer,
    table: Entity,
    game: &MahjongSnapshot,
    own_seat: u8,
    assets: &UiAssets,
    game_assets: &MahjongAssets,
) {
    render_round_status_with_scores(
        commands,
        table,
        game,
        own_seat,
        &game.match_scores,
        assets,
        game_assets,
    );
}

pub(super) fn render_round_status_with_scores(
    commands: &mut UiComposer,
    table: Entity,
    game: &MahjongSnapshot,
    own_seat: u8,
    scores: &[i32; 4],
    assets: &UiAssets,
    game_assets: &MahjongAssets,
) -> Vec<Entity> {
    let origin = status_origin();
    let size = Vec2::new(STATUS_WIDTH, STATUS_HEIGHT);
    let background_origin = origin - size * Vec2::new(0.06, 0.08);
    let background_size = size * Vec2::new(1.12, 1.28);
    let background = spawn_node(
        commands,
        table,
        Node {
            position_type: PositionType::Absolute,
            left: px(background_origin.x),
            top: px(background_origin.y),
            width: px(background_size.x),
            height: px(background_size.y),
            ..default()
        },
        None,
    );
    commands.entity(background).insert((
        MaterialNode(game_assets.status.clone()),
        MahjongProjectionFrame {
            anchor_bottom: false,
            center: background_origin + background_size * 0.5,
        },
        UiTransform::default(),
        Pickable::IGNORE,
        ZIndex(3),
    ));
    let status = spawn_node(
        commands,
        table,
        Node {
            position_type: PositionType::Absolute,
            left: px(origin.x),
            top: px(origin.y),
            width: px(STATUS_WIDTH),
            height: px(STATUS_HEIGHT),
            ..default()
        },
        None,
    );
    commands.entity(status).insert((
        MahjongProjectionFrame {
            anchor_bottom: false,
            center: Vec2::new(
                origin.x + STATUS_WIDTH * 0.5,
                origin.y + STATUS_HEIGHT * 0.5,
            ),
        },
        UiTransform::default(),
        Pickable::IGNORE,
        ZIndex(4),
    ));
    let current_relative = matches!(game.phase, MahjongPhaseView::Playing).then(|| {
        let current_seat = game
            .players
            .iter()
            .find(|player| player.player.id == game.current_player)
            .map_or(own_seat, |player| player.seat.0);
        (current_seat + 4 - own_seat) % 4
    });
    if let Some(relative) = current_relative {
        // The phase comes from global elapsed time, so rebuilding the UI cannot restart the pulse.
        let light = commands.child(
            status,
            (
                Node {
                    position_type: PositionType::Absolute,
                    left: px(0),
                    right: px(0),
                    top: px(0),
                    bottom: px(0),
                    ..default()
                },
                MahjongTurnSector,
                Pickable::IGNORE,
            ),
        );
        commands.entity(light).insert(MaterialNode(
            game_assets.status_lights[usize::from(relative)].clone(),
        ));
    }
    let center = spawn_node(
        commands,
        status,
        Node {
            position_type: PositionType::Absolute,
            left: percent(28),
            top: percent(29),
            width: percent(44),
            height: percent(42),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        None,
    );
    commands.entity(center).insert((
        Pickable::IGNORE,
        status_text_transform(0, Vec2::new(0.5, 0.49)),
    ));
    add_text(
        commands,
        center,
        format!(
            "{}{}局",
            wind_label(game.prevalent_wind),
            game.sequence_index % 4 + 1
        ),
        28.0,
        Color::srgb(0.96, 0.78, 0.35),
        assets,
    );
    add_text(
        commands,
        center,
        game.wall_len.to_string(),
        28.0,
        Color::srgb(0.95, 0.96, 0.89),
        assets,
    );
    let mut score_texts = Vec::with_capacity(4);
    for relative in 0..4 {
        let player = game
            .players
            .iter()
            .find(|player| (player.seat.0 + 4 - own_seat) % 4 == relative);
        let wind = player.map(|player| player.seat_wind);
        let score = player.map_or(0, |player| scores[usize::from(player.player.id.0)]);
        render_corner_wind(commands, status, relative, wind, game_assets);
        score_texts.push(render_score(
            commands,
            status,
            relative,
            score,
            current_relative == Some(relative),
            assets,
        ));
    }
    score_texts
}

fn render_corner_wind(
    commands: &mut UiComposer,
    status: Entity,
    relative: u8,
    wind: Option<MahjongWind>,
    assets: &MahjongAssets,
) {
    let Some(wind) = wind else { return };
    let uv = match relative {
        0 => Vec2::new(0.1603, 0.8234),
        1 => Vec2::new(0.8389, 0.8234),
        2 => Vec2::new(0.8393, 0.1611),
        _ => Vec2::new(0.1603, 0.1611),
    };
    let position = status_point(uv);
    let badge = spawn_node(
        commands,
        status,
        Node {
            position_type: PositionType::Absolute,
            left: px(position.x - 24.0),
            top: px(position.y - 24.0),
            width: px(48.0),
            height: px(48.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        None,
    );
    commands
        .entity(badge)
        .insert((status_text_transform(relative, uv), Pickable::IGNORE));
    // Center the actual ink rather than a font line box, including when a wind rotates to another corner.
    commands.child(
        badge,
        (
            Node {
                width: px(38),
                height: px(38),
                ..default()
            },
            MaterialNode(assets.wind_labels.get(wind).clone()),
            Pickable::IGNORE,
        ),
    );
}

fn render_score(
    commands: &mut UiComposer,
    status: Entity,
    relative: u8,
    score: i32,
    active: bool,
    assets: &UiAssets,
) -> Entity {
    // These bounds follow the recessed windows in center-panel.png.
    let uv = score_window(relative).center() / 100.0;
    let position = status_point(uv);
    let label = spawn_node(
        commands,
        status,
        Node {
            position_type: PositionType::Absolute,
            left: px(position.x - 55.0),
            top: px(position.y - 18.0),
            width: px(110.0),
            height: px(36.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        None,
    );
    commands.entity(label).insert(Pickable::IGNORE);
    let color = if active {
        TEXT
    } else {
        Color::srgb(0.83, 0.85, 0.83)
    };
    let digits = score.to_string();
    let font_size = (23.0 - digits.len().saturating_sub(6) as f32 * 1.2).max(14.0);
    let text = add_text(commands, label, digits, font_size, color, assets);
    commands.entity(text).insert((
        status_text_transform(relative, uv),
        TextLayout::no_wrap(),
        Pickable::IGNORE,
    ));
    text
}

fn score_window(relative: u8) -> Rect {
    match relative {
        0 => Rect::new(30.0, 75.0, 70.0, 85.0),
        1 => Rect::new(76.0, 30.0, 86.0, 68.0),
        2 => Rect::new(30.0, 14.0, 70.0, 24.0),
        _ => Rect::new(14.0, 30.0, 24.0, 68.0),
    }
}

pub(super) fn mahjong_status_score_position(relative: u8) -> Vec2 {
    status_origin() + status_point(score_window(relative).center() / 100.0)
}

fn status_origin() -> Vec2 {
    Vec2::new(STATUS_LEFT, STATUS_TOP) + central_screen_offset()
}

fn status_point(asset_uv: Vec2) -> Vec2 {
    let uv = asset_uv;
    let denominator = 1.0 - (1.0 - TOP_RATIO) * uv.y;
    Vec2::new(
        0.5 + (uv.x - 0.5) * TOP_RATIO / denominator,
        uv.y * TOP_RATIO / denominator,
    ) * Vec2::new(STATUS_WIDTH, STATUS_HEIGHT)
}

fn status_text_transform(relative: u8, asset_uv: Vec2) -> UiTransform {
    let uv = asset_uv;
    let q = 1.0 - TOP_RATIO;
    let denominator = 1.0 - q * uv.y;
    let unit = STATUS_WIDTH * TOP_RATIO / (1.0 - q * 0.5);
    let x = Vec2::new(STATUS_WIDTH * TOP_RATIO / denominator, 0.0) / unit;
    let y = Vec2::new(
        STATUS_WIDTH * (uv.x - 0.5) * TOP_RATIO * q,
        STATUS_HEIGHT * TOP_RATIO,
    ) / (denominator * denominator * unit);
    let (sine, cosine) = (-f32::from(relative) * consts::FRAC_PI_2).sin_cos();
    let along = x * cosine + y * sine;
    let across = -x * sine + y * cosine;
    UiTransform {
        rotation: Rot2::radians(along.y.atan2(along.x)),
        scale: Vec2::new(along.length(), across.length()),
        ..default()
    }
}

pub(super) fn mahjong_status_score_angle(relative: u8) -> f32 {
    status_text_transform(relative, score_window(relative).center() / 100.0)
        .rotation
        .as_radians()
}

pub(super) fn animate_mahjong_turn_sector(
    time: Res<Time>,
    mut materials: ResMut<Assets<MahjongTurnLightMaterial>>,
    sectors: Query<&MaterialNode<MahjongTurnLightMaterial>, With<MahjongTurnSector>>,
) {
    let breath = (time.elapsed_secs() * consts::TAU / 2.4).sin() * 0.5 + 0.5;
    for material in &sectors {
        if let Some(mut material) = materials.get_mut(&material.0) {
            material.params.y = 0.35 + breath * 0.65;
        }
    }
}
