use super::{
    TEXAS_PORTRAIT_WIDTH, TexasChipTableState, TexasInitialDeal, TexasTableLayout, TexasUiState,
    add_role_tokens, add_texas_actions, add_texas_card, add_texas_chip_popup, add_texas_win_rate,
};
use crate::app::presentation::{
    PendingDealSound, PlayerMenuProfile, PlayerPortraitSpec, SeatSide, TEXT, TablePieceDeal,
    TurnBorderAnimationKey, TurnBorderMaterial, add_player_portrait,
    add_turn_border_trace_with_radius, attach_start_game_seat_transition,
};
use crate::app::runtime::{AvatarImages, UiAssets};
use bevy::prelude::*;
use leocard_game_common::PlayerId;
use leocard_protocol::{GameKind, TexasPhaseView, TexasPlayerView, TexasSnapshot};
use leocard_ui::{UiComposer, spawn_node};

#[expect(
    clippy::too_many_arguments,
    reason = "the own-area builder combines game controls with explicit visual resources"
)]
pub(super) fn add_texas_own_area(
    commands: &mut UiComposer,
    table: Entity,
    game: &TexasSnapshot,
    own: &TexasPlayerView,
    initial_deal: Option<&TexasInitialDeal>,
    ui: &mut TexasUiState,
    interaction_menu_open: Option<PlayerId>,
    assets: &UiAssets,
    avatars: &AvatarImages,
    turn_border_materials: &mut Assets<TurnBorderMaterial>,
    chip_state: &TexasChipTableState,
    start_transition_active: bool,
    intro_only: bool,
) {
    let portrait_height = 76.0 * 1.17;
    let own_seat = spawn_node(
        commands,
        table,
        Node {
            position_type: PositionType::Absolute,
            left: px(10),
            bottom: px(8),
            width: px(TEXAS_PORTRAIT_WIDTH),
            height: px(portrait_height),
            ..default()
        },
        None,
    );
    let portrait = add_player_portrait(
        commands,
        own_seat,
        Node {
            width: px(TEXAS_PORTRAIT_WIDTH),
            height: px(portrait_height),
            ..default()
        },
        PlayerPortraitSpec {
            player: own.player.id,
            profile: PlayerMenuProfile {
                name: &own.player.name,
                avatar: own.player.avatar.and_then(|id| avatars.remote.get(&id)),
                reference_points: own.player.reference_points,
                completed_games: own.player.completed_games,
                game_profiles: &own.player.game_profiles,
            },
            avatar_size: 52.0 * 1.17,
            auto_play: own.auto_play,
            menu_open: interaction_menu_open == Some(own.player.id),
            name_color: TEXT,
        },
        assets,
    );
    attach_start_game_seat_transition(
        commands,
        portrait.portrait,
        own.player.id,
        start_transition_active,
    );
    if !own.folded && game.current_player == Some(own.player.id) {
        add_turn_border_trace_with_radius(
            commands,
            portrait.avatar_ring,
            turn_border_materials,
            TurnBorderAnimationKey::new(GameKind::Texas, game.match_id, own.player.id),
            52.0 * 1.17 * 0.2,
            52.0 * 1.17,
        );
    }
    add_role_tokens(commands, portrait.avatar_ring, own.player.id, game, assets);
    if !intro_only {
        add_texas_win_rate(
            commands,
            portrait.avatar_ring,
            own.player.id,
            SeatSide::Top,
            game,
            &ui.spectator,
            assets,
        );
    }
    if intro_only {
        return;
    }
    add_texas_chip_popup(
        commands,
        own_seat,
        own.stack,
        None,
        assets,
        &chip_state.stack_counts(own.player.id),
    );

    // 一手结束后，自己的底牌也和其他玩家一样改放到面前的筹码区。
    if !matches!(game.phase, TexasPhaseView::HandComplete { .. }) {
        let omaha = game.your_hole_cards.len() == 4;
        let (area_width, card_width, card_height, card_gap) = if omaha {
            // 四张底牌使用较窄牌面，与操作行保留间隔。
            (324.0, 78.0, 106.0, 4.0)
        } else {
            (180.0, 84.0, 114.0, 9.0)
        };
        let scale = TexasTableLayout::HAND_SCALE;
        let (area_width, card_width, card_height, card_gap) = (
            area_width * scale,
            card_width * scale,
            card_height * scale,
            card_gap * scale,
        );
        let hole_cards = spawn_node(
            commands,
            table,
            Node {
                position_type: PositionType::Absolute,
                left: percent(50),
                bottom: px(8),
                width: px(area_width),
                height: px(card_height),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::FlexEnd,
                justify_content: JustifyContent::Center,
                column_gap: px(card_gap),
                ..default()
            },
            None,
        );
        commands
            .entity(hole_cards)
            .insert(UiTransform::from_translation(Val2::px(
                -area_width / 2.0,
                0.0,
            )));
        if !own.folded {
            for (index, card) in game.your_hole_cards.iter().copied().enumerate() {
                let delay = initial_deal
                    .and_then(|deal| {
                        deal.own_delays
                            .get(index)
                            .or_else(|| deal.own_delays.last())
                    })
                    .copied();
                let animation = initial_deal
                    .zip(delay)
                    .map(|(deal, delay)| TablePieceDeal::from_anchor(deal.source, delay));
                add_texas_card(
                    commands,
                    hole_cards,
                    card,
                    (card_width, card_height),
                    animation,
                    assets,
                );
                if let Some(delay) = delay {
                    commands.effect(PendingDealSound {
                        remaining: delay,
                        variant: index % assets.audio.deal_sounds.len().max(1),
                    });
                }
            }
        }
    }
    add_texas_actions(commands, table, game, own, ui, assets);
}
