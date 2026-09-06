use super::*;

const UNO_PALETTE_SHADER: &str = "shaders/uno_palette.wgsl";

/// GPU 直接绘制调色轮。params: x=选中色编号，y=0 底盘/1 独立扇区，z=透明度。
#[derive(AsBindGroup, Asset, TypePath, Debug, Clone)]
pub struct UnoPaletteMaterial {
    #[uniform(0)]
    pub params: Vec4,
}

impl UnoPaletteMaterial {
    pub fn new(selected: UnoColor, selected_sector: bool) -> Self {
        let selected = match selected {
            UnoColor::Red => 0.0,
            UnoColor::Yellow => 1.0,
            UnoColor::Green => 2.0,
            UnoColor::Blue => 3.0,
            UnoColor::Pink => 4.0,
            UnoColor::Teal => 5.0,
            UnoColor::Orange => 6.0,
            UnoColor::Purple => 7.0,
        };
        Self {
            params: Vec4::new(selected, if selected_sector { 1.0 } else { 0.0 }, 0.0, 0.0),
        }
    }
}

impl UiMaterial for UnoPaletteMaterial {
    fn fragment_shader() -> ShaderRef {
        UNO_PALETTE_SHADER.into()
    }
}
