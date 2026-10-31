macro_rules! fan {
    ($id:literal, $title:literal, $tier:ident, $fan:ident, $description:literal) => {
        fan!($id, $title, $tier, $fan, $description, 1)
    };
    ($id:literal, $title:literal, $tier:ident, $fan:ident, $description:literal, $target:expr) => {
        award!(
            "leocard:mahjong/",
            Mahjong,
            $id,
            $title,
            $tier,
            $description,
            |event| u64::from(mahjong_fan(event, MahjongFan::$fan)),
            $target,
            Lifetime
        )
    };
}

pub(super) use fan;
