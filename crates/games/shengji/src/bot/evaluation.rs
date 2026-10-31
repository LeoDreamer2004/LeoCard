use super::candidates::hand_value;
use crate::ShengjiTeamId;
use crate::{
    ShengjiCard, ShengjiCategory, ShengjiClassifiedPlay, ShengjiTrickRecord, ShengjiTrump,
    compare_for_trick,
};
use leocard_game_common::PlayerId;

use std::cmp::Ordering;

pub(super) fn quick_value(
    play: &ShengjiClassifiedPlay,
    hand: &[ShengjiCard],
    trick: Option<&ShengjiTrickRecord>,
    player: PlayerId,
    trump: ShengjiTrump,
) -> f64 {
    let remaining: Vec<_> = hand
        .iter()
        .copied()
        .filter(|card| !play.cards.contains(card))
        .collect();
    let cost = hand_value(hand, trump) - hand_value(&remaining, trump);
    let points = play
        .cards
        .iter()
        .map(|card| f64::from(card.points()))
        .sum::<f64>();
    if let Some(trick) = trick {
        let current = &trick
            .plays
            .iter()
            .find(|(actor, _)| *actor == trick.winner)
            .expect("winner")
            .1;
        let wins = compare_for_trick(&trick.plays[0].1, current, play, trump) == Ordering::Greater;
        let friendly = ShengjiTeamId::of(trick.winner) == ShengjiTeamId::of(player);
        let secure = friendly
            && (trick.plays.len() == 3
                || current.strongest_component().comparison_strength() >= 12);
        let captured = f64::from(trick.points) + points;
        let gain = if wins {
            captured + 3.0
        } else if secure {
            points
        } else {
            -points
        };
        gain - cost * if secure { 1.4 } else { 0.65 }
    } else {
        let strength = play.strongest_component().comparison_strength();
        // 副牌先兑现大牌；吊主则保留控场牌，优先用小主消耗其他人的主牌。
        // 这是搜索先验，确有得分或保底收益时，模拟仍可选择大主。
        let strength_value = f64::from(strength)
            * if play.category == ShengjiCategory::Trump {
                -0.65
            } else {
                1.3
            };
        strength_value + play.cards.len() as f64 * 2.0 - cost * 0.8 - points * 0.12
    }
}

#[cfg(test)]
mod tests {
    use super::super::candidates::plays;
    use super::*;
    use crate::{ShengjiRank, ShengjiSuit};

    fn trump() -> ShengjiTrump {
        ShengjiTrump::new(ShengjiRank::Ten, Some(ShengjiSuit::Heart)).unwrap()
    }

    fn best(hand: &[ShengjiCard], trick: Option<&ShengjiTrickRecord>) -> Vec<ShengjiCard> {
        let lead = trick
            .and_then(|trick| trick.plays.first())
            .map(|(_, play)| play);
        plays(hand, lead, trump(), 64)
            .into_iter()
            .max_by(|left, right| {
                quick_value(left, hand, trick, PlayerId(0), trump()).total_cmp(&quick_value(
                    right,
                    hand,
                    trick,
                    PlayerId(0),
                    trump(),
                ))
            })
            .unwrap()
            .cards
    }

    #[test]
    fn drawing_trumps_preserves_jokers_but_side_suits_still_cash_aces() {
        let low = ShengjiCard::suited(0, ShengjiSuit::Heart, ShengjiRank::Three);
        let hand = [
            low,
            ShengjiCard::small_joker(0),
            ShengjiCard::big_joker(0),
            ShengjiCard::big_joker(1),
        ];
        assert_eq!(best(&hand, None), vec![low]);
        let ace = ShengjiCard::suited(0, ShengjiSuit::Spade, ShengjiRank::Ace);
        let hand = [
            ace,
            ShengjiCard::suited(0, ShengjiSuit::Spade, ShengjiRank::Three),
        ];
        assert_eq!(best(&hand, None), vec![ace]);
    }

    #[test]
    fn following_can_still_spend_a_joker_to_take_opponents_points() {
        let king = ShengjiCard::suited(0, ShengjiSuit::Heart, ShengjiRank::King);
        let lead = plays(&[king], None, trump(), 1).remove(0);
        let trick = ShengjiTrickRecord {
            leader: PlayerId(1),
            winner: PlayerId(1),
            plays: vec![(PlayerId(1), lead)],
            points: 10,
        };
        let joker = ShengjiCard::big_joker(0);
        let hand = [
            ShengjiCard::suited(0, ShengjiSuit::Heart, ShengjiRank::Three),
            joker,
        ];
        assert_eq!(best(&hand, Some(&trick)), vec![joker]);
    }
}
