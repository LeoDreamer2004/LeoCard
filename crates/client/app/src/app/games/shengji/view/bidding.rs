use super::super::skin::{add_shengji_button, add_shengji_disabled_button};
use super::super::{ShengjiAssets, ShengjiUiAction};
use super::bid_strip::add_shengji_bid_strip;
use super::shengji_current_level;
use crate::app::presentation::{ACCENT, GameButtonTone, add_text};
use crate::app::runtime::{ClientResource, UiAssets};
use crate::app::shell::UiAction;
use bevy::prelude::*;
use leocard_protocol::{ShengjiDeclarationView, ShengjiPhaseView, ShengjiSnapshot};
use leocard_shengji::bid_joker_for_suit;
use leocard_shengji::{ShengjiBidTrump, ShengjiCard, ShengjiRank, ShengjiSuit};
use leocard_ui::{UiComposer, spawn_node};

#[derive(Component)]
pub(crate) struct ShengjiBiddingCountdown;

pub(crate) fn shengji_bidding_countdown_label(milliseconds: u16) -> String {
    format!("{}秒", milliseconds.div_ceil(1000))
}

pub(crate) fn sync_shengji_bidding_countdown(
    client: Option<Res<ClientResource>>,
    mut labels: Query<&mut Text, With<ShengjiBiddingCountdown>>,
) {
    let Some(milliseconds) = client
        .as_deref()
        .and_then(|client| client.0.model().shengji_game())
        .and_then(|game| match game.phase {
            ShengjiPhaseView::BiddingGrace {
                milliseconds_remaining,
                ..
            } => Some(milliseconds_remaining),
            _ => None,
        })
    else {
        return;
    };
    let expected = shengji_bidding_countdown_label(milliseconds);
    for mut label in &mut labels {
        if label.0 != expected {
            label.0.clone_from(&expected);
        }
    }
}

pub(super) fn add_shengji_bidding_panel(
    commands: &mut UiComposer,
    hand_area: Entity,
    game: &ShengjiSnapshot,
    assets: &UiAssets,
    game_assets: &ShengjiAssets,
) {
    let bottom_copy = match game.phase {
        ShengjiPhaseView::Dealing { .. } | ShengjiPhaseView::BiddingGrace { .. } => false,
        ShengjiPhaseView::BottomCopying { player, .. }
            if player == game.you
                && !game
                    .players
                    .iter()
                    .any(|player| player.id == game.you && player.auto_play) =>
        {
            true
        }
        _ => return,
    };
    let panel = spawn_node(
        commands,
        hand_area,
        Node {
            position_type: PositionType::Absolute,
            left: percent(25),
            right: percent(25),
            top: px(2),
            height: px(54),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            column_gap: px(6),
            ..default()
        },
        None,
    );
    commands.entity(panel).insert(GlobalZIndex(80));
    add_shengji_bid_strip(commands, panel, game, bottom_copy, assets, game_assets);
    if bottom_copy {
        add_shengji_button(
            commands,
            panel,
            "不抄底",
            UiAction::Shengji(ShengjiUiAction::DeclineBottomCopy),
            assets,
            116.0,
            42.0,
            GameButtonTone::Pass,
        );
    } else if let ShengjiPhaseView::BiddingGrace {
        milliseconds_remaining,
        you_confirmed,
        ..
    } = &game.phase
    {
        let countdown = add_text(
            commands,
            panel,
            shengji_bidding_countdown_label(*milliseconds_remaining),
            13.0,
            ACCENT,
            assets,
        );
        commands.entity(countdown).insert(ShengjiBiddingCountdown);
        let pass_label = if *you_confirmed {
            "✓ 已确认"
        } else {
            match game.declaration.as_ref() {
                None => "不亮主",
                Some(declaration) if declaration.player == game.you => "不加亮",
                Some(_) => "不反主",
            }
        };
        add_shengji_bid_pass_button(commands, panel, pass_label, *you_confirmed, assets);
    }
}

fn add_shengji_bid_pass_button(
    commands: &mut UiComposer,
    parent: Entity,
    label: &str,
    confirmed: bool,
    assets: &UiAssets,
) {
    if confirmed {
        add_shengji_disabled_button(commands, parent, label, assets, 116.0, 42.0);
    } else {
        add_shengji_button(
            commands,
            parent,
            label,
            UiAction::Shengji(ShengjiUiAction::ConfirmBidPass),
            assets,
            116.0,
            42.0,
            GameButtonTone::Pass,
        );
    }
}

pub(crate) fn shengji_declaration_candidate(
    game: &ShengjiSnapshot,
    suit: Option<ShengjiSuit>,
) -> Option<Vec<ShengjiCard>> {
    if matches!(game.phase, ShengjiPhaseView::BottomCopying { .. })
        && game
            .declaration
            .as_ref()
            .is_some_and(|declaration| declaration.player == game.you)
    {
        return None;
    }
    let level = shengji_current_level(game);
    if game.rules.bid_with_joker {
        return joker_bid_declaration_candidate(game, suit, level);
    }
    let exposed = game
        .declaration
        .as_ref()
        .filter(|declaration| declaration.player == game.you)
        .map_or(&[][..], |declaration| declaration.cards.as_slice());
    let available = game
        .your_hand
        .iter()
        .copied()
        .filter(|card| !exposed.contains(card))
        .collect::<Vec<_>>();
    if game.rules.deck_count >= 3 {
        return multi_deck_declaration_candidate(game, suit, level, &available);
    }
    match suit {
        Some(suit) => {
            let matching = available
                .iter()
                .copied()
                .filter(|card| card.rank() == level && card.suit() == Some(suit))
                .collect::<Vec<_>>();
            let protecting = game.declaration.as_ref().is_some_and(|declaration| {
                declaration.player == game.you
                    && declaration.cards.len() == 1
                    && declaration.trump == ShengjiBidTrump::Suit(suit)
            });
            if game.declaration.is_none() || protecting {
                matching.first().copied().map(|card| vec![card])
            } else if matching.len() >= 2
                && game.declaration.as_ref().is_some_and(|declaration| {
                    !declaration.protected
                        && ShengjiBidTrump::Suit(suit).strength() > declaration.trump.strength()
                })
            {
                Some(matching[..2].to_vec())
            } else {
                None
            }
        }
        None => [ShengjiRank::BigJoker, ShengjiRank::SmallJoker]
            .into_iter()
            .find_map(|rank| {
                let matching = available
                    .iter()
                    .copied()
                    .filter(|card| card.rank() == rank)
                    .collect::<Vec<_>>();
                let trump = if rank == ShengjiRank::BigJoker {
                    ShengjiBidTrump::NoTrumpBigJoker
                } else {
                    ShengjiBidTrump::NoTrumpSmallJoker
                };
                (matching.len() >= 2
                    && game
                        .declaration
                        .as_ref()
                        .is_none_or(|declaration| trump.strength() > declaration.trump.strength()))
                .then(|| matching[..2].to_vec())
            }),
    }
}

fn joker_bid_declaration_candidate(
    game: &ShengjiSnapshot,
    suit: Option<ShengjiSuit>,
    level: ShengjiRank,
) -> Option<Vec<ShengjiCard>> {
    let current = game.declaration.as_ref();
    let current_count = current.map(|declaration| declaration_primary_count(declaration, level));
    let current_strength = current.and_then(|declaration| {
        declaration
            .trump
            .declaration_strength(game.rules.deck_count, current_count.unwrap_or_default())
    });
    let legal_strength = |trump: ShengjiBidTrump, count: usize| {
        let strength = trump.declaration_strength(game.rules.deck_count, count)?;
        if let Some(declaration) = current {
            if declaration.protected
                && matches!(trump, ShengjiBidTrump::Suit(_))
                && count <= current_count.unwrap_or_default()
            {
                return None;
            }
            if current_strength.is_some_and(|current| strength <= current) {
                return None;
            }
        }
        Some(strength)
    };

    match suit {
        Some(suit) => {
            let trump = ShengjiBidTrump::Suit(suit);
            let extends_current = current.is_some_and(|declaration| {
                declaration.player == game.you && declaration.trump == trump
            });
            // 加固只提交新增级牌；此前被反掉的声明不限制这些牌再次使用。
            let matching = game
                .your_hand
                .iter()
                .copied()
                .filter(|card| {
                    card.rank() == level
                        && card.suit() == Some(suit)
                        && (!extends_current || !current.unwrap().cards.contains(card))
                })
                .collect::<Vec<_>>();
            let companion_rank = bid_joker_for_suit(suit);
            let companion = current
                .filter(|_| extends_current)
                .and_then(|declaration| {
                    declaration
                        .cards
                        .iter()
                        .copied()
                        .find(|card| card.rank() == companion_rank && card.suit().is_none())
                })
                .or_else(|| {
                    game.your_hand
                        .iter()
                        .copied()
                        .find(|card| card.rank() == companion_rank && card.suit().is_none())
                })?;
            let existing_count = if extends_current {
                current_count.unwrap_or_default()
            } else {
                0
            };
            let minimum_count = if current.is_none() {
                1
            } else if extends_current {
                existing_count + 1
            } else {
                2
            };
            for total_count in minimum_count..=usize::from(game.rules.deck_count) {
                if legal_strength(trump, total_count).is_none() {
                    continue;
                }
                let needed = total_count.saturating_sub(existing_count);
                if matching.len() < needed {
                    continue;
                }
                let mut cards = matching[..needed].to_vec();
                let current_has_companion = extends_current
                    && current.is_some_and(|declaration| declaration.cards.contains(&companion));
                if !current_has_companion {
                    cards.push(companion);
                }
                return Some(cards);
            }
            None
        }
        None => {
            // 无主在带王亮中只能反主，不能作为首次亮牌。
            current?;
            let mut candidates = Vec::<(u8, Vec<ShengjiCard>)>::new();
            for (trump, rank) in [
                (ShengjiBidTrump::NoTrumpSmallJoker, ShengjiRank::SmallJoker),
                (ShengjiBidTrump::NoTrumpBigJoker, ShengjiRank::BigJoker),
            ] {
                let matching = game
                    .your_hand
                    .iter()
                    .copied()
                    .filter(|card| card.rank() == rank)
                    .collect::<Vec<_>>();
                let extends_current = current.is_some_and(|declaration| {
                    declaration.player == game.you && declaration.trump == trump
                });
                let existing_cards = current
                    .filter(|_| extends_current)
                    .map_or(&[][..], |declaration| declaration.cards.as_slice());
                let available = matching
                    .iter()
                    .copied()
                    .filter(|card| !existing_cards.contains(card))
                    .collect::<Vec<_>>();
                let existing_count = existing_cards.len();
                let minimum_count = if extends_current {
                    (existing_count + 1).max(2)
                } else {
                    2
                };
                for total_count in minimum_count..=usize::from(game.rules.deck_count) {
                    let Some(strength) = legal_strength(trump, total_count) else {
                        continue;
                    };
                    let needed = total_count.saturating_sub(existing_count);
                    if available.len() >= needed {
                        candidates.push((strength, available[..needed].to_vec()));
                    }
                }
            }
            candidates.sort_by_key(|(strength, _)| *strength);
            candidates.into_iter().next().map(|(_, cards)| cards)
        }
    }
}

fn declaration_primary_count(declaration: &ShengjiDeclarationView, level: ShengjiRank) -> usize {
    match declaration.trump {
        ShengjiBidTrump::Suit(suit) => declaration
            .cards
            .iter()
            .filter(|card| card.rank() == level && card.suit() == Some(suit))
            .count(),
        ShengjiBidTrump::NoTrumpSmallJoker => declaration
            .cards
            .iter()
            .filter(|card| card.rank() == ShengjiRank::SmallJoker)
            .count(),
        ShengjiBidTrump::NoTrumpBigJoker => declaration
            .cards
            .iter()
            .filter(|card| card.rank() == ShengjiRank::BigJoker)
            .count(),
    }
}

fn multi_deck_declaration_candidate(
    game: &ShengjiSnapshot,
    suit: Option<ShengjiSuit>,
    level: ShengjiRank,
    available: &[ShengjiCard],
) -> Option<Vec<ShengjiCard>> {
    let current_strength = game.declaration.as_ref().and_then(|declaration| {
        declaration
            .trump
            .declaration_strength(game.rules.deck_count, declaration.cards.len())
    });
    let mut candidates = Vec::<(u8, Vec<ShengjiCard>)>::new();
    let faces = match suit {
        Some(suit) => vec![(ShengjiBidTrump::Suit(suit), Some(suit), level)],
        None => vec![
            (
                ShengjiBidTrump::NoTrumpSmallJoker,
                None,
                ShengjiRank::SmallJoker,
            ),
            (
                ShengjiBidTrump::NoTrumpBigJoker,
                None,
                ShengjiRank::BigJoker,
            ),
        ],
    };
    for (trump, face_suit, rank) in faces {
        let matching = available
            .iter()
            .copied()
            .filter(|card| card.rank() == rank && card.suit() == face_suit)
            .collect::<Vec<_>>();
        let minimum_count = if game.declaration.is_some() {
            2
        } else {
            usize::from(face_suit.is_none()) + 1
        };
        for total_count in minimum_count..=usize::from(game.rules.deck_count) {
            let Some(strength) = trump.declaration_strength(game.rules.deck_count, total_count)
            else {
                continue;
            };
            if current_strength.is_some_and(|current| strength <= current) {
                continue;
            }
            let existing_count = game.declaration.as_ref().map_or(0, |declaration| {
                usize::from(
                    declaration.player == game.you
                        && declaration.trump == trump
                        && declaration
                            .cards
                            .iter()
                            .all(|card| card.rank() == rank && card.suit() == face_suit),
                ) * declaration.cards.len()
            });
            if total_count <= existing_count {
                continue;
            }
            let needed = total_count - existing_count;
            if matching.len() >= needed {
                candidates.push((strength, matching[..needed].to_vec()));
            }
        }
    }
    candidates.sort_by_key(|(strength, _)| *strength);
    candidates.into_iter().next().map(|(_, cards)| cards)
}
