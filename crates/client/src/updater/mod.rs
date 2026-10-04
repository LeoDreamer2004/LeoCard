mod installer;
mod receipt;
#[cfg(test)]
mod tests;

pub(crate) use installer::{launch_installer, run_if_requested};
pub(crate) use receipt::completed_update;
