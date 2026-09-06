//! 跨游戏共用的手牌运动状态与轻量缓动曲线。

#[derive(Clone, Copy, Default, PartialEq)]
pub struct CardAnimationState {
    pub slot_hover_amount: f32,
    pub face_hover_amount: f32,
    pub selected_amount: f32,
    pub deal_elapsed: f32,
    pub dealing: bool,
}

pub fn ease_out_cubic(value: f32) -> f32 {
    1.0 - (1.0 - value).powi(3)
}

pub fn smootherstep(value: f32) -> f32 {
    let value = value.clamp(0.0, 1.0);
    value * value * value * (value * (value * 6.0 - 15.0) + 10.0)
}
