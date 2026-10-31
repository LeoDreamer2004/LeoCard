use crate::app::{
    presentation::{ACCENT, DANGER, add_text},
    runtime::{ClientResource, UiAssets},
};
use bevy::prelude::*;
use leocard_protocol::{GameSnapshot, McGuessSnapshot};

use leocard_ui::UiComposer;

#[derive(Component)]
pub(super) struct GuessClockLabel;

fn guess_clock_label(remaining: Option<u16>) -> String {
    remaining.map_or_else(String::new, |seconds| format!("剩余 {seconds} 秒"))
}

fn guess_clock_color(remaining: Option<u16>) -> Color {
    if remaining.is_some_and(|seconds| seconds <= 5) {
        DANGER
    } else {
        ACCENT
    }
}

pub(super) fn add_guess_clock(
    commands: &mut UiComposer,
    parent: Entity,
    remaining: Option<u16>,
    assets: &UiAssets,
) {
    if remaining.is_none() {
        return;
    }
    let clock = add_text(
        commands,
        parent,
        guess_clock_label(remaining),
        20.0,
        guess_clock_color(remaining),
        assets,
    );
    commands.entity(clock).insert((
        GuessClockLabel,
        Node {
            width: px(120),
            flex_shrink: 0.0,
            ..default()
        },
    ));
}

pub(crate) fn only_guess_timer_changed(before: &McGuessSnapshot, after: &McGuessSnapshot) -> bool {
    if before.guess_seconds_remaining == after.guess_seconds_remaining {
        return false;
    }
    let mut normalized = before.clone();
    normalized.guess_seconds_remaining = after.guess_seconds_remaining;
    normalized == *after
}

pub(super) fn sync_guess_clock(
    client: Option<Res<ClientResource>>,
    mut labels: Query<(&mut Text, &mut TextColor), With<GuessClockLabel>>,
) {
    let remaining = client
        .as_deref()
        .and_then(|client| {
            client
                .0
                .model()
                .game_snapshot()
                .and_then(GameSnapshot::mc_guess)
        })
        .and_then(|game| game.guess_seconds_remaining);
    let expected = guess_clock_label(remaining);
    let color = guess_clock_color(remaining);
    for (mut label, mut text_color) in &mut labels {
        if label.0 != expected {
            label.0.clone_from(&expected);
        }
        if text_color.0 != color {
            text_color.0 = color;
        }
    }
}
