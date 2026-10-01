use leocard_achievements::{
    AchievementBook, AchievementContext, AchievementTrigger, achievement_by_id,
};
use leocard_protocol::{GameEvent, MatchId, PlayerId, TexasHoldemEvent};
use leocard_texas_holdem::{
    GameState, Phase, TexasHoldemAction, TexasHoldemCard as Card, TexasHoldemHandStatistics,
    TexasHoldemPlayerId, TexasHoldemRank as Rank, TexasHoldemRuleSet, TexasHoldemSuit as Suit,
    build_deck,
};

pub(super) fn achieved(book: &AchievementBook, id: &str) -> bool {
    book.achieved(achievement_by_id(&format!("leocard:texas/{id}")).unwrap())
}

pub(super) fn trigger(
    book: &mut AchievementBook,
    facts: TexasHoldemHandStatistics,
    sequence: u128,
) {
    book.trigger(
        &AchievementTrigger::Game {
            player: PlayerId(0),
            event: GameEvent::TexasHoldem(TexasHoldemEvent::HandAnalyzed {
                player: PlayerId(0),
                statistics: Box::new(facts),
            }),
        },
        Some(AchievementContext {
            match_id: MatchId([1; 16]),
            hand_index: Some(sequence as u32),
            sequence,
        }),
        100,
    );
}

pub(super) fn card(rank: Rank, suit: Suit) -> Card {
    Card::new(suit, rank)
}

pub(super) fn game(hands: &[[Card; 2]], board: [Card; 5], stacks: Vec<u32>) -> GameState {
    let prefix = deck(hands, board, 0);
    GameState::new_with_stacks(
        TexasHoldemRuleSet {
            player_count: hands.len() as u8,
            ..Default::default()
        },
        TexasHoldemPlayerId(0),
        stacks,
        prefix,
    )
    .unwrap()
}

pub(super) fn check_down(game: &mut GameState) {
    while matches!(game.phase(), Phase::Betting(_)) {
        let player = game.current_player().unwrap();
        let action = if game.blind_to_post().is_some() {
            TexasHoldemAction::PostBlind
        } else if game.amount_to_call(player).unwrap() > 0 {
            TexasHoldemAction::Call
        } else {
            TexasHoldemAction::Check
        };
        game.act(player, action).unwrap();
    }
}

pub(super) fn post_blinds(game: &mut GameState) {
    for _ in 0..2 {
        game.act(game.current_player().unwrap(), TexasHoldemAction::PostBlind)
            .unwrap();
    }
}

pub(super) fn deck(hands: &[[Card; 2]], board: [Card; 5], dealer: usize) -> Vec<Card> {
    let mut prefix = Vec::new();
    for round in 0..2 {
        for distance in 1..=hands.len() {
            prefix.push(hands[(dealer + distance) % hands.len()][round]);
        }
    }
    prefix.extend(board);
    let mut rest = build_deck(false)
        .into_iter()
        .filter(|card| !prefix.contains(card))
        .collect::<Vec<_>>();
    prefix.append(&mut rest);
    prefix
}
