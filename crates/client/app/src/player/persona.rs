/// 跨界面复用的玩家彩蛋身份，匹配顺序固定，普通玩家保留自己的名称。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlayerPersona {
    Yang,
    Snowy,
    Dream,
    Ofne,
    Regular,
}

impl From<&str> for PlayerPersona {
    fn from(name: &str) -> Self {
        if name.contains("Yang") {
            Self::Yang
        } else if name.contains("Snowy") {
            Self::Snowy
        } else if name.contains("Dream") {
            Self::Dream
        } else if name.contains("ofne") {
            Self::Ofne
        } else {
            Self::Regular
        }
    }
}

impl PlayerPersona {
    pub const fn nickname(self) -> Option<&'static str> {
        match self {
            Self::Yang => Some("官儿"),
            Self::Snowy => Some("马眼"),
            Self::Dream => Some("Dev"),
            Self::Ofne => Some("群主"),
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
            ("MyYang123", PlayerPersona::Yang, Some("官儿")),
            ("SnowyFox", PlayerPersona::Snowy, Some("马眼")),
            ("LeoDreamer", PlayerPersona::Dream, Some("Dev")),
            ("ofne123", PlayerPersona::Ofne, Some("群主")),
            ("Alice", PlayerPersona::Regular, None),
            ("YangDream", PlayerPersona::Yang, Some("官儿")),
        ] {
            assert_eq!(PlayerPersona::from(name), persona);
            assert_eq!(persona.nickname(), nickname);
        }
    }
}
