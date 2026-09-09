use super::RuntimePlugin;
use crate::app::games::GamesPlugin;
use crate::app::presentation::PresentationPlugin;
use crate::app::shell::ShellPlugin;
use bevy::prelude::*;

pub(crate) fn launch() {
    App::new()
        .add_plugins((RuntimePlugin, PresentationPlugin, ShellPlugin, GamesPlugin))
        .run();
}
