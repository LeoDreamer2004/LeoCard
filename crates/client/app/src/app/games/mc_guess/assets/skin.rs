use super::{EntryIcons, IconSprite};
use bevy::image::{ImageLoaderSettings, ImageSampler};
use bevy::prelude::*;
use bevy::ui::VisualBox;
use leocard_mc_guess::{McGuessEntry, McGuessEntryKind, McGuessGraphNodeView};
use leocard_ui::{ScrollbarSkin, TextInputSkin};

#[derive(Resource)]
pub(crate) struct McGuessAssets {
    frame: Handle<Image>,
    popup_frame: Handle<Image>,
    inset: Handle<Image>,
    cell: Handle<Image>,
    surface: Handle<Image>,
    scroll_track: Handle<Image>,
    scroll_thumb: Handle<Image>,
    disabled_scroll_thumb: Handle<Image>,
    button: Handle<Image>,
    disabled_button: Handle<Image>,
    highlighted_button: Handle<Image>,
    close_icon: Handle<Image>,
    input: Handle<Image>,
    focused_input: Handle<Image>,
    icons: EntryIcons,
    node_symbols: Handle<Image>,
}

impl McGuessAssets {
    pub(in super::super) fn close_icon(&self) -> Handle<Image> {
        self.close_icon.clone()
    }

    pub(in super::super) fn icon(&self, entry: &McGuessEntry) -> ImageNode {
        self.icons.image(entry)
    }

    pub(in super::super) fn node_sprite(
        &self,
        node: &McGuessGraphNodeView,
        layouts: &Assets<TextureAtlasLayout>,
    ) -> IconSprite {
        if let Some(entry) = &node.entry {
            return self.icons.sprite(entry, layouts);
        }
        let index = match node.kind {
            Some(McGuessEntryKind::Block) | None => 0,
            Some(McGuessEntryKind::Item) => 1,
            Some(McGuessEntryKind::Mob) => 2,
            Some(McGuessEntryKind::Enchantment) => 3,
            Some(McGuessEntryKind::Effect) => 4,
            Some(McGuessEntryKind::VillagerProfession) => 5,
            Some(McGuessEntryKind::Landscape) => 6,
            Some(McGuessEntryKind::Biome) => 7,
        };
        IconSprite {
            image: self.node_symbols.clone(),
            region: Vec4::new(index as f32 * 128.0, 0.0, 128.0, 128.0),
        }
    }

    pub(in super::super) fn frame(&self) -> ImageNode {
        Self::image(self.frame.clone())
    }

    pub(in super::super) fn popup_frame(&self) -> ImageNode {
        Self::image(self.popup_frame.clone())
    }

    pub(in super::super) fn inset(&self) -> ImageNode {
        Self::recessed(self.inset.clone())
    }

    pub(in super::super) fn surface(&self) -> ImageNode {
        ImageNode::new(self.surface.clone())
            .with_mode(NodeImageMode::Stretch)
            .with_visual_box(VisualBox::PaddingBox)
    }

    pub(in super::super) fn cell(&self, tint: Color) -> ImageNode {
        ImageNode::new(self.cell.clone())
            .with_mode(NodeImageMode::Stretch)
            .with_color(tint)
            .with_visual_box(VisualBox::PaddingBox)
    }

    pub(in super::super) fn scrollbar(&self) -> ScrollbarSkin {
        ScrollbarSkin::textured(
            Self::recessed(self.scroll_track.clone()),
            Self::sliced(self.scroll_thumb.clone(), 1.0),
            Self::sliced(self.highlighted_button.clone(), 3.0),
            Self::sliced(self.disabled_scroll_thumb.clone(), 1.0),
        )
    }

    pub(in super::super) fn button(&self, enabled: bool, highlighted: bool) -> ImageNode {
        Self::sliced(
            if !enabled {
                self.disabled_button.clone()
            } else if highlighted {
                self.highlighted_button.clone()
            } else {
                self.button.clone()
            },
            3.0,
        )
    }

    pub(in super::super) fn input(&self) -> TextInputSkin {
        TextInputSkin::new(
            Self::sliced(self.input.clone(), 3.0),
            Self::sliced(self.focused_input.clone(), 3.0),
        )
    }

    fn image(texture: Handle<Image>) -> ImageNode {
        Self::sliced(texture, 2.0)
    }

    fn recessed(texture: Handle<Image>) -> ImageNode {
        ImageNode::new(texture)
            .with_mode(NodeImageMode::Sliced(TextureSlicer {
                border: BorderRect {
                    min_inset: Vec2::new(1.0, 3.0),
                    max_inset: Vec2::ONE,
                },
                center_scale_mode: SliceScaleMode::Stretch,
                sides_scale_mode: SliceScaleMode::Stretch,
                max_corner_scale: 2.0,
            }))
            .with_visual_box(VisualBox::BorderBox)
    }

    fn sliced(texture: Handle<Image>, border: f32) -> ImageNode {
        ImageNode::new(texture)
            .with_mode(NodeImageMode::Sliced(TextureSlicer {
                border: BorderRect::all(border),
                center_scale_mode: SliceScaleMode::Stretch,
                sides_scale_mode: SliceScaleMode::Stretch,
                max_corner_scale: 2.0,
            }))
            .with_visual_box(VisualBox::BorderBox)
    }
}

pub(in super::super) fn load_guess_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let load = |path: &'static str| {
        asset_server
            .load_builder()
            .with_settings(|settings: &mut ImageLoaderSettings| {
                settings.sampler = ImageSampler::nearest();
            })
            .load(path)
    };
    commands.insert_resource(McGuessAssets {
        frame: load("ui/minecraft_guess/widgets/table-frame.png"),
        popup_frame: load("ui/minecraft_guess/widgets/popup-frame.png"),
        inset: load("ui/minecraft_guess/widgets/table-inset.png"),
        cell: load("ui/minecraft_guess/widgets/table-cell.png"),
        surface: load("ui/minecraft_guess/widgets/table-surface.png"),
        scroll_track: load("ui/minecraft_guess/widgets/scroll-track.png"),
        scroll_thumb: load("ui/minecraft_guess/widgets/scroll-thumb.png"),
        disabled_scroll_thumb: load("ui/minecraft_guess/widgets/scroll-thumb-disabled.png"),
        button: load("ui/minecraft_guess/widgets/button.png"),
        disabled_button: load("ui/minecraft_guess/widgets/button-disabled.png"),
        highlighted_button: load("ui/minecraft_guess/widgets/button-highlighted.png"),
        close_icon: load("ui/minecraft_guess/widgets/close.png"),
        input: load("ui/minecraft_guess/widgets/input.png"),
        focused_input: load("ui/minecraft_guess/widgets/input-focused.png"),
        icons: EntryIcons::new(&asset_server, &mut layouts),
        node_symbols: load("ui/minecraft_guess/graph/node-symbols.png"),
    });
}
