//! Shared card-table artwork and the visible bounds of its generated textures.

use bevy::image::{ImageLoaderSettings, ImageSampler};
use bevy::prelude::*;
use bevy::ui::VisualBox;
use leocard_ui::sliced_image;

#[derive(Default)]
pub(crate) struct TableSurfaceAssets {
    pub felt: Handle<Image>,
    pub rail: Handle<Image>,
    /// Inner opening and clean outer silhouette in normalized source coordinates.
    pub felt_bounds: Vec4,
    pub frame_bounds: Vec4,
}

impl TableSurfaceAssets {
    pub(crate) fn load(
        server: &AssetServer,
        directory: &str,
        felt_bounds: Vec4,
        frame_bounds: Vec4,
    ) -> Self {
        let load = |layer| {
            server
                .load_builder()
                .with_settings(|settings: &mut ImageLoaderSettings| {
                    settings.sampler = ImageSampler::linear();
                })
                .load(format!("{directory}/table-{layer}.png"))
        };
        Self {
            felt: load("felt"),
            rail: load("rail"),
            felt_bounds,
            frame_bounds,
        }
    }
}

#[derive(Default)]
pub(crate) struct TabletopAssets {
    pub surface: TableSurfaceAssets,
    side_frame: Handle<Image>,
    hanging_sign: Handle<Image>,
    side_fold: Handle<Image>,
    side_robot: Handle<Image>,
    side_help: Handle<Image>,
    side_button: Handle<Image>,
}

impl TabletopAssets {
    pub(crate) fn load(server: &AssetServer) -> Self {
        let load = |name| {
            server
                .load_builder()
                .with_settings(|settings: &mut ImageLoaderSettings| {
                    settings.sampler = ImageSampler::linear();
                })
                .load(format!("ui/tabletop/{name}.png"))
        };
        Self {
            surface: TableSurfaceAssets::load(
                server,
                "ui/tabletop",
                Vec4::new(108.0, 102.0, 1647.0, 791.0) / Vec4::new(1774.0, 887.0, 1774.0, 887.0),
                Vec4::new(49.0, 39.0, 1707.0, 858.0) / Vec4::new(1774.0, 887.0, 1774.0, 887.0),
            ),
            side_frame: load("side-frame"),
            hanging_sign: load("hanging-sign"),
            side_fold: load("side-fold"),
            side_robot: load("side-robot"),
            side_help: load("side-help"),
            side_button: load("side-button"),
        }
    }

    pub(crate) fn side_frame_image(&self) -> ImageNode {
        sliced_image(self.side_frame.clone(), 28.0, 0.5)
    }

    pub(crate) fn hanging_sign_image(&self, scale: f32) -> ImageNode {
        let mut image = ImageNode::new(self.hanging_sign.clone()).with_mode(NodeImageMode::Sliced(
            TextureSlicer {
                // The two suspension cords stay inside the fixed upper corner patches.
                border: BorderRect {
                    min_inset: Vec2::new(120.0, 70.0),
                    max_inset: Vec2::new(120.0, 24.0),
                },
                center_scale_mode: SliceScaleMode::Stretch,
                sides_scale_mode: SliceScaleMode::Stretch,
                max_corner_scale: 0.55 * scale,
            },
        ));
        image.visual_box = VisualBox::BorderBox;
        image
    }

    pub(crate) fn fold_image(&self, expanded: bool) -> ImageNode {
        let mut image = ImageNode::new(self.side_fold.clone());
        image.flip_x = expanded;
        image
    }

    pub(crate) fn robot_image(&self) -> ImageNode {
        ImageNode::new(self.side_robot.clone())
    }

    pub(crate) fn help_image(&self) -> ImageNode {
        ImageNode::new(self.side_help.clone())
    }

    pub(crate) fn side_button_image(&self) -> ImageNode {
        ImageNode::new(self.side_button.clone())
    }

    pub(crate) fn drawer_image(&self) -> ImageNode {
        sliced_image(self.side_button.clone(), 12.0, 0.75)
    }
}
