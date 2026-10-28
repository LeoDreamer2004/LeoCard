use super::{TexasAchievementEvent, register};
use crate::{
    GameState, TexasHoldemAction, TexasHoldemPlayerId, TexasHoldemRank, TexasHoldemRuleSet,
    TexasHoldemSuit, build_deck,
};
use leocard_achievements::{AchievementRegistry, AchievementTrigger, ParticipantId};
use std::collections::HashSet;

#[test]
fn tied_pot_achievements_count_shared_winners_and_exclude_folders() {
    let mut registry = AchievementRegistry::default();
    register(&mut registry);
    for (count, fold) in [(3, Some(2)), (3, None), (5, None), (5, Some(4))] {
        let board: Vec<_> = [
            TexasHoldemRank::Ten,
            TexasHoldemRank::Jack,
            TexasHoldemRank::Queen,
            TexasHoldemRank::King,
            TexasHoldemRank::Ace,
        ]
        .map(|rank| crate::TexasHoldemCard::new(TexasHoldemSuit::Heart, rank))
        .into();
        let holes: Vec<_> = build_deck(false)
            .into_iter()
            .filter(|card| !board.contains(card))
            .take(usize::from(count) * 2)
            .collect();
        let mut deck = holes;
        deck.extend(board);
        let reserved: HashSet<_> = deck.iter().copied().collect();
        deck.extend(
            build_deck(false)
                .into_iter()
                .filter(|card| !reserved.contains(card)),
        );
        let mut game = GameState::new_with_deck(
            TexasHoldemRuleSet {
                player_count: count,
                ..Default::default()
            },
            TexasHoldemPlayerId(0),
            deck,
        )
        .unwrap();
        for _ in 0..20 {
            let Some(player) = game.current_player() else {
                break;
            };
            let action = if game.blind_to_post().is_some() {
                TexasHoldemAction::PostBlind
            } else if Some(player.0) == fold {
                TexasHoldemAction::Fold
            } else {
                TexasHoldemAction::AllIn
            };
            game.act(player, action).unwrap();
        }
        assert!(game.current_player().is_none());
        for index in 0..usize::from(count) {
            let report = game.hand_statistics(TexasHoldemPlayerId(index)).unwrap();
            let winners = if Some(index) == fold {
                0
            } else {
                count - u8::from(fold.is_some())
            };
            assert_eq!(report.tied_winners, winners);
            let trigger = AchievementTrigger::Game {
                player: ParticipantId(index as u8),
                event: TexasAchievementEvent::HandAnalyzed {
                    player: ParticipantId(index as u8),
                    statistics: Box::new(report),
                }
                .into(),
            };
            for (id, minimum) in [("three_way_tie", 3), ("five_way_tie", 5)] {
                let definition = registry.by_id(&format!("leocard:texas/{id}")).unwrap();
                assert_eq!(
                    (definition.criteria[0].amount)(&trigger),
                    u64::from(winners >= minimum)
                );
            }
        }
    }
}
