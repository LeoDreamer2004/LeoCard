use bevy::prelude::Vec2;

/// 头像旁的弹层与出牌区朝向；本人的横向出牌区也使用 Top。
#[derive(Clone, Copy)]
pub(crate) enum SeatSide {
    Left,
    Top,
    Right,
}

/// 顺时针绕桌的相对座位，两侧行号从下向上。
#[derive(Clone, Copy)]
pub(crate) enum TableSeat {
    Bottom,
    Left(usize),
    Top,
    Right(usize),
}

impl TableSeat {
    pub(crate) fn side(self) -> SeatSide {
        match self {
            Self::Bottom | Self::Top => SeatSide::Top,
            Self::Left(_) => SeatSide::Left,
            Self::Right(_) => SeatSide::Right,
        }
    }

    pub(crate) fn mirrored(self) -> Self {
        match self {
            Self::Left(row) => Self::Right(row),
            Self::Right(row) => Self::Left(row),
            _ => self,
        }
    }

    pub(crate) fn column(self, bottom: f32, top: f32, sides: Vec2) -> f32 {
        match self {
            Self::Bottom => bottom,
            Self::Top => top,
            Self::Left(_) => sides.x,
            Self::Right(_) => sides.y,
        }
    }

    pub(crate) fn position(self, anchors: TableSeatAnchors<'_>) -> Vec2 {
        let x = self.column(anchors.bottom.x, anchors.top.x, anchors.columns);
        let y = match self {
            Self::Bottom => anchors.bottom.y,
            Self::Top => anchors.top.y,
            Self::Left(row) | Self::Right(row) => anchors.rows[row],
        };
        Vec2::new(x, y)
    }
}

/// 坐标可用设计像素或百分比；具体游戏提供内容区的尺寸和间距。
pub(crate) struct TableSeatAnchors<'a> {
    pub bottom: Vec2,
    pub top: Vec2,
    pub columns: Vec2,
    pub rows: &'a [f32],
}

#[derive(Clone, Copy)]
pub(crate) struct TableSeats {
    count: u8,
}

impl TableSeats {
    pub(crate) const fn new(count: u8) -> Self {
        Self { count }
    }

    pub(crate) fn seat(self, relative: u8) -> TableSeat {
        use TableSeat::{Bottom, Left, Right, Top};
        let seats: &[_] = match self.count {
            1 => &[Bottom],
            2 => &[Bottom, Top],
            3 => &[Bottom, Left(0), Right(0)],
            4 => &[Bottom, Left(0), Top, Right(0)],
            5 => &[Bottom, Left(0), Left(1), Right(1), Right(0)],
            6 => &[Bottom, Left(0), Left(1), Top, Right(1), Right(0)],
            7 => &[
                Bottom,
                Left(0),
                Left(1),
                Left(2),
                Right(2),
                Right(1),
                Right(0),
            ],
            8 => &[
                Bottom,
                Left(0),
                Left(1),
                Left(2),
                Top,
                Right(2),
                Right(1),
                Right(0),
            ],
            _ => unreachable!("table layouts support one to eight players"),
        };
        seats[usize::from(relative)]
    }

    pub(crate) fn side_rows(self) -> usize {
        usize::from((self.count.saturating_sub(1)) / 2)
    }

    pub(crate) fn relative(self, origin: u8, target: u8) -> u8 {
        (target + self.count - origin) % self.count
    }

    /// 将有空座的物理座次按当前玩家顺序均匀映射到此牌桌。
    pub(crate) fn occupied_slot(
        self,
        origin: u8,
        target: u8,
        occupied: impl Iterator<Item = u8>,
        capacity: u8,
    ) -> u8 {
        let ring = Self::new(capacity);
        let distance = ring.relative(origin, target);
        let mut rank = 0;
        let mut count = 0;
        for seat in occupied {
            rank += u16::from(ring.relative(origin, seat) < distance);
            count += 1;
        }
        ((rank * u16::from(self.count) + count / 2) / count.max(1)) as u8
    }
}
