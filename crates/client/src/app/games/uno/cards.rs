use super::*;
use leocard_protocol::{UnoPhaseView, UnoSnapshot};
use leocard_uno::{UnoCard, UnoColor, UnoFace, UnoPendingDrawKind};

pub fn uno_card_handle(assets: &UiAssets, card: UnoCard) -> Handle<Image> {
    assets
        .games
        .uno_cards
        .get(&(card.color(), card.face()))
        .cloned()
        .expect("所有 UNO 牌面都应预加载")
}

pub(super) fn uno_card_is_playable(game: &UnoSnapshot, card: UnoCard) -> bool {
    if game.current_player != Some(game.you)
        || game.pending_swap.is_some()
        || game.current_color.is_none()
        || !matches!(game.phase, UnoPhaseView::Playing)
        || game
            .your_drawn_card
            .is_some_and(|drawn_card| drawn_card != card)
    {
        return false;
    }
    let own_skips = game
        .players
        .iter()
        .find(|player| player.id == game.you)
        .map_or(0, |player| player.skipped_turns);
    if own_skips > 0 {
        return false;
    }
    if game.pending_skip > 0 {
        return if game.rules.is_flip() {
            game.rules.flip.action_stacking
                && card.face() == game.discard_top.face()
                && matches!(card.face(), UnoFace::Skip | UnoFace::SkipEveryone)
        } else {
            game.rules.action_stacking
                && matches!(card.face(), UnoFace::Skip | UnoFace::ReverseSkip)
        };
    }
    if game.pending_kind.is_some() {
        if game.rules.is_no_mercy() {
            return match game.pending_kind {
                Some(UnoPendingDrawKind::NoMercy(minimum)) => card
                    .face()
                    .draw_value()
                    .is_some_and(|value| value >= minimum),
                _ => false,
            };
        }
        if game.rules.is_flip() {
            if !game.rules.flip.action_stacking {
                return false;
            }
            return matches!(
                (game.pending_kind, card.face()),
                (
                    Some(UnoPendingDrawKind::FlipDrawOne),
                    UnoFace::DrawOne | UnoFace::WildDrawTwo
                ) | (
                    Some(UnoPendingDrawKind::FlipWildDrawTwo),
                    UnoFace::WildDrawTwo
                ) | (Some(UnoPendingDrawKind::FlipDrawFive), UnoFace::DrawFive)
                    | (
                        Some(UnoPendingDrawKind::FlipWildDrawColor),
                        UnoFace::WildDrawColor
                    )
            );
        }
        if !game.rules.action_stacking {
            return false;
        }
        return match (game.pending_kind, card.face()) {
            (Some(UnoPendingDrawKind::DrawTwo), UnoFace::DrawTwo | UnoFace::ReverseDrawTwo)
            | (Some(UnoPendingDrawKind::WildDrawFour), UnoFace::WildDrawFour) => true,
            (_, UnoFace::WildNoU) => true,
            (_, UnoFace::StackOne | UnoFace::StackTwo) => card.color() == game.current_color,
            (_, UnoFace::WildStackThree | UnoFace::WildStackNumber) => true,
            (Some(UnoPendingDrawKind::DrawTwo), UnoFace::WildDrawFour) => true,
            _ => false,
        };
    }
    card.face().is_wild()
        || card.color() == game.current_color
        || uno_faces_match(card.face(), game.discard_top.face())
}

fn uno_faces_match(left: UnoFace, right: UnoFace) -> bool {
    if left == right {
        return !matches!(left, UnoFace::StackOne | UnoFace::StackTwo);
    }
    matches!(
        (left, right),
        (UnoFace::ReverseDrawTwo, UnoFace::Reverse | UnoFace::DrawTwo)
            | (UnoFace::Reverse | UnoFace::DrawTwo, UnoFace::ReverseDrawTwo)
            | (UnoFace::ReverseSkip, UnoFace::Reverse | UnoFace::Skip)
            | (UnoFace::Reverse | UnoFace::Skip, UnoFace::ReverseSkip)
    )
}

pub fn uno_pair_for_selection(game: &UnoSnapshot, selected: UnoCard) -> Option<UnoCard> {
    let jump_in = if game.rules.is_flip() {
        game.rules.flip.jump_in
    } else {
        game.rules.is_classic() && game.rules.jump_in
    };
    if !jump_in
        || game.uno_declared.contains(&game.you)
        || selected.color().is_none()
        || selected.face().is_extension()
    {
        return None;
    }
    game.your_hand.iter().copied().find(|card| {
        *card != selected && card.color() == selected.color() && card.face() == selected.face()
    })
}

pub fn toggle_uno_selection(
    game: Option<&UnoSnapshot>,
    selected: &mut HashSet<UnoCard>,
    card: UnoCard,
) {
    if selected.remove(&card) {
        return;
    }
    if selected.len() == 1 {
        let first = *selected.iter().next().unwrap();
        if game.is_some_and(|game| uno_pair_for_selection(game, first) == Some(card)) {
            selected.insert(card);
            return;
        }
    }
    selected.clear();
    selected.insert(card);
}

pub fn uno_ui_color(color: UnoColor) -> Color {
    match color {
        UnoColor::Red => Color::srgb(0.91, 0.18, 0.16),
        UnoColor::Yellow => Color::srgb(0.96, 0.72, 0.08),
        UnoColor::Green => Color::srgb(0.10, 0.67, 0.28),
        UnoColor::Blue => Color::srgb(0.08, 0.42, 0.86),
        UnoColor::Pink => Color::srgb(0.91, 0.24, 0.58),
        UnoColor::Teal => Color::srgb(0.06, 0.62, 0.62),
        UnoColor::Orange => Color::srgb(0.96, 0.43, 0.10),
        UnoColor::Purple => Color::srgb(0.48, 0.25, 0.76),
    }
}

pub fn uno_should_show_reverse_effect(card: UnoCard, play_index: u8, play_count: u8) -> bool {
    play_index == 0
        && match card.face() {
            UnoFace::Reverse => play_count % 2 == 1,
            UnoFace::ReverseDrawTwo
            | UnoFace::ReverseSkip
            | UnoFace::WildPowerReverse
            | UnoFace::WildNoU
            | UnoFace::WildReverseDrawFour => true,
            _ => false,
        }
}

/// UNO 手牌沿用其他游戏的柔和抬升、渐变描边与阴影，不用突兀的离散跳变。
pub fn animate_uno_hand_cards(
    time: Res<Time>,
    mut ui: ResMut<UiState>,
    buttons: Query<&Interaction, With<Button>>,
    mut cards: Query<
        (
            &mut UnoHandCardVisual,
            &mut UiTransform,
            &mut Outline,
            &mut BoxShadow,
            &mut BorderColor,
        ),
        Without<UnoFlipCard>,
    >,
) {
    let response = 1.0 - (-14.0 * time.delta_secs()).exp();
    let pulse = 0.76 + 0.24 * (time.elapsed_secs() * 6.5).sin();
    for (mut visual, mut transform, mut outline, mut shadow, mut border) in &mut cards {
        let selected = ui.uno.selected.contains(&visual.card);
        let hovered = buttons.get(visual.button).is_ok_and(|interaction| {
            matches!(*interaction, Interaction::Hovered | Interaction::Pressed)
        });
        let hover_target = f32::from(hovered);
        let selected_target = f32::from(selected);
        visual.hover_amount += (hover_target - visual.hover_amount) * response;
        visual.selected_amount += (selected_target - visual.selected_amount) * response;
        visual.selected = selected;

        let glow = (visual.selected_amount * (0.76 + pulse * 0.24)).clamp(0.0, 1.0);
        transform.translation = Val2::px(
            0.0,
            -(visual.hover_amount * 10.0 + visual.selected_amount * 22.0),
        );
        transform.scale = Vec2::splat(1.0 + visual.hover_amount * 0.025);
        outline.width = px(glow * 2.3);
        outline.color = ACCENT.with_alpha(glow * 0.9);
        border.set_all(ACCENT.with_alpha(visual.selected_amount));
        if let Some(style) = shadow.0.first_mut() {
            style.color = if visual.selected_amount > 0.01 {
                ACCENT.with_alpha(glow * 0.5)
            } else {
                Color::BLACK.with_alpha(0.42)
            };
            style.spread_radius = px(glow * 1.6);
            style.blur_radius = px(5.0 + glow * 8.0);
        }
        ui.uno.card_animations.insert(
            visual.card,
            CardAnimationState {
                face_hover_amount: visual.hover_amount,
                selected_amount: visual.selected_amount,
                ..default()
            },
        );
    }
}

pub fn animate_uno_swap_target_panels(
    time: Res<Time>,
    mut panels: Query<(
        &UnoSwapTargetPanel,
        &mut BackgroundColor,
        &mut Outline,
        &mut BoxShadow,
    )>,
) {
    let pulse = 0.5 + 0.5 * (time.elapsed_secs() * 4.8).sin();
    for (target, mut background, mut outline, mut shadow) in &mut panels {
        if target.selected {
            background.0 = Color::BLACK.with_alpha(0.78);
            outline.width = px(2.4);
            outline.color = ACCENT.with_alpha(0.96);
        } else {
            background.0 = ACCENT.mix(&PANEL, 0.42 + pulse * 0.12).with_alpha(0.97);
            outline.width = px(2.0 + pulse * 0.8);
            outline.color = ACCENT.with_alpha(0.68 + pulse * 0.28);
        }
        if let Some(style) = shadow.0.first_mut() {
            style.color = ACCENT.with_alpha(if target.selected {
                0.42
            } else {
                0.24 + pulse * 0.22
            });
            style.blur_radius = px(if target.selected {
                11.0
            } else {
                8.0 + pulse * 6.0
            });
            style.spread_radius = px(1.0 + pulse * 1.5);
        }
    }
}
