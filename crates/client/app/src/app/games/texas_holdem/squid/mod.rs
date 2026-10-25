mod appearance;
mod state;
#[cfg(test)]
mod tests;
mod view;

pub(super) use state::SquidTableState;
pub(super) use view::{add_squid_markers, animate_squid_markers};
