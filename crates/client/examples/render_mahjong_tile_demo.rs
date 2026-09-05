use std::{
    fs,
    path::{Path, PathBuf},
};

use image::{GrayImage, Luma, Rgba, RgbaImage};

const WIDTH: u32 = 1280;
const HEIGHT: u32 = 720;
const TILE_SCALE: f32 = 315.0;
const HALF_SIZE: [f32; 2] = [0.42, 0.50];
const CORNER_RADIUS: f32 = 0.065;
const INK_BLACK: [u8; 3] = [29, 35, 34];
const INK_RED: [u8; 3] = [199, 51, 57];
const INK_GREEN: [u8; 3] = [20, 137, 92];
const INK_BLUE: [u8; 3] = [37, 102, 164];

struct TileSpec {
    glyph: &'static str,
    center: [f32; 2],
    rotation: f32,
}

struct HeightField {
    width: u32,
    height: u32,
    values: Vec<f32>,
}

fn main() {
    let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let glyph_root = workspace.join("assets/cards/mahjong/hong-kong");
    let runtime_height_root = workspace.join("assets/cards/mahjong/hong-kong-height");
    let height_root = workspace.join("assets/mahjong/demo/height");
    recolor_runtime_glyphs(&glyph_root);
    generate_runtime_height_maps(&glyph_root, &runtime_height_root);
    fs::create_dir_all(&height_root).expect("failed to create the demo height-map directory");
    let output = workspace.join("assets/mahjong/demo/mahjong-tile-lighting-demo.png");
    let closeup_output = workspace.join("assets/mahjong/demo/mahjong-engraving-closeup-demo.png");
    let specs = [
        TileSpec {
            glyph: "characters-1.png",
            center: [256.0, 365.0],
            rotation: -0.105,
        },
        TileSpec {
            glyph: "dots-5.png",
            center: [448.0, 358.0],
            rotation: -0.048,
        },
        TileSpec {
            glyph: "bamboo-1.png",
            center: [640.0, 355.0],
            rotation: 0.016,
        },
        TileSpec {
            glyph: "dragon-red.png",
            center: [832.0, 358.0],
            rotation: 0.070,
        },
        TileSpec {
            glyph: "dragon-white.png",
            center: [1024.0, 365.0],
            rotation: 0.120,
        },
    ];

    let mut canvas = RgbaImage::new(WIDTH, HEIGHT);
    render_felt(&mut canvas);
    for (index, spec) in specs.iter().enumerate() {
        let glyph = image::open(glyph_root.join(spec.glyph))
            .unwrap_or_else(|error| panic!("failed to load {}: {error}", spec.glyph))
            .into_rgba8();
        let height = build_height_field(&glyph);
        let stem = spec.glyph.strip_suffix(".png").unwrap_or(spec.glyph);
        save_height_field(&height, &height_root.join(format!("{stem}-height.png")));
        render_tile(&mut canvas, &glyph, &height, spec, index as u32, TILE_SCALE);
    }
    canvas
        .save(&output)
        .unwrap_or_else(|error| panic!("failed to save {}: {error}", output.display()));

    let glyph = image::open(glyph_root.join("characters-1.png"))
        .expect("failed to load characters-1.png")
        .into_rgba8();
    let height = build_height_field(&glyph);
    let mut closeup = RgbaImage::new(720, 720);
    render_felt(&mut closeup);
    render_tile(
        &mut closeup,
        &glyph,
        &height,
        &TileSpec {
            glyph: "characters-1.png",
            center: [350.0, 350.0],
            rotation: -0.035,
        },
        0,
        560.0,
    );
    closeup
        .save(&closeup_output)
        .unwrap_or_else(|error| panic!("failed to save {}: {error}", closeup_output.display()));
    println!("{}", output.display());
    println!("{}", closeup_output.display());
}

fn recolor_runtime_glyphs(root: &Path) {
    for rank in 1..=9 {
        recolor_glyph(&root.join(format!("characters-{rank}.png")), |_, y| {
            if y < 0.45 { INK_BLACK } else { INK_RED }
        });
        recolor_glyph(&root.join(format!("dots-{rank}.png")), |x, y| {
            dot_ink(rank, x, y)
        });
        recolor_glyph(&root.join(format!("bamboo-{rank}.png")), |x, y| {
            bamboo_ink(rank, x, y)
        });
    }
}

fn recolor_glyph(path: &Path, color_at: impl Fn(f32, f32) -> [u8; 3]) {
    let mut glyph = image::open(path)
        .unwrap_or_else(|error| panic!("failed to load {}: {error}", path.display()))
        .into_rgba8();
    let width = glyph.width() as f32;
    let height = glyph.height() as f32;
    for (x, y, pixel) in glyph.enumerate_pixels_mut() {
        let color = color_at((x as f32 + 0.5) / width, (y as f32 + 0.5) / height);
        pixel[0] = color[0];
        pixel[1] = color[1];
        pixel[2] = color[2];
    }
    glyph
        .save(path)
        .unwrap_or_else(|error| panic!("failed to save {}: {error}", path.display()));
}

fn dot_ink(rank: u8, x: f32, y: f32) -> [u8; 3] {
    match rank {
        1 => {
            let radius = ((x - 0.5).powi(2) + (y - 0.5).powi(2)).sqrt();
            if radius > 0.34 {
                INK_BLUE
            } else if radius > 0.19 {
                INK_GREEN
            } else {
                INK_RED
            }
        }
        2 => nearest_ink(x, y, &[(0.50, 0.28, INK_GREEN), (0.50, 0.72, INK_BLUE)]),
        3 => nearest_ink(
            x,
            y,
            &[
                (0.29, 0.20, INK_BLUE),
                (0.50, 0.50, INK_RED),
                (0.71, 0.80, INK_GREEN),
            ],
        ),
        4 => nearest_ink(
            x,
            y,
            &[
                (0.29, 0.20, INK_BLUE),
                (0.71, 0.20, INK_BLUE),
                (0.29, 0.80, INK_GREEN),
                (0.71, 0.80, INK_GREEN),
            ],
        ),
        5 => nearest_ink(
            x,
            y,
            &[
                (0.29, 0.20, INK_BLUE),
                (0.71, 0.20, INK_BLUE),
                (0.50, 0.50, INK_RED),
                (0.29, 0.80, INK_BLUE),
                (0.71, 0.80, INK_BLUE),
            ],
        ),
        6 => nearest_ink(
            x,
            y,
            &[
                (0.35, 0.18, INK_GREEN),
                (0.65, 0.18, INK_GREEN),
                (0.35, 0.53, INK_RED),
                (0.65, 0.53, INK_RED),
                (0.35, 0.82, INK_RED),
                (0.65, 0.82, INK_RED),
            ],
        ),
        7 => nearest_ink(
            x,
            y,
            &[
                (0.26, 0.16, INK_GREEN),
                (0.50, 0.30, INK_GREEN),
                (0.74, 0.43, INK_GREEN),
                (0.36, 0.58, INK_RED),
                (0.64, 0.58, INK_RED),
                (0.36, 0.82, INK_RED),
                (0.64, 0.82, INK_RED),
            ],
        ),
        8 => INK_BLUE,
        9 => nearest_ink(
            x,
            y,
            &[
                (0.25, 0.20, INK_BLUE),
                (0.50, 0.20, INK_BLUE),
                (0.75, 0.20, INK_BLUE),
                (0.25, 0.50, INK_RED),
                (0.50, 0.50, INK_RED),
                (0.75, 0.50, INK_RED),
                (0.25, 0.80, INK_GREEN),
                (0.50, 0.80, INK_GREEN),
                (0.75, 0.80, INK_GREEN),
            ],
        ),
        _ => INK_BLUE,
    }
}

fn bamboo_ink(rank: u8, x: f32, y: f32) -> [u8; 3] {
    match rank {
        1 if y > 0.46 && y < 0.60 => INK_RED,
        1 if x < 0.46 && y > 0.24 && y < 0.48 => INK_BLUE,
        1 if x > 0.58 && y < 0.26 => INK_RED,
        3 => nearest_ink(
            x,
            y,
            &[
                (0.50, 0.25, INK_RED),
                (0.25, 0.67, INK_GREEN),
                (0.75, 0.67, INK_GREEN),
            ],
        ),
        5 => nearest_ink(
            x,
            y,
            &[
                (0.25, 0.22, INK_GREEN),
                (0.75, 0.22, INK_GREEN),
                (0.50, 0.50, INK_RED),
                (0.25, 0.78, INK_GREEN),
                (0.75, 0.78, INK_GREEN),
            ],
        ),
        6 => nearest_ink(
            x,
            y,
            &[
                (0.20, 0.25, INK_RED),
                (0.50, 0.25, INK_RED),
                (0.80, 0.25, INK_RED),
                (0.20, 0.75, INK_GREEN),
                (0.50, 0.75, INK_GREEN),
                (0.80, 0.75, INK_GREEN),
            ],
        ),
        7 => nearest_ink(
            x,
            y,
            &[
                (0.50, 0.16, INK_RED),
                (0.20, 0.47, INK_GREEN),
                (0.50, 0.47, INK_GREEN),
                (0.80, 0.47, INK_GREEN),
                (0.20, 0.78, INK_GREEN),
                (0.50, 0.78, INK_GREEN),
                (0.80, 0.78, INK_GREEN),
            ],
        ),
        9 => nearest_ink(
            x,
            y,
            &[
                (0.20, 0.20, INK_GREEN),
                (0.50, 0.20, INK_GREEN),
                (0.80, 0.20, INK_GREEN),
                (0.20, 0.50, INK_GREEN),
                (0.50, 0.50, INK_RED),
                (0.80, 0.50, INK_GREEN),
                (0.20, 0.80, INK_GREEN),
                (0.50, 0.80, INK_GREEN),
                (0.80, 0.80, INK_GREEN),
            ],
        ),
        _ => INK_GREEN,
    }
}

fn nearest_ink(x: f32, y: f32, marks: &[(f32, f32, [u8; 3])]) -> [u8; 3] {
    marks
        .iter()
        .min_by(|left, right| {
            let left_distance = (x - left.0).powi(2) + (y - left.1).powi(2);
            let right_distance = (x - right.0).powi(2) + (y - right.1).powi(2);
            left_distance.total_cmp(&right_distance)
        })
        .map(|mark| mark.2)
        .unwrap_or(INK_BLACK)
}

fn generate_runtime_height_maps(glyph_root: &std::path::Path, output: &std::path::Path) {
    fs::create_dir_all(output).expect("failed to create the runtime height-map directory");
    let mut paths = fs::read_dir(glyph_root)
        .expect("failed to read Mahjong glyph directory")
        .map(|entry| entry.expect("failed to read Mahjong glyph entry").path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "png"))
        .filter(|path| path.file_name().is_some_and(|name| name != "back.png"))
        .collect::<Vec<_>>();
    paths.sort();
    for path in paths {
        let glyph = image::open(&path)
            .unwrap_or_else(|error| panic!("failed to load {}: {error}", path.display()))
            .into_rgba8();
        let height = build_height_field(&glyph);
        let file_name = path.file_name().expect("glyph path has a file name");
        save_height_field(&height, &output.join(file_name));
    }
}

fn render_felt(canvas: &mut RgbaImage) {
    let width = canvas.width();
    let height = canvas.height();
    for y in 0..height {
        for x in 0..width {
            let nx = (x as f32 / width as f32 - 0.5) * 2.0;
            let ny = (y as f32 / height as f32 - 0.48) * 2.0;
            let radial = (1.0 - (nx * nx * 0.48 + ny * ny * 0.85).sqrt()).clamp(0.0, 1.0);
            let weave =
                (((x + y * 3) as f32 * 0.19).sin() + ((x * 2 + y) as f32 * 0.13).sin()) * 0.006;
            let grain = (pixel_noise(x, y, 19) - 0.5) * 0.012;
            let vignette = 0.56 + radial * 0.44;
            let color = [
                (0.025 + radial * 0.035 + grain) * vignette,
                (0.105 + radial * 0.125 + weave + grain) * vignette,
                (0.085 + radial * 0.095 + weave * 0.7 + grain) * vignette,
            ];
            canvas.put_pixel(x, y, rgba(color, 1.0));
        }
    }
}

fn render_tile(
    canvas: &mut RgbaImage,
    glyph: &RgbaImage,
    height: &HeightField,
    spec: &TileSpec,
    seed: u32,
    scale: f32,
) {
    let extent = (scale * 0.70) as i32;
    let min_x = (spec.center[0] as i32 - extent).max(0);
    let max_x = (spec.center[0] as i32 + extent).min(canvas.width() as i32 - 1);
    let min_y = (spec.center[1] as i32 - extent).max(0);
    let max_y = (spec.center[1] as i32 + extent).min(canvas.height() as i32 - 1);
    let cosine = spec.rotation.cos();
    let sine = spec.rotation.sin();
    let antialias = 1.35 / scale;

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let world_x = x as f32 + 0.5 - spec.center[0];
            let world_y = spec.center[1] - (y as f32 + 0.5);
            let point = [
                (cosine * world_x + sine * world_y) / scale,
                (-sine * world_x + cosine * world_y) / scale,
            ];

            let face_distance = rounded_box(point, HALF_SIZE, CORNER_RADIUS);
            let side_distance = rounded_box(
                [point[0] - 0.026, point[1] + 0.050],
                HALF_SIZE,
                CORNER_RADIUS,
            );
            let shadow_distance = rounded_box(
                [point[0] - 0.060, point[1] + 0.082],
                HALF_SIZE,
                CORNER_RADIUS + 0.01,
            );
            let face = coverage(face_distance, antialias);
            let side = coverage(side_distance, antialias);
            let shadow = (1.0 - smoothstep(-0.018, 0.085, shadow_distance)) * 0.36;

            if shadow > 0.001 {
                blend(canvas, x as u32, y as u32, [0.002, 0.007, 0.005], shadow);
            }
            if side > 0.001 {
                let side_gradient = (0.55 + point[1] * 0.52 - point[0] * 0.18).clamp(0.0, 1.0);
                let mut side_color =
                    mix3([0.035, 0.235, 0.196], [0.085, 0.415, 0.335], side_gradient);
                let exposed = side * (1.0 - face * 0.985);
                for channel in &mut side_color {
                    *channel += exposed * 0.035;
                }
                blend(canvas, x as u32, y as u32, side_color, side);
            }
            if face <= 0.001 {
                continue;
            }

            let epsilon = 0.002;
            let gradient = normalize2([
                rounded_box([point[0] + epsilon, point[1]], HALF_SIZE, CORNER_RADIUS)
                    - rounded_box([point[0] - epsilon, point[1]], HALF_SIZE, CORNER_RADIUS),
                rounded_box([point[0], point[1] + epsilon], HALF_SIZE, CORNER_RADIUS)
                    - rounded_box([point[0], point[1] - epsilon], HALF_SIZE, CORNER_RADIUS),
            ]);
            let bevel = 1.0 - smoothstep(0.005, 0.070, -face_distance);
            let glyph_uv = glyph_uv(point);
            let glyph_sample = sample_glyph(glyph, glyph_uv);
            let texel = [
                2.5 / (height.width - 1) as f32,
                2.5 / (height.height - 1) as f32,
            ];
            let engraving = sample_height(height, glyph_uv);
            let engraving_gradient = [
                sample_height(height, [glyph_uv[0] + texel[0], glyph_uv[1]])
                    - sample_height(height, [glyph_uv[0] - texel[0], glyph_uv[1]]),
                sample_height(height, [glyph_uv[0], glyph_uv[1] - texel[1]])
                    - sample_height(height, [glyph_uv[0], glyph_uv[1] + texel[1]]),
            ];
            let groove_edge =
                ((engraving_gradient[0].powi(2) + engraving_gradient[1].powi(2)).sqrt() * 3.0)
                    .clamp(0.0, 1.0);
            let normal = normalize3([
                gradient[0] * bevel * 0.82 + engraving_gradient[0] * 2.6,
                gradient[1] * bevel * 0.82 + engraving_gradient[1] * 2.6,
                1.0,
            ]);
            let light = normalize3([-0.52, 0.72, 1.25]);
            let half_vector = normalize3([light[0], light[1], light[2] + 1.0]);
            let diffuse = 0.80 + 0.20 * dot3(normal, light).max(0.0);
            let specular = dot3(normal, half_vector).max(0.0).powf(42.0) * (0.17 + bevel * 0.18);
            let engraving_glint =
                dot3(normal, half_vector).max(0.0).powf(30.0) * groove_edge * 0.48;
            let edge_ao = mix(0.80, 1.0, smoothstep(0.004, 0.054, -face_distance));
            let warmth = (0.54 + point[1] * 0.15 - point[0] * 0.05).clamp(0.0, 1.0);
            let ivory = mix3([0.835, 0.816, 0.755], [0.982, 0.974, 0.925], warmth);
            let grain = (pixel_noise(x as u32, y as u32, seed * 97 + 31) - 0.5) * 0.010;
            let stripe = (point[0] + point[1] * 0.38 + 0.18) / 0.17;
            let sheen = (-stripe * stripe).exp() * 0.022;
            let mut face_color = [0.0; 3];
            for channel in 0..3 {
                face_color[channel] = ivory[channel] * diffuse * edge_ao + specular + sheen + grain;
            }

            for channel in 0..3 {
                face_color[channel] *= 1.0 - engraving * 0.09 - groove_edge * 0.03;
                let ink = glyph_sample[channel] * (0.91 + diffuse * 0.09);
                face_color[channel] = mix(face_color[channel], ink, glyph_sample[3]);
                face_color[channel] += specular * (1.0 - glyph_sample[3] * 0.72);
                face_color[channel] += engraving_glint * (1.0 - glyph_sample[3] * 0.35);
            }
            blend(canvas, x as u32, y as u32, face_color, face);
        }
    }
}

fn rounded_box(point: [f32; 2], half_size: [f32; 2], radius: f32) -> f32 {
    let q = [
        point[0].abs() - half_size[0] + radius,
        point[1].abs() - half_size[1] + radius,
    ];
    q[0].max(q[1]).min(0.0) + (q[0].max(0.0).powi(2) + q[1].max(0.0).powi(2)).sqrt() - radius
}

fn coverage(distance: f32, antialias: f32) -> f32 {
    1.0 - smoothstep(-antialias, antialias, distance)
}

fn glyph_uv(point: [f32; 2]) -> [f32; 2] {
    [
        point[0] / (HALF_SIZE[0] * 2.0) + 0.5,
        0.5 - point[1] / (HALF_SIZE[1] * 2.0),
    ]
}

fn build_height_field(image: &RgbaImage) -> HeightField {
    let width = image.width();
    let height = image.height();
    let mut distance = vec![0.0_f32; (width * height) as usize];
    for y in 0..height {
        for x in 0..width {
            let alpha = image.get_pixel(x, y).0[3];
            if alpha > 3 {
                distance[(y * width + x) as usize] = 1_000_000.0;
            }
        }
    }

    let diagonal = 2.0_f32.sqrt();
    for y in 0..height {
        for x in 0..width {
            let index = (y * width + x) as usize;
            if distance[index] == 0.0 {
                continue;
            }
            if x > 0 {
                distance[index] = distance[index].min(distance[index - 1] + 1.0);
            }
            if y > 0 {
                distance[index] = distance[index].min(distance[index - width as usize] + 1.0);
                if x > 0 {
                    distance[index] =
                        distance[index].min(distance[index - width as usize - 1] + diagonal);
                }
                if x + 1 < width {
                    distance[index] =
                        distance[index].min(distance[index - width as usize + 1] + diagonal);
                }
            }
        }
    }
    for y in (0..height).rev() {
        for x in (0..width).rev() {
            let index = (y * width + x) as usize;
            if distance[index] == 0.0 {
                continue;
            }
            if x + 1 < width {
                distance[index] = distance[index].min(distance[index + 1] + 1.0);
            }
            if y + 1 < height {
                distance[index] = distance[index].min(distance[index + width as usize] + 1.0);
                if x > 0 {
                    distance[index] =
                        distance[index].min(distance[index + width as usize - 1] + diagonal);
                }
                if x + 1 < width {
                    distance[index] =
                        distance[index].min(distance[index + width as usize + 1] + diagonal);
                }
            }
        }
    }

    let values = distance
        .into_iter()
        .enumerate()
        .map(|(index, distance)| {
            if distance == 0.0 {
                return 0.0;
            }
            let x = index as u32 % width;
            let y = index as u32 / width;
            let alpha = image.get_pixel(x, y).0[3] as f32 / 255.0;
            let distance = (distance - 0.65 + alpha * 0.65).max(0.0);
            (1.0 - (-distance / 16.0).exp()).clamp(0.0, 1.0)
        })
        .collect::<Vec<_>>();
    let mut values = blur_height_field(&values, width, height);
    for y in 0..height {
        for x in 0..width {
            let index = (y * width + x) as usize;
            let alpha = image.get_pixel(x, y).0[3] as f32 / 255.0;
            values[index] *= smoothstep(0.0, 1.0, alpha);
        }
    }
    HeightField {
        width,
        height,
        values,
    }
}

fn blur_height_field(values: &[f32], width: u32, height: u32) -> Vec<f32> {
    const WEIGHTS: [f32; 7] = [1.0, 6.0, 15.0, 20.0, 15.0, 6.0, 1.0];
    const WEIGHT_SUM: f32 = 64.0;
    let mut horizontal = vec![0.0; values.len()];
    let mut output = vec![0.0; values.len()];
    for y in 0..height {
        for x in 0..width {
            let mut value = 0.0;
            for (offset, weight) in (-3..=3).zip(WEIGHTS) {
                let sample_x = (x as i32 + offset).clamp(0, width as i32 - 1) as u32;
                value += values[(y * width + sample_x) as usize] * weight;
            }
            horizontal[(y * width + x) as usize] = value / WEIGHT_SUM;
        }
    }
    for y in 0..height {
        for x in 0..width {
            let mut value = 0.0;
            for (offset, weight) in (-3..=3).zip(WEIGHTS) {
                let sample_y = (y as i32 + offset).clamp(0, height as i32 - 1) as u32;
                value += horizontal[(sample_y * width + x) as usize] * weight;
            }
            output[(y * width + x) as usize] = value / WEIGHT_SUM;
        }
    }
    output
}

fn save_height_field(height: &HeightField, path: &std::path::Path) {
    let mut image = GrayImage::new(height.width, height.height);
    for y in 0..height.height {
        for x in 0..height.width {
            let value = (height.values[(y * height.width + x) as usize] * 255.0).round() as u8;
            image.put_pixel(x, y, Luma([value]));
        }
    }
    image
        .save(path)
        .unwrap_or_else(|error| panic!("failed to save {}: {error}", path.display()));
}

fn sample_height(height: &HeightField, uv: [f32; 2]) -> f32 {
    let u = uv[0].clamp(0.0, 1.0) * (height.width - 1) as f32;
    let v = uv[1].clamp(0.0, 1.0) * (height.height - 1) as f32;
    let x0 = u.floor() as u32;
    let y0 = v.floor() as u32;
    let x1 = (x0 + 1).min(height.width - 1);
    let y1 = (y0 + 1).min(height.height - 1);
    let tx = u - x0 as f32;
    let ty = v - y0 as f32;
    let at = |x, y| height.values[(y * height.width + x) as usize];
    mix(
        mix(at(x0, y0), at(x1, y0), tx),
        mix(at(x0, y1), at(x1, y1), tx),
        ty,
    )
}

fn sample_glyph(image: &RgbaImage, uv: [f32; 2]) -> [f32; 4] {
    let u = uv[0].clamp(0.0, 1.0);
    let v = uv[1].clamp(0.0, 1.0);
    let x = u * (image.width() - 1) as f32;
    let y = v * (image.height() - 1) as f32;
    let x0 = x.floor() as u32;
    let y0 = y.floor() as u32;
    let x1 = (x0 + 1).min(image.width() - 1);
    let y1 = (y0 + 1).min(image.height() - 1);
    let tx = x - x0 as f32;
    let ty = y - y0 as f32;
    let top = mix4(pixel4(image, x0, y0), pixel4(image, x1, y0), tx);
    let bottom = mix4(pixel4(image, x0, y1), pixel4(image, x1, y1), tx);
    mix4(top, bottom, ty)
}

fn pixel4(image: &RgbaImage, x: u32, y: u32) -> [f32; 4] {
    let pixel = image.get_pixel(x, y).0;
    pixel.map(|channel| channel as f32 / 255.0)
}

fn blend(canvas: &mut RgbaImage, x: u32, y: u32, source: [f32; 3], alpha: f32) {
    let destination = canvas.get_pixel(x, y).0;
    let alpha = alpha.clamp(0.0, 1.0);
    let mut output = [0_u8; 4];
    for channel in 0..3 {
        let destination = destination[channel] as f32 / 255.0;
        output[channel] = ((source[channel] * alpha + destination * (1.0 - alpha)).clamp(0.0, 1.0)
            * 255.0)
            .round() as u8;
    }
    output[3] = 255;
    canvas.put_pixel(x, y, Rgba(output));
}

fn rgba(color: [f32; 3], alpha: f32) -> Rgba<u8> {
    Rgba([
        (color[0].clamp(0.0, 1.0) * 255.0).round() as u8,
        (color[1].clamp(0.0, 1.0) * 255.0).round() as u8,
        (color[2].clamp(0.0, 1.0) * 255.0).round() as u8,
        (alpha.clamp(0.0, 1.0) * 255.0).round() as u8,
    ])
}

fn pixel_noise(x: u32, y: u32, seed: u32) -> f32 {
    let mut value = x
        .wrapping_mul(0x9e37_79b9)
        .wrapping_add(y.wrapping_mul(0x85eb_ca6b))
        .wrapping_add(seed.wrapping_mul(0xc2b2_ae35));
    value ^= value >> 16;
    value = value.wrapping_mul(0x7feb_352d);
    value ^= value >> 15;
    value = value.wrapping_mul(0x846c_a68b);
    value ^= value >> 16;
    value as f32 / u32::MAX as f32
}

fn smoothstep(edge0: f32, edge1: f32, value: f32) -> f32 {
    let t = ((value - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn mix(a: f32, b: f32, amount: f32) -> f32 {
    a + (b - a) * amount
}

fn mix3(a: [f32; 3], b: [f32; 3], amount: f32) -> [f32; 3] {
    [
        mix(a[0], b[0], amount),
        mix(a[1], b[1], amount),
        mix(a[2], b[2], amount),
    ]
}

fn mix4(a: [f32; 4], b: [f32; 4], amount: f32) -> [f32; 4] {
    [
        mix(a[0], b[0], amount),
        mix(a[1], b[1], amount),
        mix(a[2], b[2], amount),
        mix(a[3], b[3], amount),
    ]
}

fn normalize2(vector: [f32; 2]) -> [f32; 2] {
    let length = (vector[0] * vector[0] + vector[1] * vector[1])
        .sqrt()
        .max(0.00001);
    [vector[0] / length, vector[1] / length]
}

fn normalize3(vector: [f32; 3]) -> [f32; 3] {
    let length = (vector[0] * vector[0] + vector[1] * vector[1] + vector[2] * vector[2])
        .sqrt()
        .max(0.00001);
    [vector[0] / length, vector[1] / length, vector[2] / length]
}

fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
