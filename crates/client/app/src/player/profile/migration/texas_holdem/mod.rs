mod archive;
#[cfg(test)]
mod tests;

pub(super) use archive::{
    V3_PROFILE_HEADER, V3TexasHoldemStats, V4_PROFILE_HEADER, decode_v3, decode_v4,
};
