use bevy::prelude::*;
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::ShaderRef;

const MAHJONG_TILE_SHADER: &str = "shaders/mahjong_tile.wgsl";

#[derive(AsBindGroup, Asset, TypePath, Debug, Clone)]
pub(crate) struct MahjongTileMaterial {
    /// x: 交互；y: 正背面；z: 可见度；w: -5 暗杠、-4 副露、-3 对家、-2 侧家、-1 自家、2 双层墙、3 下层牌。
    #[uniform(0)]
    pub params: Vec4,
    /// xy: 局部光照方向；z: 同张牌牌面蓝色蒙版强度。
    #[uniform(0)]
    pub lighting: Vec4,
    /// rgb: 状态蒙版颜色；w: 强度，优先于同牌提示。
    #[uniform(0)]
    pub highlight: Vec4,
    #[texture(1)]
    #[sampler(2)]
    pub glyph: Handle<Image>,
    #[texture(3)]
    #[sampler(4)]
    pub height: Handle<Image>,
}

#[derive(Clone, Copy, Default)]
pub(crate) enum MahjongTileHighlight {
    #[default]
    None,
    Red,
}

impl MahjongTileHighlight {
    pub(super) fn overlay(self) -> Vec4 {
        match self {
            Self::None => Vec4::ZERO,
            Self::Red => Vec4::new(1.0, 0.18, 0.16, 0.52),
        }
    }
}

impl UiMaterial for MahjongTileMaterial {
    fn fragment_shader() -> ShaderRef {
        MAHJONG_TILE_SHADER.into()
    }
}

pub(super) fn mahjong_local_light(orientation: u8) -> Vec4 {
    let direction = match orientation {
        0 => Vec2::new(-0.50, -0.72),
        1 => Vec2::new(0.72, -0.50),
        2 => Vec2::new(0.50, 0.72),
        _ => Vec2::new(-0.72, 0.50),
    };
    direction.extend(0.0).extend(0.0)
}

pub(super) fn mahjong_local_shadow(orientation: u8) -> Vec2 {
    match orientation {
        0 => Vec2::new(2.0, 5.0),
        1 => Vec2::new(-5.0, 2.0),
        2 => Vec2::new(-2.0, -5.0),
        _ => Vec2::new(5.0, -2.0),
    }
}
