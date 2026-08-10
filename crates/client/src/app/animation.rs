//! 跨游戏共用的轻量动画曲线。

pub(super) fn ease_out_cubic(value: f32) -> f32 {
    1.0 - (1.0 - value).powi(3)
}
