use leocard_achievements::{
    AchievementBook, AchievementContext, AchievementTrigger, achievement_by_id,
};
use leocard_mahjong::{
    Fan, FanValue, MahjongMatchLength, MahjongMatchProgress, MahjongScoreResult, build_deck,
};
use leocard_protocol::{
    GameEvent, MahjongEvent, MahjongHandResultView, MahjongWinView, MatchId, PlayerId,
};

pub(crate) fn finish(
    fans: &[(Fan, u8)],
    points: u16,
    hand: u8,
    complete: bool,
) -> AchievementTrigger {
    AchievementTrigger::Game {
        player: PlayerId(0),
        event: GameEvent::Mahjong(MahjongEvent::HandFinished {
            result: MahjongHandResultView {
                match_length: MahjongMatchLength::SingleHand,
                match_progress: MahjongMatchProgress {
                    completed_hands: 1,
                    exhaustive_draws: 0,
                },
                winners: vec![MahjongWinView {
                    player: PlayerId(0),
                    from: None,
                    winning_tile: build_deck()[0],
                    wait_kind_count: 1,
                    score: MahjongScoreResult {
                        fans: fans
                            .iter()
                            .map(|&(fan, count)| FanValue {
                                fan,
                                count,
                                points: fan.points() * u16::from(count),
                            })
                            .collect(),
                        points_without_flowers: points,
                        flower_points: 0,
                        total_points: points,
                    },
                }],
                exhaustive_draw: false,
                deltas: [0; 4],
                match_scores: [0; 4],
                match_complete: complete,
                sequence_index: hand,
                reference_changes: vec![],
            },
        }),
    }
}

pub(crate) fn context(match_byte: u8, hand: u32, sequence: u128) -> Option<AchievementContext> {
    Some(AchievementContext {
        match_id: MatchId([match_byte; 16]),
        hand_index: Some(hand),
        sequence,
    })
}

pub(crate) fn achieved(book: &AchievementBook, id: &str) -> bool {
    book.achieved(achievement_by_id(&format!("leocard:mahjong/{id}")).unwrap())
}
