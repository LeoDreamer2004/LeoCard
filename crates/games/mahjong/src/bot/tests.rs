use super::search::{Position, Search};
use super::{MahjongBot, MahjongBotAction, MahjongBotObservation};
use crate::{
    MahjongGameState, MahjongMeld, MahjongPhase, MahjongRuleSet, MahjongScoreInput, MahjongSuit,
    MahjongTile, MahjongTileKind, MahjongWinContext, MahjongWinSource, MahjongWind, build_deck,
};
use leocard_game_common::{PlayerId, SearchBudget};
use std::time::Duration;

fn suited(suit: MahjongSuit, ranks: &[u8]) -> Vec<MahjongTileKind> {
    ranks
        .iter()
        .map(|rank| MahjongTileKind::suited(suit, *rank))
        .collect()
}

fn observation(kinds: Vec<MahjongTileKind>, melds: Vec<MahjongMeld>) -> MahjongBotObservation {
    let mut seen = [0; 34];
    let hand = kinds
        .into_iter()
        .map(|kind| {
            let count = &mut seen[kind.index34().unwrap()];
            let tile = MahjongTile::new(kind, *count);
            *count += 1;
            tile
        })
        .collect::<Vec<_>>();
    for kind in melds.iter().flat_map(|meld| meld.tile_kinds()) {
        seen[kind.index34().unwrap()] += 1;
    }
    let discard = *hand.last().unwrap();
    MahjongBotObservation {
        player: PlayerId(0),
        hand,
        melds,
        public: Vec::new(),
        discards: Vec::new(),
        unseen: seen.map(|count| 4 - count),
        options: vec![MahjongBotAction::Discard(discard)],
        may_declare_ready: true,
        claim: None,
        rules: MahjongRuleSet {
            declared_ready: true,
            ..Default::default()
        },
        context: MahjongWinContext {
            declared_ready: false,
            source: MahjongWinSource::SelfDraw,
            seat_wind: MahjongWind::East,
            prevalent_wind: MahjongWind::East,
            last_wall_tile: false,
            last_of_kind: false,
            flower_count: 0,
        },
        wall: 60,
        scores: [0; 4],
    }
}

fn ready_position(observation: &MahjongBotObservation) -> Position {
    let mut counts = [0; 34];
    for tile in &observation.hand[..observation.hand.len() - 1] {
        counts[tile.kind().index34().unwrap()] += 1;
    }
    Position {
        counts,
        melds: observation.melds.clone(),
        replacement: false,
        declared_ready: true,
    }
}

#[test]
fn ready_candidates_follow_the_rule_and_execute_a_real_declaration() {
    let kinds = suited(
        MahjongSuit::Characters,
        &[1, 1, 1, 2, 2, 2, 3, 3, 3, 4, 4, 4, 5, 9],
    );
    let fixture = observation(kinds, Vec::new());
    let mut deck = build_deck();
    let slots = (0..3)
        .flat_map(|round| round * 16..round * 16 + 4)
        .chain([48, 52]);
    for (slot, tile) in slots.zip(&fixture.hand) {
        let index = deck.iter().position(|held| held == tile).unwrap();
        deck.swap(slot, index);
    }
    for declared_ready in [false, true] {
        let mut game = MahjongGameState::new_with_deck(
            MahjongRuleSet {
                declared_ready,
                ..Default::default()
            },
            deck.clone(),
            PlayerId(0),
        )
        .unwrap();
        while matches!(game.phase(), MahjongPhase::Dealing { .. }) {
            game.advance_deal().unwrap();
        }
        let observation = MahjongBotObservation::from_game(&game, PlayerId(0)).unwrap();
        assert_eq!(observation.may_declare_ready, declared_ready);
        let action = MahjongBot::choose(&observation, &SearchBudget::new(Duration::ZERO));
        if declared_ready {
            let MahjongBotAction::DeclareReady(tile) = action else {
                panic!("a live, valuable wait should be declared: {action:?}");
            };
            game.declare_ready(PlayerId(0), tile).unwrap();
            assert!(game.player(PlayerId(0)).unwrap().has_declared_ready());
        } else {
            assert!(matches!(action, MahjongBotAction::Discard(_)));
        }
    }
}

#[test]
fn declaration_can_supply_the_eighth_fan() {
    let mut kinds = suited(MahjongSuit::Characters, &[1, 2]);
    kinds.extend([MahjongTileKind::Wind(MahjongWind::East); 2]);
    kinds.extend(suited(MahjongSuit::Bamboo, &[9]));
    let observation = observation(
        kinds,
        vec![
            MahjongMeld::chow(MahjongSuit::Dots, 2, PlayerId(1)),
            MahjongMeld::chow(MahjongSuit::Bamboo, 3, PlayerId(1)),
            MahjongMeld::chow(MahjongSuit::Dots, 6, PlayerId(1)),
        ],
    );
    let winning_tile = MahjongTileKind::suited(MahjongSuit::Characters, 3);
    let mut concealed = observation.hand[..4]
        .iter()
        .map(|tile| tile.kind())
        .collect::<Vec<_>>();
    concealed.push(winning_tile);
    for declared_ready in [false, true] {
        let mut context = observation.context;
        context.source = MahjongWinSource::Discard(PlayerId(1));
        context.declared_ready = declared_ready;
        let score = observation
            .rules
            .score_hand(&MahjongScoreInput {
                concealed: concealed.clone(),
                melds: observation.melds.clone(),
                winning_tile,
                context,
            })
            .unwrap();
        assert_eq!(score.points_without_flowers, 7 + u16::from(declared_ready));
    }
    assert_eq!(
        MahjongBot::choose(&observation, &SearchBudget::new(Duration::ZERO)),
        MahjongBotAction::DeclareReady(*observation.hand.last().unwrap())
    );
}

#[test]
fn bot_does_not_lock_a_dead_wait_or_a_chicken_hand_below_eight_fan() {
    let mut kinds = suited(MahjongSuit::Characters, &[5, 6]);
    kinds.extend([MahjongTileKind::Wind(MahjongWind::East); 2]);
    kinds.extend(suited(MahjongSuit::Characters, &[9]));
    let mut observation = observation(
        kinds,
        vec![
            MahjongMeld::chow(MahjongSuit::Characters, 2, PlayerId(1)),
            MahjongMeld::chow(MahjongSuit::Dots, 5, PlayerId(1)),
            MahjongMeld::chow(MahjongSuit::Bamboo, 7, PlayerId(1)),
        ],
    );
    let discard = MahjongBotAction::Discard(*observation.hand.last().unwrap());
    assert_eq!(
        MahjongBot::choose(&observation, &SearchBudget::new(Duration::ZERO)),
        discard
    );
    observation.rules.minimum_eight_points = false;
    assert!(matches!(
        MahjongBot::choose(&observation, &SearchBudget::new(Duration::ZERO)),
        MahjongBotAction::DeclareReady(_)
    ));
    observation.unseen = [0; 34];
    assert_eq!(
        MahjongBot::choose(&observation, &SearchBudget::new(Duration::ZERO)),
        discard
    );
}

#[test]
fn ready_search_discards_a_miss_without_changing_the_locked_hand() {
    let kinds = suited(
        MahjongSuit::Characters,
        &[1, 1, 1, 2, 2, 2, 3, 3, 3, 4, 4, 4, 5, 9],
    );
    let mut observation = observation(kinds, Vec::new());
    observation.unseen = [0; 34];
    observation.unseen[MahjongTileKind::suited(MahjongSuit::Dots, 9)
        .index34()
        .unwrap()] = 1;
    let position = ready_position(&observation);
    let budget = SearchBudget::new(Duration::from_secs(1));
    let mut search = Search::new(&observation, &budget);
    let expected = search.evaluate(&position, &[0; 34]);
    let actual = search.future(&position, &observation.unseen, 1).unwrap();
    assert!((actual - expected).abs() < 0.000_001);
}
