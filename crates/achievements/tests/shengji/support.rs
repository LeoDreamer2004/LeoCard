pub(super) use leocard_achievements::{
    AchievementBook, AchievementContext, AchievementTrigger, achievement_by_id,
};
use leocard_protocol::ShengjiPublicPlay;
pub(super) use leocard_protocol::{GameEvent, PlayerId, ShengjiEvent};
pub(super) use leocard_shengji::{
    Category, Component, HandResult, ShengjiCard as Card, ShengjiClassifiedPlay,
    ShengjiHandStatistics, ShengjiMatchStatistics, ShengjiOpeningHandStatistics, ShengjiPlayerId,
    ShengjiRank as Rank, ShengjiTeamId,
};

pub(super) fn statistics(player: u8) -> ShengjiHandStatistics {
    let card = Card::small_joker(0);
    ShengjiHandStatistics {
        player: ShengjiPlayerId(player),
        deck_count: 2,
        opening_hand: ShengjiOpeningHandStatistics {
            card_count: 25,
            trump_count: 5,
            joker_count: 1,
        },
        burial: None,
        bottom_burier: Some(ShengjiPlayerId(0)),
        first_trick_cut_dealer: false,
        result: HandResult {
            dealer: ShengjiPlayerId(0),
            dealer_team: ShengjiTeamId(0),
            collecting_team: ShengjiTeamId(1),
            trick_points: 40,
            penalty_adjustment: 0,
            kitty_points: 0,
            kitty_multiplier: 0,
            collecting_score: 40,
            promoted_team: ShengjiTeamId(0),
            promoted_steps: 1,
            next_dealer: ShengjiPlayerId(2),
            levels: [Rank::Three, Rank::Two],
        },
        last_trick_winner: ShengjiPlayerId(0),
        last_winning_play: ShengjiClassifiedPlay {
            cards: vec![card],
            category: Category::Trump,
            components: vec![Component::Single { card, strength: 20 }],
        },
    }
}

pub(super) fn analyzed(facts: ShengjiHandStatistics) -> AchievementTrigger {
    AchievementTrigger::Game {
        player: PlayerId(facts.player.0),
        event: GameEvent::Shengji(ShengjiEvent::HandAnalyzed {
            player: PlayerId(facts.player.0),
            statistics: Box::new(facts),
            match_statistics: ShengjiMatchStatistics::default(),
        }),
    }
}

pub(super) fn amount(id: &str, trigger: &AchievementTrigger) -> u64 {
    (achievement_by_id(&format!("leocard:shengji/{id}"))
        .unwrap()
        .criteria[0]
        .amount)(trigger)
}

pub(super) fn hand_amount(id: &str, facts: &ShengjiHandStatistics) -> u64 {
    amount(id, &analyzed(facts.clone()))
}

pub(super) fn played(play: ShengjiClassifiedPlay, is_lead: bool) -> AchievementTrigger {
    AchievementTrigger::Game {
        player: PlayerId(1),
        event: GameEvent::Shengji(ShengjiEvent::CardsPlayed {
            play: ShengjiPublicPlay {
                player: PlayerId(1),
                play,
                throw_penalty: 0,
            },
            is_lead,
        }),
    }
}
