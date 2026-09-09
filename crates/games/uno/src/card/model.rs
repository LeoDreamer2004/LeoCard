use std::cmp::Ordering;
use std::fmt;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum UnoColor {
    Red,
    Yellow,
    Green,
    Blue,
    Pink,
    Teal,
    Orange,
    Purple,
}

impl UnoColor {
    pub const LIGHT: [Self; 4] = [Self::Red, Self::Yellow, Self::Green, Self::Blue];
    pub const DARK: [Self; 4] = [Self::Pink, Self::Teal, Self::Orange, Self::Purple];
    pub const ALL: [Self; 4] = Self::LIGHT;
    pub const ALL_SIDES: [Self; 8] = [
        Self::Red,
        Self::Yellow,
        Self::Green,
        Self::Blue,
        Self::Pink,
        Self::Teal,
        Self::Orange,
        Self::Purple,
    ];

    pub const fn is_light(self) -> bool {
        matches!(self, Self::Red | Self::Yellow | Self::Green | Self::Blue)
    }

    pub const fn is_dark(self) -> bool {
        !self.is_light()
    }
}

impl fmt::Display for UnoColor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Red => "红",
            Self::Yellow => "黄",
            Self::Green => "绿",
            Self::Blue => "蓝",
            Self::Pink => "粉",
            Self::Teal => "青",
            Self::Orange => "橙",
            Self::Purple => "紫",
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum UnoFace {
    Number(u8),
    DrawOne,
    DrawTwo,
    DrawFour,
    DrawFive,
    Reverse,
    Skip,
    SkipEveryone,
    Flip,
    DiscardAll,
    SwapOne,
    RefreshHand,
    ReverseDrawTwo,
    ReverseSkip,
    StackOne,
    StackTwo,
    Wild,
    WildDrawTwo,
    WildDrawFour,
    WildDrawColor,
    WildForceTrade,
    WildPassHands,
    WildPowerReverse,
    WildNoU,
    WildStackThree,
    WildStackNumber,
    WildReverseDrawFour,
    WildDrawSix,
    WildDrawTen,
    WildColorRoulette,
    DarkWild,
}

impl UnoFace {
    pub const fn score(self) -> u16 {
        match self {
            Self::Number(value) => value as u16,
            Self::DrawTwo
            | Self::DrawOne
            | Self::DrawFour
            | Self::DrawFive
            | Self::Reverse
            | Self::Skip
            | Self::SkipEveryone
            | Self::DiscardAll
            | Self::SwapOne
            | Self::RefreshHand
            | Self::ReverseDrawTwo
            | Self::ReverseSkip
            | Self::StackOne
            | Self::StackTwo => 20,
            Self::Wild
            | Self::DarkWild
            | Self::WildDrawFour
            | Self::WildForceTrade
            | Self::WildPassHands
            | Self::WildPowerReverse
            | Self::WildNoU
            | Self::WildStackThree
            | Self::WildStackNumber
            | Self::WildReverseDrawFour
            | Self::WildDrawSix
            | Self::WildDrawTen
            | Self::WildColorRoulette => 50,
            Self::Flip => 20,
            Self::WildDrawTwo | Self::WildDrawColor => 50,
        }
    }

    pub const fn flip_score(self) -> u16 {
        match self {
            Self::Number(value) => value as u16,
            Self::DrawOne | Self::DrawFive | Self::Reverse | Self::Skip | Self::Flip => 20,
            Self::SkipEveryone => 30,
            Self::Wild | Self::DarkWild => 40,
            Self::WildDrawTwo | Self::WildDrawColor => 50,
            _ => self.score(),
        }
    }

    pub const fn is_wild(self) -> bool {
        matches!(
            self,
            Self::Wild
                | Self::DarkWild
                | Self::WildDrawTwo
                | Self::WildDrawFour
                | Self::WildDrawColor
                | Self::WildForceTrade
                | Self::WildPassHands
                | Self::WildPowerReverse
                | Self::WildNoU
                | Self::WildStackThree
                | Self::WildStackNumber
                | Self::WildReverseDrawFour
                | Self::WildDrawSix
                | Self::WildDrawTen
                | Self::WildColorRoulette
        )
    }

    pub const fn is_swap_pack(self) -> bool {
        matches!(
            self,
            Self::SwapOne | Self::RefreshHand | Self::WildForceTrade | Self::WildPassHands
        )
    }

    pub const fn is_reverse_pack(self) -> bool {
        matches!(
            self,
            Self::ReverseDrawTwo | Self::ReverseSkip | Self::WildPowerReverse | Self::WildNoU
        )
    }

    pub const fn is_stack_pack(self) -> bool {
        matches!(
            self,
            Self::StackOne | Self::StackTwo | Self::WildStackThree | Self::WildStackNumber
        )
    }

    pub const fn is_extension(self) -> bool {
        self.is_swap_pack() || self.is_reverse_pack() || self.is_stack_pack()
    }

    pub const fn is_no_mercy(self) -> bool {
        matches!(
            self,
            Self::DrawFour
                | Self::SkipEveryone
                | Self::DiscardAll
                | Self::WildReverseDrawFour
                | Self::WildDrawSix
                | Self::WildDrawTen
                | Self::WildColorRoulette
        )
    }

    pub const fn draw_value(self) -> Option<u8> {
        match self {
            Self::DrawOne => Some(1),
            Self::DrawTwo => Some(2),
            Self::DrawFour | Self::WildReverseDrawFour => Some(4),
            Self::DrawFive => Some(5),
            Self::WildDrawTwo => Some(2),
            Self::WildDrawSix => Some(6),
            Self::WildDrawTen => Some(10),
            _ => None,
        }
    }
}

impl fmt::Display for UnoFace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Number(value) => value.fmt(f),
            Self::DrawOne => f.write_str("+1"),
            Self::DrawTwo => f.write_str("+2"),
            Self::DrawFour => f.write_str("+4"),
            Self::DrawFive => f.write_str("+5"),
            Self::Reverse => f.write_str("反转"),
            Self::Skip => f.write_str("跳过"),
            Self::SkipEveryone => f.write_str("全员禁手"),
            Self::Flip => f.write_str("翻面"),
            Self::DiscardAll => f.write_str("全部弃牌"),
            Self::SwapOne => f.write_str("交换一张"),
            Self::RefreshHand => f.write_str("刷新手牌"),
            Self::ReverseDrawTwo => f.write_str("反转摸二"),
            Self::ReverseSkip => f.write_str("反转禁手"),
            Self::StackOne => f.write_str("堆叠+1"),
            Self::StackTwo => f.write_str("堆叠+2"),
            Self::Wild => f.write_str("万能"),
            Self::DarkWild => f.write_str("暗面万能"),
            Self::WildDrawTwo => f.write_str("万能+2"),
            Self::WildDrawFour => f.write_str("万能+4"),
            Self::WildDrawColor => f.write_str("指定颜色摸牌"),
            Self::WildForceTrade => f.write_str("指定换手"),
            Self::WildPassHands => f.write_str("顺序传手"),
            Self::WildPowerReverse => f.write_str("强力反转"),
            Self::WildNoU => f.write_str("罚牌反弹"),
            Self::WildStackThree => f.write_str("万能堆叠+3"),
            Self::WildStackNumber => f.write_str("万能随机堆叠"),
            Self::WildReverseDrawFour => f.write_str("万能反转+4"),
            Self::WildDrawSix => f.write_str("万能+6"),
            Self::WildDrawTen => f.write_str("万能+10"),
            Self::WildColorRoulette => f.write_str("万能颜色轮盘"),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CardSide {
    color: Option<UnoColor>,
    face: UnoFace,
}

impl CardSide {
    pub const fn colored(color: UnoColor, face: UnoFace) -> Self {
        Self {
            color: Some(color),
            face,
        }
    }

    pub const fn wild(face: UnoFace) -> Self {
        Self { color: None, face }
    }

    pub const fn color(self) -> Option<UnoColor> {
        self.color
    }

    pub const fn face(self) -> UnoFace {
        self.face
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum UnoFlipSide {
    Light,
    Dark,
}

/// 一张物理牌。`copy` 区分同牌面的不同实体牌，从 0 开始。
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UnoCard {
    color: Option<UnoColor>,
    face: UnoFace,
    copy: u8,
    opposite: Option<CardSide>,
}

impl UnoCard {
    pub const fn number(color: UnoColor, value: u8, copy: u8) -> Self {
        assert!(value <= 9, "UNO number cards are 0..=9");
        assert!(copy < 2);
        Self {
            color: Some(color),
            face: UnoFace::Number(value),
            copy,
            opposite: None,
        }
    }

    pub const fn action(color: UnoColor, face: UnoFace, copy: u8) -> Self {
        assert!(matches!(
            face,
            UnoFace::DrawTwo
                | UnoFace::DrawOne
                | UnoFace::DrawFour
                | UnoFace::DrawFive
                | UnoFace::Reverse
                | UnoFace::Skip
                | UnoFace::SkipEveryone
                | UnoFace::Flip
                | UnoFace::DiscardAll
                | UnoFace::SwapOne
                | UnoFace::RefreshHand
                | UnoFace::ReverseDrawTwo
                | UnoFace::ReverseSkip
                | UnoFace::StackOne
                | UnoFace::StackTwo
        ));
        let copy_count = match face {
            UnoFace::SwapOne
            | UnoFace::RefreshHand
            | UnoFace::ReverseDrawTwo
            | UnoFace::ReverseSkip
            | UnoFace::StackOne
            | UnoFace::StackTwo => 1,
            UnoFace::DrawOne | UnoFace::DrawFive | UnoFace::Flip => 2,
            UnoFace::DrawFour | UnoFace::SkipEveryone => 2,
            UnoFace::DrawTwo | UnoFace::Reverse | UnoFace::Skip | UnoFace::DiscardAll => 3,
            _ => 0,
        };
        assert!(copy < copy_count);
        Self {
            color: Some(color),
            face,
            copy,
            opposite: None,
        }
    }

    pub const fn wild(face: UnoFace, copy: u8) -> Self {
        assert!(matches!(
            face,
            UnoFace::Wild
                | UnoFace::DarkWild
                | UnoFace::WildDrawTwo
                | UnoFace::WildDrawFour
                | UnoFace::WildDrawColor
                | UnoFace::WildForceTrade
                | UnoFace::WildPassHands
                | UnoFace::WildPowerReverse
                | UnoFace::WildNoU
                | UnoFace::WildStackThree
                | UnoFace::WildStackNumber
                | UnoFace::WildReverseDrawFour
                | UnoFace::WildDrawSix
                | UnoFace::WildDrawTen
                | UnoFace::WildColorRoulette
        ));
        let copy_count = match face {
            UnoFace::WildReverseDrawFour | UnoFace::WildColorRoulette => 8,
            _ => 4,
        };
        assert!(copy < copy_count);
        Self {
            color: None,
            face,
            copy,
            opposite: None,
        }
    }

    pub const fn paired(active: CardSide, opposite: CardSide, copy: u8) -> Self {
        Self {
            color: active.color,
            face: active.face,
            copy,
            opposite: Some(opposite),
        }
    }

    pub const fn color(self) -> Option<UnoColor> {
        self.color
    }

    pub const fn face(self) -> UnoFace {
        self.face
    }

    pub const fn copy(self) -> u8 {
        self.copy
    }

    pub const fn opposite(self) -> Option<CardSide> {
        self.opposite
    }

    pub const fn is_double_sided(self) -> bool {
        self.opposite.is_some()
    }

    pub const fn flipped(self) -> Self {
        match self.opposite {
            Some(opposite) => Self {
                color: opposite.color,
                face: opposite.face,
                copy: self.copy,
                opposite: Some(CardSide {
                    color: self.color,
                    face: self.face,
                }),
            },
            None => self,
        }
    }

    pub const fn public_face(self) -> Self {
        Self {
            color: self.color,
            face: self.face,
            copy: self.copy,
            opposite: None,
        }
    }

    pub const fn opposite_public_face(self) -> Option<Self> {
        match self.opposite {
            Some(opposite) => Some(Self {
                color: opposite.color,
                face: opposite.face,
                copy: self.copy,
                opposite: None,
            }),
            None => None,
        }
    }

    pub const fn score(self) -> u16 {
        if self.is_double_sided() {
            self.face.flip_score()
        } else {
            self.face.score()
        }
    }

    pub(crate) fn display_cmp(left: &Self, right: &Self) -> Ordering {
        (left.color, left.face, left.copy, left.opposite).cmp(&(
            right.color,
            right.face,
            right.copy,
            right.opposite,
        ))
    }
}

impl fmt::Display for UnoCard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.color {
            Some(color) => write!(f, "{color}{}", self.face),
            None => self.face.fmt(f),
        }
    }
}
