use crate::{FollowError, PlayError};
use std::error::Error;
use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ShengjiAutoPlayError {
    EmptyHand,
    NotEnoughCards { required: usize, actual: usize },
    InvalidLead(PlayError),
    InvalidFollow(FollowError),
}

impl fmt::Display for ShengjiAutoPlayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyHand => f.write_str("托管策略没有可出的牌"),
            Self::NotEnoughCards { required, actual } => {
                write!(f, "需要跟出 {required} 张牌，手中只有 {actual} 张")
            }
            Self::InvalidLead(error) => error.fmt(f),
            Self::InvalidFollow(error) => error.fmt(f),
        }
    }
}

impl Error for ShengjiAutoPlayError {}
