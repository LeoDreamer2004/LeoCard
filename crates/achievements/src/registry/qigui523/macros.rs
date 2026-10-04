macro_rules! qigui523 {
    ($id:literal, $title:literal, $tier:ident, $description:literal, $amount:expr, $target:expr) => {
        award!(
            "leocard:qigui523/",
            QiGui523,
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

pub(super) use qigui523;
