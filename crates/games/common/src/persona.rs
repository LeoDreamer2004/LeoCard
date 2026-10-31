/// 玩家彩蛋
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlayerPersona {
    DreamerLeo,
    Cnofne,
    Yang120,
    SnowyWind,
    Wpy03,
    Regular,
}

impl From<&str> for PlayerPersona {
    fn from(name: &str) -> Self {
        let lower = name.to_lowercase();
        if lower.contains("dream") {
            Self::DreamerLeo
        } else if lower.contains("ofne") {
            Self::Cnofne
        } else if lower.contains("yang") {
            Self::Yang120
        } else if lower.contains("snowy") {
            Self::SnowyWind
        } else if lower.contains("wpy") {
            Self::Wpy03
        } else {
            Self::Regular
        }
    }
}
