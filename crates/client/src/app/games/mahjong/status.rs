use super::{MahjongAssets, MahjongTurnSector, wind_label};
use crate::app::presentation::{PlayerSeatValue, add_player_seat_value, add_text, spawn_node};
use crate::app::runtime::UiAssets;
use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use leocard_protocol::{MahjongPhaseView, MahjongSnapshot};

const STATUS_LEFT: f32 = 570.0;
const STATUS_TOP: f32 = 276.0;
const STATUS_WIDTH: u32 = 140;
const STATUS_HEIGHT: u32 = 104;
const INNER_LEFT: f32 = 24.0;
const INNER_TOP: f32 = 20.0;
const INNER_WIDTH: f32 = 92.0;
const INNER_HEIGHT: f32 = 64.0;
const FRAME_COLOR: Color = Color::srgba(0.004, 0.035, 0.040, 0.55);

pub(super) struct MahjongStatusImages {
    pub outline: Handle<Image>,
    pub sectors: [Handle<Image>; 4],
}

pub(super) fn create_mahjong_status_images(images: &mut Assets<Image>) -> MahjongStatusImages {
    const SCALE: u32 = 4;
    let width = STATUS_WIDTH * SCALE;
    let height = STATUS_HEIGHT * SCALE;
    let mut outline = image::RgbaImage::new(width, height);
    let mut sectors = std::array::from_fn(|_| image::RgbaImage::new(width, height));
    let corners = [
        (Vec2::new(0.0, 0.0), Vec2::new(INNER_LEFT, INNER_TOP)),
        (
            Vec2::new(STATUS_WIDTH as f32, 0.0),
            Vec2::new(INNER_LEFT + INNER_WIDTH, INNER_TOP),
        ),
        (
            Vec2::new(0.0, STATUS_HEIGHT as f32),
            Vec2::new(INNER_LEFT, INNER_TOP + INNER_HEIGHT),
        ),
        (
            Vec2::new(STATUS_WIDTH as f32, STATUS_HEIGHT as f32),
            Vec2::new(INNER_LEFT + INNER_WIDTH, INNER_TOP + INNER_HEIGHT),
        ),
    ];
    for y in 0..height {
        for x in 0..width {
            let point = Vec2::new(
                (x as f32 + 0.5) / SCALE as f32,
                (y as f32 + 0.5) / SCALE as f32,
            );
            if let Some(relative) = status_sector_at(point) {
                sectors[relative as usize].put_pixel(x, y, image::Rgba([255, 255, 255, 255]));
                if corners
                    .iter()
                    .any(|&(start, end)| point_segment_distance(point, start, end) < 0.65)
                {
                    outline.put_pixel(x, y, image::Rgba([2, 14, 16, 180]));
                }
            }
        }
    }
    MahjongStatusImages {
        outline: add_status_image(images, outline),
        sectors: sectors.map(|mask| add_status_image(images, mask)),
    }
}

fn add_status_image(images: &mut Assets<Image>, pixels: image::RgbaImage) -> Handle<Image> {
    let pixels = image::imageops::resize(
        &pixels,
        STATUS_WIDTH,
        STATUS_HEIGHT,
        image::imageops::FilterType::Triangle,
    );
    images.add(Image::from_dynamic(
        image::DynamicImage::ImageRgba8(pixels),
        true,
        RenderAssetUsages::default(),
    ))
}

fn status_sector_at(point: Vec2) -> Option<u8> {
    let x = point.x;
    let y = point.y;
    let width = STATUS_WIDTH as f32;
    let height = STATUS_HEIGHT as f32;
    let nearest_x = x.clamp(12.0, width - 12.0);
    let nearest_y = y.clamp(12.0, height - 12.0);
    if (point - Vec2::new(nearest_x, nearest_y)).length_squared() > 12.0 * 12.0
        || x < 1.0
        || y < 1.0
        || x >= width - 1.0
        || y >= height - 1.0
        || ((INNER_LEFT..INNER_LEFT + INNER_WIDTH).contains(&x)
            && (INNER_TOP..INNER_TOP + INNER_HEIGHT).contains(&y))
    {
        return None;
    }
    let distances = [
        (height - y) / INNER_TOP,
        (width - x) / INNER_LEFT,
        y / INNER_TOP,
        x / INNER_LEFT,
    ];
    distances
        .iter()
        .enumerate()
        .min_by(|left, right| left.1.total_cmp(right.1))
        .map(|(relative, _)| relative as u8)
}

fn point_segment_distance(point: Vec2, start: Vec2, end: Vec2) -> f32 {
    let segment = end - start;
    let fraction = ((point - start).dot(segment) / segment.length_squared()).clamp(0.0, 1.0);
    (point - (start + segment * fraction)).length()
}

pub(super) fn render_round_status(
    commands: &mut Commands,
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
    commands: &mut Commands,
    table: Entity,
    game: &MahjongSnapshot,
    own_seat: u8,
    scores: &[i32; 4],
    assets: &UiAssets,
    game_assets: &MahjongAssets,
) -> Vec<Entity> {
    let status = spawn_node(
        commands,
        table,
        Node {
            position_type: PositionType::Absolute,
            left: px(STATUS_LEFT),
            top: px(STATUS_TOP),
            width: px(STATUS_WIDTH),
            height: px(STATUS_HEIGHT),
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(12)),
            overflow: Overflow::clip(),
            ..default()
        },
        Some(Color::srgba(0.016, 0.114, 0.125, 0.97)),
    );
    commands.entity(status).insert((
        BorderColor::all(FRAME_COLOR),
        BoxShadow::new(Color::BLACK.with_alpha(0.46), px(0), px(4), px(0), px(9)),
        FocusPolicy::Pass,
        ZIndex(20),
    ));

    let current_relative = matches!(game.phase, MahjongPhaseView::Playing).then(|| {
        let current_seat = game
            .players
            .iter()
            .find(|player| player.id == game.current_player)
            .map(|player| player.seat.0)
            .unwrap_or(own_seat);
        (current_seat + 4 - own_seat) % 4
    });
    if let Some(relative) = current_relative {
        let highlight = spawn_status_image(
            commands,
            status,
            &game_assets.status.sectors[relative as usize],
            Color::srgba(0.12, 0.42, 0.42, 0.18),
        );
        commands.entity(highlight).insert(MahjongTurnSector);
    }
    spawn_status_image(commands, status, &game_assets.status.outline, Color::WHITE);

    let center = spawn_node(
        commands,
        status,
        Node {
            position_type: PositionType::Absolute,
            left: px(INNER_LEFT),
            top: px(INNER_TOP),
            width: px(INNER_WIDTH),
            height: px(INNER_HEIGHT),
            border: UiRect::all(px(1)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            row_gap: px(1),
            ..default()
        },
        Some(Color::srgba(0.020, 0.149, 0.161, 0.97)),
    );
    commands
        .entity(center)
        .insert((BorderColor::all(FRAME_COLOR), FocusPolicy::Pass));
    add_text(
        commands,
        center,
        format!(
            "{}{}局",
            wind_label(game.prevalent_wind),
            game.sequence_index % 4 + 1
        ),
        18.0,
        Color::srgb(0.92, 0.79, 0.43),
        assets,
    );
    add_text(
        commands,
        center,
        game.wall_len.to_string(),
        22.0,
        Color::srgb(0.95, 0.96, 0.89),
        assets,
    );

    let mut score_texts = Vec::with_capacity(4);
    for relative in 0..4 {
        let player = game
            .players
            .iter()
            .find(|player| (player.seat.0 + 4 - own_seat) % 4 == relative);
        let wind = player
            .map(|player| wind_label(player.seat_wind))
            .unwrap_or("?");
        let score = player
            .map(|player| scores[usize::from(player.id.0)])
            .unwrap_or(0);
        score_texts.push(render_wind_label(
            commands,
            status,
            relative,
            (wind, score),
            current_relative == Some(relative),
            assets,
        ));
    }
    score_texts
}

fn spawn_status_image(
    commands: &mut Commands,
    status: Entity,
    image: &Handle<Image>,
    tint: Color,
) -> Entity {
    let entity = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                top: px(0),
                width: px(STATUS_WIDTH),
                height: px(STATUS_HEIGHT),
                ..default()
            },
            ImageNode::new(image.clone()).with_color(tint),
            FocusPolicy::Pass,
        ))
        .id();
    commands.entity(status).add_child(entity);
    entity
}

fn render_wind_label(
    commands: &mut Commands,
    status: Entity,
    relative: u8,
    (wind, score): (&str, i32),
    active: bool,
    assets: &UiAssets,
) -> Entity {
    let (left, top, width, height) = match relative {
        0 => (INNER_LEFT, INNER_TOP + INNER_HEIGHT, INNER_WIDTH, INNER_TOP),
        1 => (
            INNER_LEFT + INNER_WIDTH,
            INNER_TOP,
            INNER_LEFT,
            INNER_HEIGHT,
        ),
        2 => (INNER_LEFT, 0.0, INNER_WIDTH, INNER_TOP),
        _ => (0.0, INNER_TOP, INNER_LEFT, INNER_HEIGHT),
    };
    let label = spawn_node(
        commands,
        status,
        Node {
            position_type: PositionType::Absolute,
            left: px(left),
            top: px(top),
            width: px(width),
            height: px(height),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        None,
    );
    commands.entity(label).insert(FocusPolicy::Pass);
    add_player_seat_value(
        commands,
        label,
        PlayerSeatValue {
            label: wind,
            value: score,
            width: if relative.is_multiple_of(2) {
                width
            } else {
                height
            },
            rotation: -(relative as f32) * std::f32::consts::FRAC_PI_2,
            active,
        },
        assets,
    )
}

pub(super) fn animate_mahjong_turn_sector(
    time: Res<Time>,
    mut sectors: Query<&mut ImageNode, With<MahjongTurnSector>>,
) {
    let breath = (time.elapsed_secs() * std::f32::consts::PI * 0.9).sin() * 0.5 + 0.5;
    let alpha = 0.12 + breath * 0.24;
    for mut sector in &mut sectors {
        sector.color = Color::srgba(0.12, 0.42, 0.42, alpha);
    }
}
