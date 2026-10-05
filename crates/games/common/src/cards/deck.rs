use std::{collections::HashSet, error::Error, fmt, hash::Hash};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeckError {
    InvalidSize { expected: usize, actual: usize },
    InvalidContents,
}

impl fmt::Display for DeckError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSize { expected, actual } => {
                write!(f, "牌堆应为 {expected} 张，实际为 {actual}")
            }
            Self::InvalidContents => f.write_str("牌堆有缺牌、重复牌或非法牌"),
        }
    }
}

impl Error for DeckError {}

/// 按物理牌身份校验完整牌堆；顺序可以不同，每张物理牌必须恰好出现一次。
/// 多副牌或同牌面的副本应由调用方的牌类型区分。
pub fn validate_deck<Card: Eq + Hash>(deck: &[Card], expected: &[Card]) -> Result<(), DeckError> {
    if deck.len() != expected.len() {
        return Err(DeckError::InvalidSize {
            expected: expected.len(),
            actual: deck.len(),
        });
    }
    let actual: HashSet<_> = deck.iter().collect();
    let expected: HashSet<_> = expected.iter().collect();
    if actual.len() != deck.len() || actual != expected {
        return Err(DeckError::InvalidContents);
    }
    Ok(())
}
