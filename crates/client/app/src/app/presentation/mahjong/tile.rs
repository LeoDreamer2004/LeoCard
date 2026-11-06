use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::ShaderRef;
use leocard_ui::{UiComposer, ui_component};

const MAHJONG_TILE_SHADER: &str = "shaders/mahjong_tile.wesl";

#[derive(AsBindGroup, Asset, TypePath, Debug, Clone, PartialEq, Default)]
pub(crate) struct MahjongTileMaterial {
    /// Scene cards use the measured table camera; explanatory UI cards remain front-facing.
    pub table_projection: bool,
    /// Foreground melds keep their screen anchors and are unprojected onto the table first.
    pub overlay_projection: bool,
    pub paint_order: i32,
    /// x: 交互；y: 正背面；z: 可见度；w: -6 自家、-5 暗杠、-4 副露、-3 对家、-2 侧家、-1 展示立牌、2 双层墙、3 下层牌。
    #[uniform(0)]
    pub params: Vec4,
    /// xy: 局部光照方向；z: 同牌提示；w: 牌体相对屏幕的实际旋转角。
    #[uniform(0)]
    pub lighting: Vec4,
    /// rgb: 状态蒙版颜色；w: 强度，优先于同牌提示。
    #[uniform(0)]
    pub highlight: Vec4,
    /// 牌体离桌高度，用于牌体投影与落地阴影，不叠加矩形 UI 阴影。
    #[uniform(0)]
    pub elevation: f32,
    /// Projected ground shadows are painted below every solid tile in a shared table layer.
    #[uniform(0)]
    pub shadow_only: f32,
    #[uniform(0)]
    pub paint_bounds: Vec4,
    #[uniform(0)]
    pub ray_origin: Vec4,
    #[uniform(0)]
    pub ray_forward: Vec4,
    #[uniform(0)]
    pub ray_right: Vec4,
    #[uniform(0)]
    pub ray_down: Vec4,
    #[uniform(0)]
    pub box_half_size: Vec4,
    #[uniform(0)]
    pub response: Vec4,
    #[texture(1)]
    #[sampler(2)]
    pub glyph: Handle<Image>,
    #[texture(3)]
    #[sampler(4)]
    pub height: Handle<Image>,
}

/// Layout, picking and animations stay on the tile; its padded painting surface can cast a shadow outside it.
#[derive(Component)]
pub(crate) struct MahjongTileSurface(pub Handle<MahjongTileMaterial>);

#[derive(Component)]
pub(crate) struct MahjongTileRestRotation(pub f32);

pub(crate) fn attach_mahjong_tile_surface(
    commands: &mut UiComposer,
    tile: Entity,
    material: Handle<MahjongTileMaterial>,
    projected: bool,
) {
    commands
        .entity(tile)
        .insert(MahjongTileSurface(material.clone()));
    if projected {
        return;
    }
    let (offset, extent) = (-25, 150);
    commands.mount(
        tile,
        "tile-surface",
        bsn! {
            Node {
                position_type: PositionType::Absolute,
                left: percent(offset), top: percent(offset), width: percent(extent), height: percent(extent),
            }
            @ui_component(MaterialNode(material))
            Pickable::IGNORE
        },
    );
}

#[expect(
    clippy::type_complexity,
    reason = "synchronize changed transforms and retained material bindings"
)]
pub(super) fn sync_mahjong_tile_orientation(
    tiles: Query<
        (&UiGlobalTransform, &MahjongTileSurface),
        Or<(Changed<UiGlobalTransform>, Changed<MahjongTileSurface>)>,
    >,
    mut materials: ResMut<Assets<MahjongTileMaterial>>,
) {
    for (transform, surface) in &tiles {
        let axis = transform.matrix2.x_axis;
        let angle = axis.y.atan2(axis.x);
        let direction = Rot2::radians(-angle) * Vec2::new(-0.50, -0.72);
        if let Some(mut material) = materials.get_mut(&surface.0) {
            material.lighting.x = direction.x;
            material.lighting.y = direction.y;
            material.lighting.w = angle;
        }
    }
}

#[derive(Clone, Copy, Default)]
pub(crate) enum MahjongTileHighlight {
    #[default]
    None,
    Red,
}

impl MahjongTileHighlight {
    pub(crate) fn overlay(self) -> Vec4 {
        match self {
            Self::None => Vec4::ZERO,
            Self::Red => Vec4::new(1.0, 0.18, 0.16, 0.52),
        }
    }
}

impl UiMaterial for MahjongTileMaterial {
    fn vertex_shader() -> ShaderRef {
        "shaders/mahjong_tile_vertex.wesl".into()
    }

    fn fragment_shader() -> ShaderRef {
        MAHJONG_TILE_SHADER.into()
    }
}

pub(crate) fn mahjong_local_light(orientation: u8) -> Vec4 {
    let direction = match orientation {
        0 => Vec2::new(-0.50, -0.72),
        1 => Vec2::new(0.72, -0.50),
        2 => Vec2::new(0.50, 0.72),
        _ => Vec2::new(-0.72, 0.50),
    };
    direction.extend(0.0).extend(0.0)
}
