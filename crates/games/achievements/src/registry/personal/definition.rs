macro_rules! personal {
    ($id:literal, $title:literal, $tier:ident, $description:literal, $amount:expr, $target:expr) => {
        crate::achievement_award!(
            "leocard:personal/",
            Personal,
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

pub(super) use personal;
