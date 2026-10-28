macro_rules! doudizhu {
    ($id:literal, $title:literal, $tier:ident, $description:literal, $amount:expr) => {
        doudizhu!($id, $title, $tier, $description, $amount, 1)
    };
    ($id:literal, $title:literal, $tier:ident, $description:literal, $amount:expr, $target:expr) => {
        award!(
            "leocard:doudizhu/",
            Doudizhu,
            $id,
            $title,
            $tier,
            $description,
            $amount,
            $target,
            Lifetime
        )
    };
}

pub(super) use doudizhu;
