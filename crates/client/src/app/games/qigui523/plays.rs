use super::*;
use leocard_protocol::{GamePhaseView, PlayerPublicState, PublicPlay, PublicPlayRecord};
use leocard_protocol::{PlayerId, QiGui523Snapshot};

pub(super) fn add_round_play_for_optional_player(
    commands: &mut Commands,
    parent: Entity,
    side: SeatSide,
    game: &QiGui523Snapshot,
    player: Option<&PlayerPublicState>,
    play_effect: Option<&ActivePlayEffect>,
    last_play: Option<&(PlayerId, PublicPlay)>,
    assets: &UiAssets,
) {
    let justify_content = match side {
        SeatSide::Left => JustifyContent::FlexStart,
        SeatSide::Top => JustifyContent::Center,
        SeatSide::Right => JustifyContent::FlexEnd,
    };
    let play = spawn_round_play_container(commands, parent, justify_content);
    if let Some(player) = player {
        add_round_play(
            commands,
            play,
            game,
            player.id,
            play_effect,
            last_play,
            assets,
        );
    }
}

pub(super) fn spawn_round_play_container(
    commands: &mut Commands,
    parent: Entity,
    justify_content: JustifyContent,
) -> Entity {
    spawn_node(
        commands,
        parent,
        Node {
            width: px(148),
            min_width: px(148),
            max_width: px(148),
            min_height: px(116),
            flex_shrink: 0.0,
            position_type: PositionType::Relative,
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::NoWrap,
            align_items: AlignItems::Center,
            justify_content,
            ..default()
        },
        None,
    )
}

pub(super) fn add_round_play(
    commands: &mut Commands,
    parent: Entity,
    game: &QiGui523Snapshot,
    player: PlayerId,
    play_effect: Option<&ActivePlayEffect>,
    last_play: Option<&(PlayerId, PublicPlay)>,
    assets: &UiAssets,
) {
    if matches!(&game.phase, GamePhaseView::Finished { .. })
        && game
            .players
            .iter()
            .find(|state| state.id == player)
            .is_some_and(|state| state.ready)
    {
        add_text(commands, parent, "准备", 24.0, READY, assets);
        return;
    }
    let is_current_player = matches!(&game.phase, GamePhaseView::Playing)
        && game
            .trick
            .as_ref()
            .is_some_and(|trick| trick.current_player == player);
    if is_current_player && turn_clock_visible(game, player) {
        add_turn_clock(
            commands,
            parent,
            game.turn_timer.filter(|timer| timer.player == player),
            assets,
        );
    }
    if is_current_player {
        return;
    }
    let cards = if matches!(&game.phase, GamePhaseView::Finished { .. }) {
        last_play.and_then(|(record_player, play)| {
            (*record_player == player).then_some(play.cards.as_slice())
        })
    } else {
        game.trick.as_ref().and_then(|trick| {
            trick.records.iter().rev().find_map(|record| match record {
                PublicPlayRecord::Played {
                    player: record_player,
                    play,
                } if *record_player == player => Some(play.cards.as_slice()),
                PublicPlayRecord::Passed {
                    player: record_player,
                } if *record_player == player => Some(&[][..]),
                _ => None,
            })
        })
    };
    let Some(cards) = cards.filter(|cards| !cards.is_empty()) else {
        if !matches!(&game.phase, GamePhaseView::Finished { .. }) {
            add_text(commands, parent, "不出", 22.0, MUTED, assets);
        }
        return;
    };
    let sequence_style = play_effect
        .filter(|effect| effect.player == player && effect.play.cards.len() == cards.len())
        .and_then(|effect| sequence_effect_style(&effect.play.kind));
    let mut displayed_cards = cards.to_vec();
    sort_cards_high_to_low(&mut displayed_cards);
    let last_card = displayed_cards.len().saturating_sub(1);
    let (card_width, card_height) = CardSize::Seat.dimensions();
    let cards_width = card_width + TABLE_CARD_REVEAL * last_card as f32;
    let card_group = spawn_node(
        commands,
        parent,
        Node {
            width: px(cards_width),
            min_width: px(cards_width),
            max_width: px(cards_width),
            height: px(card_height),
            min_height: px(card_height),
            max_height: px(card_height),
            flex_shrink: 0.0,
            position_type: PositionType::Relative,
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::NoWrap,
            ..default()
        },
        None,
    );
    for (index, card) in displayed_cards.iter().enumerate() {
        let card_entity = add_card_image(
            commands,
            card_group,
            *card,
            CardSize::Seat,
            index,
            index == last_card,
            sequence_style.is_some(),
            assets,
        );
        if sequence_style.is_some() {
            commands.entity(card_entity).insert((
                SequenceEffectCard { index },
                UiTransform::IDENTITY,
                FocusPolicy::Pass,
            ));
        }
    }
    if let Some((label, color, motif)) = sequence_style {
        add_sequence_play_decoration(
            commands,
            card_group,
            displayed_cards.len(),
            label,
            color,
            motif,
            assets,
        );
    }
}
