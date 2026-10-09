mod plugin;
mod state;
mod view;

pub(crate) use plugin::ConfirmationPlugin;
pub(crate) use state::{ConfirmationDialog, ConfirmationUiAction};
pub(crate) use view::render_confirmation;
