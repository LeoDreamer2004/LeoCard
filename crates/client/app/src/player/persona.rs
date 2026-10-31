/// 玩家彩蛋
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlayerPersona {
    Yang120,
    SnowyWind,
    DreamerLeo,
    Cnofne,
    Wpy03,
    Regular,
}

impl From<&str> for PlayerPersona {
    fn from(name: &str) -> Self {
        if name.contains("Yang") {
            Self::Yang120
        } else if name.contains("Snowy") {
            Self::SnowyWind
        } else if name.contains("Dream") {
            Self::DreamerLeo
        } else if name.contains("ofne") {
            Self::Cnofne
        } else if name.contains("wpy") {
            Self::Wpy03
        } else {
            Self::Regular
        }
    }
}
