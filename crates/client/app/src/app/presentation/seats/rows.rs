use bevy::prelude::*;

/// 横排或折行的玩家列表；与牌桌环形布局共享人数入口。
pub(crate) struct SeatRows {
    count: u8,
    columns: u8,
}

impl SeatRows {
    pub(crate) fn new(count: u8, max_columns: u8) -> Self {
        Self {
            count,
            columns: count.min(max_columns),
        }
    }

    pub(crate) fn node(&self, row_height: f32) -> Node {
        Node {
            width: percent(100),
            height: px(row_height * f32::from(self.count.div_ceil(self.columns))),
            flex_wrap: if self.count > self.columns {
                FlexWrap::Wrap
            } else {
                FlexWrap::NoWrap
            },
            flex_shrink: 0.0,
            justify_content: JustifyContent::SpaceEvenly,
            align_items: AlignItems::Center,
            ..default()
        }
    }

    pub(crate) fn cell_width(&self) -> Val {
        percent(100.0 / f32::from(self.columns))
    }
}
