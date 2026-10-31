use super::material::EntranceVeil;
use super::state::{EntranceAnimation, LobbyEntranceState};
use crate::app::runtime::UiAssets;
use bevy::picking::Pickable;
use bevy::prelude::*;
use leocard_game_common::PlayerId;
use leocard_protocol::ChatEmoji;

#[derive(Component)]
pub(crate) struct LobbySeatedAvatar(pub PlayerId);

#[derive(Component)]
pub(super) struct EntranceOverlay {
    pub sequence: u64,
    pub veil: Handle<EntranceVeil>,
}

#[derive(Component)]
pub(super) struct EntranceHeading;

#[derive(Component)]
pub(super) struct EntranceText;

#[derive(Component)]
pub(super) struct EntranceParty(pub f32);

#[derive(Component)]
pub(super) struct EntranceParticle {
    pub origin: Vec2,
    pub velocity: Vec2,
    pub delay: f32,
    pub spin: f32,
    pub color: Color,
}

pub(super) fn sync_entrance_overlay(
    mut commands: Commands,
    state: Res<LobbyEntranceState>,
    assets: Res<UiAssets>,
    mut materials: ResMut<Assets<EntranceVeil>>,
    overlays: Query<(Entity, &EntranceOverlay)>,
) {
    let sequence = state.active.map(|active| active.sequence);
    let mut found = false;
    for (entity, overlay) in &overlays {
        if Some(overlay.sequence) == sequence {
            found = true;
        } else {
            commands.entity(entity).despawn();
        }
    }
    if let Some(active) = state.active
        && !found
    {
        spawn_entrance(&mut commands, active, &assets, &mut materials);
    }
}

fn spawn_entrance(
    commands: &mut Commands,
    active: EntranceAnimation,
    assets: &UiAssets,
    materials: &mut Assets<EntranceVeil>,
) {
    let veil = materials.add(EntranceVeil {
        spotlight: Vec4::new(-10_000.0, -10_000.0, 0.0, 0.0),
    });
    let root = commands
        .spawn((
            EntranceOverlay {
                sequence: active.sequence,
                veil: veil.clone(),
            },
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                top: px(0),
                width: percent(100),
                height: percent(100),
                ..default()
            },
            GlobalZIndex(30_000),
            Pickable {
                should_block_lower: true,
                is_hoverable: true,
            },
            MaterialNode(veil),
        ))
        .id();
    spawn_confetti(commands, root, active.sequence);
    let heading = commands
        .spawn((
            EntranceHeading,
            Node {
                position_type: PositionType::Absolute,
                top: percent(13),
                width: percent(100),
                height: px(112),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                column_gap: px(24),
                ..default()
            },
            UiTransform::IDENTITY,
            Pickable::IGNORE,
            ZIndex(2),
        ))
        .id();
    commands.entity(root).add_child(heading);
    let party = assets.social.chat_emojis[ChatEmoji::Party.index()].clone();
    for side in [-1.0, 1.0] {
        if side > 0.0 {
            let text = commands
                .spawn((
                    EntranceText,
                    Text::new("恭迎大官入场"),
                    TextFont::from_font_size(54.0).with_font(assets.font.clone()),
                    TextColor(Color::srgb(1.0, 0.78, 0.22)),
                    TextShadow {
                        offset: Vec2::new(0.0, 3.0),
                        color: Color::srgba(0.35, 0.12, 0.01, 0.8),
                    },
                    Node {
                        flex_shrink: 0.0,
                        ..default()
                    },
                    Pickable::IGNORE,
                ))
                .id();
            commands.entity(heading).add_child(text);
        }
        let image = commands
            .spawn((
                EntranceParty(side),
                Node {
                    width: px(82),
                    height: px(82),
                    flex_shrink: 0.0,
                    ..default()
                },
                ImageNode {
                    image: party.clone(),
                    flip_x: side > 0.0,
                    ..default()
                },
                UiTransform::IDENTITY,
                Pickable::IGNORE,
            ))
            .id();
        commands.entity(heading).add_child(image);
    }
}

fn spawn_confetti(commands: &mut Commands, root: Entity, seed: u64) {
    let mut random = fastrand::Rng::with_seed(seed);
    let colors = [
        Color::srgb(1.0, 0.8, 0.23),
        Color::srgb(1.0, 0.36, 0.53),
        Color::srgb(0.36, 0.75, 1.0),
        Color::srgb(0.75, 0.53, 1.0),
    ];
    for side in [-1.0, 1.0] {
        for index in 0..24 {
            let color = colors[index % colors.len()];
            let particle = commands
                .spawn((
                    EntranceParticle {
                        origin: Vec2::new(side * 238.0, 62.0),
                        velocity: Vec2::new(
                            -side * (45.0 + random.f32() * 220.0),
                            -160.0 - random.f32() * 210.0,
                        ),
                        delay: 0.16 + random.f32() * 0.26,
                        spin: (random.f32() - 0.5) * 14.0,
                        color,
                    },
                    Node {
                        position_type: PositionType::Absolute,
                        left: percent(50),
                        top: percent(13),
                        width: px(4.0 + random.f32() * 4.0),
                        height: px(7.0 + random.f32() * 6.0),
                        border_radius: BorderRadius::all(px(1)),
                        ..default()
                    },
                    BackgroundColor(color.with_alpha(0.0)),
                    UiTransform::IDENTITY,
                    Pickable::IGNORE,
                    ZIndex(1),
                ))
                .id();
            commands.entity(root).add_child(particle);
        }
    }
}
