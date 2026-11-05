use crate::achievements::{TexasAchievementEvent, register};
use crate::{
    TexasAction, TexasCard, TexasGameState, TexasHandStatistics, TexasPhase, TexasRank,
    TexasRuleSet, TexasSuit, build_deck,
};
use leocard_achievements::{AchievementRegistry, AchievementTier, AchievementTrigger};
use leocard_game_common::PlayerId;
use std::collections::HashSet;

pub(super) const fn c(rank: TexasRank, suit: TexasSuit) -> TexasCard {
    TexasCard::new(suit, rank)
}

pub(super) fn play(
    holes: &[[TexasCard; 2]],
    board: [TexasCard; 5],
    stacks: Vec<u32>,
    actions: &[(u8, TexasAction)],
) -> TexasGameState {
    let mut deck = Vec::new();
    // Dealer 0: deal one card to 1, 2, ..., 0, then repeat; no burns.
    for round in 0..2 {
        for seat in 1..=holes.len() {
            deck.push(holes[seat % holes.len()][round]);
        }
    }
    deck.extend(board);
    let reserved: HashSet<_> = deck.iter().copied().collect();
    assert_eq!(reserved.len(), deck.len(), "duplicate fixture card");
    deck.extend(
        build_deck(false)
            .into_iter()
            .filter(|card| !reserved.contains(card)),
    );
    // Heads-up is the final two funded seats of a supported table, not a two-seat ruleset.
    let player_count = holes.len().max(usize::from(TexasRuleSet::MIN_PLAYERS));
    let mut stacks = stacks;
    stacks.resize(player_count, 0);
    let mut game = TexasGameState::new_with_stacks(
        TexasRuleSet {
            player_count: player_count as u8,
            ..Default::default()
        },
        PlayerId(0),
        stacks,
        deck,
    )
    .unwrap();
    for (seat, hole) in holes.iter().enumerate() {
        assert_eq!(game.players()[seat].hole_cards(), hole);
    }
    while let Some((player, _, _)) = game.blind_to_post() {
        game.act(player, TexasAction::PostBlind).unwrap();
    }
    for &(seat, action) in actions {
        let player = PlayerId(seat);
        assert_eq!(game.current_player(), Some(player));
        game.act(player, action).unwrap();
    }
    assert!(matches!(game.phase(), TexasPhase::Complete(_)));
    assert_eq!(game.current_player(), None);
    assert_eq!(game.community(), &board);
    game
}

pub(super) fn amount(statistics: TexasHandStatistics) -> u64 {
    let mut registry = AchievementRegistry::default();
    register(&mut registry);
    let definition = registry
        .by_id("leocard:texas/aces_cracked_by_unpaired")
        .unwrap();
    assert_eq!(definition.tier, AchievementTier::Silver);
    assert_eq!(definition.title, "现在谁才是冤家？");
    let trigger = AchievementTrigger::Game {
        player: PlayerId(0),
        event: TexasAchievementEvent::HandAnalyzed {
            player: PlayerId(0),
            statistics: Box::new(statistics),
        }
        .into(),
    };
    (definition.criteria[0].amount)(&trigger)
}
