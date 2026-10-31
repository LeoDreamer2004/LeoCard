/// 玩家彩蛋
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlayerPersona {
    Yang120,
    SnowyWind,
    DreamerLeo,
    Cnofne,
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
        } else {
            Self::Regular
        }
    }
}

impl PlayerPersona {
    pub const fn notification_nickname(self) -> Option<&'static str> {
        match self {
            Self::Yang120 => Some("官儿"),
            Self::SnowyWind => Some("马眼"),
            Self::DreamerLeo => Some("Dev"),
            Self::Cnofne => Some("群主"),
            Self::Regular => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PlayerPersona;

    #[test]
    fn names_match_substrings_in_a_stable_order() {
        for (name, persona, nickname) in [
            ("MyYang123", PlayerPersona::Yang120, Some("官儿")),
            ("SnowyFox", PlayerPersona::SnowyWind, Some("马眼")),
            ("LeoDreamer", PlayerPersona::DreamerLeo, Some("Dev")),
            ("ofne123", PlayerPersona::Cnofne, Some("群主")),
            ("Alice", PlayerPersona::Regular, None),
            ("YangDream", PlayerPersona::Yang120, Some("官儿")),
        ] {
            assert_eq!(PlayerPersona::from(name), persona);
            assert_eq!(persona.notification_nickname(), nickname);
        }
    }
}
