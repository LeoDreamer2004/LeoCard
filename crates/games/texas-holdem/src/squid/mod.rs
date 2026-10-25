mod appearance;
mod state;
#[cfg(test)]
mod tests;

pub use appearance::{DEEPSEEK_SQUID_VARIANTS, deepseek_squid_marker};
pub use state::{SquidAwardStatistics, SquidState, SquidTransfer};
