use leocard_achievements::{
    AchievementBook, AchievementContext, AchievementDefinition, AchievementTrigger,
    achievement_by_id,
};
use leocard_minecraft_guess::{
    Catalogue, CorrectGuess, EntryKind, GuessActionStatistics, GuessHandStatistics, GuessPool,
};
use leocard_protocol::{GameEvent, MatchId, MinecraftGuessEvent, PlayerId};
use std::collections::BTreeSet;

fn context(match_byte: u8, hand: u32, sequence: u128) -> Option<AchievementContext> {
    Some(AchievementContext {
        match_id: MatchId([match_byte; 16]),
        hand_index: Some(hand),
        sequence,
    })
}

fn definition(id: &str) -> &'static AchievementDefinition {
    achievement_by_id(&format!("leocard:minecraft_guess/{id}")).unwrap()
}

fn win(id: &str, kind: EntryKind) -> AchievementTrigger {
    AchievementTrigger::Game {
        player: PlayerId(0),
        event: GameEvent::MinecraftGuess(MinecraftGuessEvent::ActionAnalyzed {
            player: PlayerId(0),
            statistics: Box::new(GuessActionStatistics {
                player: 0,
                correct_entry: Some(CorrectGuess {
                    id: id.into(),
                    kind,
                }),
                wrong_all_matching: false,
                unknown: false,
                unknown_run: 0,
                skip_run: 0,
                all_wrong_after_first: false,
                ended_matching_run: false,
                won_after_skip: false,
                first_cycle_win: true,
                opening_win: true,
                opponents_all_skipped: false,
                long_search_win: false,
            }),
        }),
    }
}

fn hand(consistent: bool) -> AchievementTrigger {
    AchievementTrigger::Game {
        player: PlayerId(0),
        event: GameEvent::MinecraftGuess(MinecraftGuessEvent::HandAnalyzed {
            player: PlayerId(0),
            statistics: GuessHandStatistics {
                player: 0,
                consistent,
                round_sweep: false,
                match_sweep: false,
                no_answers_top: false,
            },
        }),
    }
}

fn reload(book: &AchievementBook) -> AchievementBook {
    postcard::from_bytes(&postcard::to_allocvec(book).unwrap()).unwrap()
}

#[test]
fn live_events_are_personal_and_duplicate_sequences_do_not_count_twice() {
    let mut book = AchievementBook::default();
    let mut other = win("minecraft:stone", EntryKind::Block);
    if let AchievementTrigger::Game { player, .. } = &mut other {
        *player = PlayerId(1);
    }
    book.trigger(&other, context(1, 0, 1), 10);
    assert!(!book.achieved(definition("first_win")));
    let event = win("minecraft:stone", EntryKind::Block);
    let first = book.trigger(&event, context(1, 0, 2), 11);
    assert!(
        first
            .unlocked
            .iter()
            .any(|entry| entry.id == definition("opening_win").id)
    );
    assert!(
        book.trigger(&event, context(1, 0, 2), 12)
            .unlocked
            .is_empty()
    );
    book = reload(&book);
    book.trigger(&event, context(2, 0, 1), 13);
    assert_eq!(
        book.criterion_count(definition("ten_thousand"), "progress"),
        2
    );
}

#[test]
fn five_consistent_games_survive_reload_but_a_failure_resets_the_streak() {
    let mut book = AchievementBook::default();
    for index in 0..4 {
        book.trigger(&hand(true), context(1, index, u128::from(index) + 1), 10);
    }
    assert_eq!(
        book.criterion_count(definition("consistent_five"), "progress"),
        4
    );
    book = reload(&book);
    book.trigger(&hand(false), context(2, 0, 1), 11);
    assert_eq!(
        book.criterion_count(definition("consistent_five"), "progress"),
        0
    );
    for index in 0..5 {
        book.trigger(&hand(true), context(3, index, u128::from(index) + 1), 12);
    }
    assert!(book.achieved(definition("consistent_five")));
}

#[test]
fn collecting_all_entries_counts_block_and_item_ids_separately_and_persists() {
    let mut book = AchievementBook::default();
    let entries: Vec<_> = Catalogue::bundled().entries(GuessPool::Mixed).collect();
    let mut kinds = BTreeSet::new();
    for (index, entry) in entries.iter().enumerate() {
        // Keep one wheat entry missing even though its other kind is collected.
        if entry.id == "minecraft:wheat" {
            kinds.insert(entry.kind.label());
            if entry.kind == EntryKind::Item {
                continue;
            }
        }
        book.trigger(
            &win(&entry.id, entry.kind),
            context(1, index as u32, index as u128 + 1),
            10,
        );
        if index == entries.len() / 2 {
            book = reload(&book);
        }
    }
    assert_eq!(kinds.len(), 2);
    assert!(!book.achieved(definition("all_entries")));
    assert!(book.achieved(definition("all_versions")));
    book = reload(&book);
    book.trigger(
        &win("minecraft:wheat", EntryKind::Block),
        context(2, 0, 1),
        11,
    );
    assert!(!book.achieved(definition("all_entries")));
    book.trigger(
        &win("minecraft:wheat", EntryKind::Item),
        context(2, 1, 2),
        12,
    );
    assert!(book.achieved(definition("all_entries")));
}
