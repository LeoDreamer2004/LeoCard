use super::control_skin::DrawingControlSkin;
use super::state::DrawTool;
use crate::app::presentation::OreUiSkin;
use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use leocard_draw_guess::{CANVAS_HEIGHT, CANVAS_WIDTH};

#[derive(Resource)]
pub(super) struct DrawGuessAssets {
    pub skin: OreUiSkin,
    pub controls: DrawingControlSkin,
    pub canvas: Handle<Image>,
    pub icons: DrawingIcons,
    pub sounds: DrawingSounds,
}

pub(super) struct DrawingSounds {
    pub draw: Handle<AudioSource>,
    pub erase: Handle<AudioSource>,
    pub fill: Handle<AudioSource>,
    pub correct: Handle<AudioSource>,
    pub reveal: Handle<AudioSource>,
}

pub(super) struct DrawingIcons {
    pen: Handle<Image>,
    eraser: Handle<Image>,
    stroke_eraser: Handle<Image>,
    fill: Handle<Image>,
    pub undo: Handle<Image>,
    pub redo: Handle<Image>,
    pub clear: Handle<Image>,
}

impl DrawingIcons {
    pub fn tool(&self, tool: DrawTool) -> Handle<Image> {
        match tool {
            DrawTool::Pen => &self.pen,
            DrawTool::AreaEraser => &self.eraser,
            DrawTool::StrokeEraser => &self.stroke_eraser,
            DrawTool::Fill => &self.fill,
        }
        .clone()
    }
}

pub(super) fn load(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
) {
    let image = Image::new_fill(
        Extent3d {
            width: u32::from(CANVAS_WIDTH),
            height: u32::from(CANVAS_HEIGHT),
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[255; 4],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );
    commands.insert_resource(DrawGuessAssets {
        skin: OreUiSkin::load(&asset_server),
        controls: DrawingControlSkin::new(&images),
        canvas: images.add(image),
        sounds: DrawingSounds {
            draw: asset_server.load("audio/draw_guess/draw.ogg"),
            erase: asset_server.load("audio/draw_guess/erase.ogg"),
            fill: asset_server.load("audio/draw_guess/fill.ogg"),
            correct: asset_server.load("vendor/kenney/interface-sounds/Audio/confirmation_001.ogg"),
            reveal: asset_server.load("vendor/kenney/interface-sounds/Audio/confirmation_004.ogg"),
        },
        icons: DrawingIcons {
            pen: asset_server.load("ui/draw_guess/icons/pen.png"),
            eraser: asset_server.load("ui/draw_guess/icons/eraser.png"),
            stroke_eraser: asset_server.load("ui/draw_guess/icons/stroke-eraser.png"),
            fill: asset_server.load("ui/draw_guess/icons/fill.png"),
            undo: asset_server.load("ui/draw_guess/icons/undo.png"),
            redo: asset_server.load("ui/draw_guess/icons/redo.png"),
            clear: asset_server.load("ui/draw_guess/icons/clear.png"),
        },
    });
}
