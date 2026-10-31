use super::support::{DEAL_INTERVAL, started_session};
use leocard_game_common::PlayerId;

use leocard_shengji::{
    ShengjiCard, ShengjiGameState, ShengjiPhase, ShengjiRank, ShengjiRuleSet, ShengjiSuit,
    ShengjiTeamProgress, build_deck,
};

#[test]
fn dealer_partner_bot_can_bid_when_someone_else_receives_the_final_card() {
    let (mut session, _, _) = started_session();
    let rules = ShengjiRuleSet::default();
    let mut deck = build_deck();
    // Give the partner two heart level cards early, but it must wait for all four hands.
    for (position, copy) in [(2, 0), (6, 1)] {
        let index = deck
            .iter()
            .position(|card| {
                *card == ShengjiCard::suited(copy, ShengjiSuit::Heart, ShengjiRank::Two)
            })
            .unwrap();
        deck.swap(position, index);
    }
    session.game = Some(
        ShengjiGameState::new(
            rules,
            ShengjiTeamProgress::default(),
            Some(PlayerId(0)),
            PlayerId(0),
            deck,
        )
        .unwrap(),
    );
    session.room.players[2].is_bot = true;
    session.advance_time(DEAL_INTERVAL * 99);
    assert!(session.game.as_ref().unwrap().bidding().current().is_none());
    session.advance_time(DEAL_INTERVAL);
    let game = session.game.as_ref().unwrap();
    assert_eq!(game.phase(), &ShengjiPhase::BiddingGrace);
    assert_eq!(game.bidding().current().unwrap().player, PlayerId(2));
}
