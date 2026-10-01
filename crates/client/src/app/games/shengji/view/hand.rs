use super::super::{
    ShengjiHandCardSelectionOverlay, ShengjiHandCardSlot, ShengjiHandCardVisual, ShengjiUiAction,
    ShengjiUiState,
};
use super::{
    ShengjiCardSize, add_shengji_dealer_badge, add_shengji_trump_stars, shengji_card_face,
    shengji_display_trump, shengji_hand_sort_trump, sort_shengji_cards,
};
use crate::app::presentation::{
    ACCENT, BORDER, CardAnimationState, PendingDealSound, PlayerMenuProfile, PlayerPortraitSpec,
    TEXT, TurnBorderAnimationKey, TurnBorderMaterial, add_player_portrait,
    add_turn_border_trace_with_radius, attach_start_game_seat_transition, hand_card_pose,
    shengji_hand_card_reveal, spawn_node,
};
use crate::app::presentation::{CardDragSelection, CardSize};
use crate::app::runtime::{AvatarImages, ClientResource, UiAssets};
use crate::app::shell::{SeatSide, UiAction};
use bevy::prelude::*;
use bevy::ui::{FocusPolicy, RelativeCursorPosition};
use leocard_protocol::{GameKind, PlayerId, ShengjiPhaseView, ShengjiPlayerState, ShengjiSnapshot};
use leocard_shengji::{ShengjiCard, ShengjiTrump};

/// 每位玩家相邻两张可见手牌的发牌间隔；对应全桌约 100ms 发一张牌。
const SHENGJI_LOCAL_DEAL_INTERVAL: f32 = 0.40;

pub(super) fn add_shengji_hand(
    commands: &mut Commands,
    hand_area: Entity,
    game: &ShengjiSnapshot,
    ui: &ShengjiUiState,
    assets: &UiAssets,
) {
    let hand = spawn_node(
        commands,
        hand_area,
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            right: px(0),
            bottom: px(0),
            height: px(120),
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::NoWrap,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::FlexEnd,
            ..default()
        },
        None,
    );
    let mut cards = game.your_hand.clone();
    if let Some(failure) = game
        .throw_failure
        .as_ref()
        .filter(|failure| failure.player == game.you)
    {
        cards.retain(|card| !failure.attempted.contains(card));
    }
    let display_trump = shengji_display_trump(game);
    sort_shengji_cards(&mut cards, shengji_hand_sort_trump(game));
    let hand_len = cards.len();
    let last = cards.len().saturating_sub(1);
    for (index, card) in cards.into_iter().enumerate() {
        add_shengji_hand_card(
            commands,
            hand,
            card,
            index,
            hand_len,
            index == last,
            ui.selected.contains(&card),
            ui.card_animations.get(&card).copied().unwrap_or_default(),
            display_trump,
            assets,
        );
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "the hand-card builder keeps selection, layout, and asset inputs explicit"
)]
fn add_shengji_hand_card(
    commands: &mut Commands,
    parent: Entity,
    card: ShengjiCard,
    index: usize,
    hand_len: usize,
    is_last: bool,
    selected: bool,
    animation: CardAnimationState,
    trump: Option<ShengjiTrump>,
    assets: &UiAssets,
) {
    let (width, height) = CardSize::ShengjiHand.dimensions();
    let image = shengji_card_face(card, assets);
    let initial_pose = hand_card_pose(
        index,
        hand_len,
        animation.face_hover_amount,
        animation.selected_amount,
        animation.deal_elapsed,
        animation.dealing,
        CardSize::ShengjiHand,
    );
    let initial_glow =
        (animation.face_hover_amount * 0.72 + animation.selected_amount * 0.72).clamp(0.0, 1.0);
    let button = commands
        .spawn((
            Button,
            ShengjiHandCardSlot { card, index },
            UiAction::Shengji(ShengjiUiAction::ToggleCard),
            RelativeCursorPosition::default(),
            Node {
                width: px(if is_last {
                    width
                } else {
                    shengji_hand_card_reveal(hand_len)
                }),
                height: px(height),
                ..default()
            },
        ))
        .id();
    commands.entity(parent).add_child(button);
    let face = commands
        .spawn((
            ShengjiHandCardVisual {
                button,
                card,
                index,
                selected,
                hover_amount: animation.face_hover_amount,
                selected_amount: animation.selected_amount,
                deal_elapsed: animation.deal_elapsed,
                dealing: animation.dealing,
                hand_len,
            },
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                bottom: px(0),
                width: px(width),
                height: px(height),
                border: UiRect::all(px(2)),
                border_radius: BorderRadius::all(px(6)),
                ..default()
            },
            UiTransform {
                translation: initial_pose.translation,
                rotation: initial_pose.rotation,
                ..UiTransform::IDENTITY
            },
            ImageNode::new(image).with_color(Color::srgb(
                1.0,
                1.0 - initial_glow * 0.035,
                1.0 - initial_glow * 0.16,
            )),
            BorderColor::all(if selected { ACCENT } else { BORDER }),
            Outline::new(
                px(0.75 + initial_glow * 1.5),
                px(0),
                ACCENT.with_alpha(initial_glow * 0.92),
            ),
            BoxShadow::new(Color::NONE, px(0), px(0), px(0), px(3)),
            GlobalZIndex(index as i32 + 1),
            FocusPolicy::Pass,
        ))
        .id();
    commands.entity(button).add_child(face);
    add_shengji_trump_stars(commands, face, card, trump, ShengjiCardSize::Hand, assets);
    let overlay = commands
        .spawn((
            ShengjiHandCardSelectionOverlay { index },
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                top: px(0),
                width: percent(100),
                height: percent(100),
                border_radius: BorderRadius::all(px(5)),
                ..default()
            },
            BackgroundColor(Color::NONE),
            FocusPolicy::Pass,
        ))
        .id();
    commands.entity(face).add_child(overlay);
}

pub(crate) fn queue_shengji_deal_animations(
    client: Option<Res<ClientResource>>,
    mut ui: ResMut<ShengjiUiState>,
    assets: Res<UiAssets>,
    mut commands: Commands,
) {
    let Some(game) = client
        .as_deref()
        .and_then(|client| client.0.model().shengji_game())
    else {
        ui.observed_hand.clear();
        ui.card_animations.clear();
        return;
    };
    if ui.observed_hand.observe((game.match_id, game.hand_number)) {
        ui.selected.clear();
        ui.card_animations.clear();
    }
    let new_cards = game
        .your_hand
        .iter()
        .copied()
        .filter(|card| !ui.observed_hand.state.contains(card))
        .collect::<Vec<_>>();
    let intro_deal = ui.intro_deal_match == Some(game.match_id) && !new_cards.is_empty();
    if intro_deal {
        ui.intro_deal_match = None;
    }
    let animate_deal = intro_deal || matches!(game.phase, ShengjiPhaseView::Dealing { .. });
    for (index, card) in new_cards.into_iter().enumerate() {
        let delay = if animate_deal {
            index as f32 * SHENGJI_LOCAL_DEAL_INTERVAL
        } else {
            0.0
        };
        ui.card_animations.insert(
            card,
            CardAnimationState {
                deal_elapsed: if animate_deal { -delay } else { 0.24 },
                dealing: animate_deal,
                ..default()
            },
        );
        if animate_deal && !assets.audio.deal_sounds.is_empty() {
            commands.spawn(PendingDealSound {
                remaining: delay,
                variant: fastrand::usize(..assets.audio.deal_sounds.len()),
            });
        }
    }
    ui.observed_hand.state.clone_from(&game.your_hand);
    ui.card_animations
        .retain(|card, _| game.your_hand.contains(card));
}

pub(crate) fn animate_shengji_hand_cards(
    time: Res<Time>,
    drag: Res<CardDragSelection>,
    mut ui: ResMut<ShengjiUiState>,
    buttons: Query<&Interaction, With<Button>>,
    mut cards: Query<(
        &mut ShengjiHandCardVisual,
        &mut UiTransform,
        &mut Outline,
        &mut BoxShadow,
        &mut ImageNode,
        &mut BorderColor,
    )>,
) {
    let response = 1.0 - (-14.0 * time.delta_secs()).exp();
    let pulse = 0.72 + 0.28 * (time.elapsed_secs() * 7.0).sin();
    for (mut visual, mut transform, mut outline, mut shadow, mut image, mut border) in &mut cards {
        let Ok(interaction) = buttons.get(visual.button) else {
            continue;
        };
        let selected = ui.selected.contains(&visual.card);
        let hovered = if drag.active {
            visual.index == drag.current
        } else {
            matches!(*interaction, Interaction::Hovered | Interaction::Pressed)
        };
        let hover_target = f32::from(hovered);
        let selected_target = f32::from(selected);
        let transitioning = (hover_target - visual.hover_amount).abs() >= 0.001
            || (selected_target - visual.selected_amount).abs() >= 0.001
            || visual.selected != selected
            || visual.dealing;
        if !transitioning && !hovered && !selected {
            continue;
        }
        visual.selected = selected;
        visual.hover_amount += (hover_target - visual.hover_amount) * response;
        visual.selected_amount += (selected_target - visual.selected_amount) * response;
        if visual.dealing {
            visual.deal_elapsed += time.delta_secs();
            if visual.deal_elapsed >= 0.20 {
                visual.dealing = false;
            }
        }
        let next_animation = CardAnimationState {
            face_hover_amount: visual.hover_amount,
            selected_amount: visual.selected_amount,
            deal_elapsed: visual.deal_elapsed,
            dealing: visual.dealing,
            ..ui.card_animations
                .get(&visual.card)
                .copied()
                .unwrap_or_default()
        };
        ui.card_animations.insert(visual.card, next_animation);
        let glow = (visual.hover_amount * pulse + visual.selected_amount * 0.72).clamp(0.0, 1.0);
        let pose = hand_card_pose(
            visual.index,
            visual.hand_len,
            visual.hover_amount,
            visual.selected_amount,
            visual.deal_elapsed,
            visual.dealing,
            CardSize::ShengjiHand,
        );
        transform.translation = pose.translation;
        transform.rotation = pose.rotation;
        outline.width = px(0.75 + glow * 1.5);
        outline.color = ACCENT.with_alpha(glow * 0.92);
        image.color = Color::srgb(1.0, 1.0 - glow * 0.035, 1.0 - glow * 0.16);
        border.set_all(if selected { ACCENT } else { BORDER });
        if let Some(style) = shadow.0.first_mut() {
            style.color = ACCENT.with_alpha(glow * 0.58);
            style.spread_radius = px(glow * 2.0);
            style.blur_radius = px(2.0 + glow * 10.0);
        }
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "the self-panel builder combines player state with explicit visual resources"
)]
pub(super) fn add_shengji_self_panel(
    commands: &mut Commands,
    hand_area: Entity,
    player: &ShengjiPlayerState,
    game: &ShengjiSnapshot,
    assets: &UiAssets,
    avatars: &AvatarImages,
    turn_border_materials: &mut Assets<TurnBorderMaterial>,
    start_transition_active: bool,
    interaction_menu_open: Option<PlayerId>,
) {
    let panel = add_player_portrait(
        commands,
        hand_area,
        Node {
            position_type: PositionType::Absolute,
            left: px(12),
            bottom: px(12),
            width: px(96.0 * 1.17),
            height: px(76.0 * 1.17),
            ..default()
        },
        PlayerPortraitSpec {
            player: player.id,
            profile: PlayerMenuProfile {
                name: &player.name,
                avatar: player.avatar.and_then(|id| avatars.remote.get(&id)),
                reference_points: player.reference_points,
                completed_games: player.completed_games,
                game_profiles: &player.game_profiles,
            },
            side: SeatSide::Left,
            avatar_size: 52.0 * 1.17,
            auto_play: player.auto_play,
            menu_open: interaction_menu_open == Some(player.id),
            menu_above: true,
            name_color: TEXT,
        },
        assets,
    );
    attach_start_game_seat_transition(commands, panel.portrait, player.id, start_transition_active);
    if game.current_player == Some(player.id) {
        add_turn_border_trace_with_radius(
            commands,
            panel.avatar_ring,
            turn_border_materials,
            TurnBorderAnimationKey::new(GameKind::Shengji, game.match_id, player.id),
            52.0 * 1.17 * 0.2,
            52.0 * 1.17,
        );
    }
    if game.dealer == Some(player.id) {
        add_shengji_dealer_badge(commands, panel.portrait, SeatSide::Left, assets);
    }
}
