use super::*;
use leocard_qigui523::QiGuiCard;

#[derive(Clone, Copy)]
pub enum CardSize {
    Hand,
    Seat,
    Score,
    TableScore,
    FinishedHand,
}

pub fn sort_cards_high_to_low(cards: &mut [QiGuiCard]) {
    cards.sort_by(|left, right| {
        right
            .rank()
            .strength()
            .cmp(&left.rank().strength())
            .then_with(|| right.suit().strength().cmp(&left.suit().strength()))
            .then_with(|| right.deck().cmp(&left.deck()))
    });
}

impl CardSize {
    pub fn dimensions(self) -> (f32, f32) {
        match self {
            Self::Hand => (76.0, 103.0),
            Self::Seat => (72.0, 98.0),
            Self::Score => (36.0, 49.0),
            Self::TableScore => (28.0, 38.0),
            Self::FinishedHand => (43.2, 58.8),
        }
    }
}

pub(super) struct HandCardSpec {
    pub(super) card: QiGuiCard,
    pub(super) selected: bool,
    pub(super) animation: CardAnimationState,
    pub(super) index: usize,
    pub(super) hand_len: usize,
    pub(super) is_last: bool,
}

pub(super) fn add_card_button(
    commands: &mut Commands,
    parent: Entity,
    spec: HandCardSpec,
    assets: &UiAssets,
) {
    let HandCardSpec {
        card,
        selected,
        animation,
        index,
        hand_len,
        is_last,
    } = spec;
    let (width, height) = CardSize::Hand.dimensions();
    let image = assets
        .games
        .cards
        .get(&(card.rank(), card.suit()))
        .expect("all valid card faces are preloaded")
        .clone();
    let initial_pose = hand_card_pose(
        index,
        hand_len,
        animation.face_hover_amount,
        animation.selected_amount,
        animation.deal_elapsed,
        animation.dealing,
    );
    let initial_glow =
        (animation.face_hover_amount * 0.72 + animation.selected_amount * 0.72).clamp(0.0, 1.0);
    let button = commands
        .spawn((
            Button,
            UiAction::ToggleCard,
            HandCardSlot {
                card,
                index,
                is_last,
                hover_amount: animation.slot_hover_amount,
            },
            RelativeCursorPosition::default(),
            Node {
                width: px(if is_last { width } else { HAND_CARD_REVEAL }),
                height: px(height),
                ..default()
            },
        ))
        .id();
    commands.entity(parent).add_child(button);

    let card_face = commands
        .spawn((
            HandCardVisual {
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
    commands.entity(button).add_child(card_face);
    let overlay = commands
        .spawn((
            HandCardSelectionOverlay { index },
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
    commands.entity(card_face).add_child(overlay);
}
