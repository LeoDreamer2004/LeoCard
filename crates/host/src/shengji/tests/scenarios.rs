use super::*;
use leocard_protocol::{ClientCommand, GameCommand, ServerEvent};
use leocard_protocol::{
    GameEvent, GameViolation, PlayerId, RejectReason, ShengjiCommand, ShengjiEvent,
    ShengjiFiveTrumpCrossingStage, ShengjiPhaseView, ShengjiViolation,
};
#[cfg(test)]
use leocard_shengji::build_deck;
use leocard_shengji::{Phase, ShengjiCard, ShengjiPlayerId, ShengjiRuleSet};
use leocard_shengji::{ShengjiRank, ShengjiSuit, build_deck_for};

#[test]
fn bottom_copy_is_public_keeps_the_dealer_and_times_out_as_a_pass() {
    let rules = ShengjiRuleSet {
        bottom_copy: true,
        bottom_flip: true,
        ..ShengjiRuleSet::default()
    };
    let diamond = ShengjiCard::suited(0, ShengjiSuit::Diamond, ShengjiRank::Two);
    let hearts = [
        ShengjiCard::suited(0, ShengjiSuit::Heart, ShengjiRank::Two),
        ShengjiCard::suited(1, ShengjiSuit::Heart, ShengjiRank::Two),
    ];
    let arranged = [
        (0, diamond),
        (
            2,
            ShengjiCard::suited(1, ShengjiSuit::Diamond, ShengjiRank::Two),
        ),
        (
            4,
            ShengjiCard::suited(0, ShengjiSuit::Club, ShengjiRank::Two),
        ),
        (
            6,
            ShengjiCard::suited(1, ShengjiSuit::Club, ShengjiRank::Two),
        ),
        (1, hearts[0]),
        (5, hearts[1]),
        (
            8,
            ShengjiCard::suited(0, ShengjiSuit::Spade, ShengjiRank::Two),
        ),
        (
            10,
            ShengjiCard::suited(1, ShengjiSuit::Spade, ShengjiRank::Two),
        ),
        (12, ShengjiCard::small_joker(0)),
        (14, ShengjiCard::small_joker(1)),
        (17, ShengjiCard::big_joker(0)),
        (19, ShengjiCard::big_joker(1)),
    ];
    let mut deck = build_deck();
    for (target_index, card) in arranged {
        let index = deck
            .iter()
            .position(|candidate| *candidate == card)
            .unwrap();
        deck.swap(target_index, index);
    }
    let (mut session, connections) = started_session_with_deck(rules, deck);
    session.advance_time(DEAL_INTERVAL);
    session.handle(
        connections[0],
        message(
            0,
            5,
            ClientCommand::Game(GameCommand::Shengji(ShengjiCommand::Declare {
                cards: vec![diamond],
            })),
        ),
    );
    session.advance_time(DEAL_INTERVAL * 99);
    session.advance_time(BIDDING_GRACE);
    let buried = session.game().unwrap().players()[0].hand[..rules.kitty_size()].to_vec();
    let inquiry = session.handle(
        connections[0],
        message(
            0,
            6,
            ClientCommand::Game(GameCommand::Shengji(ShengjiCommand::Bury {
                cards: buried.clone(),
            })),
        ),
    );
    let first_burier = game_snapshot(&inquiry, connections[0]);
    assert_eq!(first_burier.your_buried, buried);
    assert!(
        connections[1..]
            .iter()
            .all(|connection| game_snapshot(&inquiry, *connection).your_buried.is_empty())
    );
    assert!(matches!(
        first_burier.phase,
        ShengjiPhaseView::BottomCopying {
            player: PlayerId(1),
            milliseconds_remaining: 10_000,
        }
    ));

    // 正常亮主定庄时，即使房间同时开启扳底配置，抄底仍然有效。
    let mut timeout_session = session.clone();
    let timeout = timeout_session.advance_time(BOTTOM_COPY_DECISION_TIMEOUT);
    assert!(matches!(
        game_snapshot(&timeout, connections[0]).phase,
        ShengjiPhaseView::Playing
    ));

    let copied = session.handle(
        connections[1],
        message(
            1,
            7,
            ClientCommand::Game(GameCommand::Shengji(ShengjiCommand::ChooseBottomCopy {
                cards: Some(hearts.to_vec()),
            })),
        ),
    );
    let copier = game_snapshot(&copied, connections[1]);
    let opponent = game_snapshot(&copied, connections[2]);
    assert_eq!(copier.dealer, Some(PlayerId(0)));
    assert_eq!(copier.trump.unwrap().suit, Some(ShengjiSuit::Heart));
    assert_eq!(copier.your_hand.len(), 33);
    assert!(copier.your_buried.is_empty());
    assert_eq!(opponent.your_hand.len(), 25);
    assert!(opponent.your_buried.is_empty());
    assert!(matches!(
        copier.phase,
        ShengjiPhaseView::BottomCopyBurying {
            player: PlayerId(1)
        }
    ));
    assert!(copied.iter().any(|delivery| matches!(
        &delivery.message.event,
        ServerEvent::GameEvent(GameEvent::Shengji(
            ShengjiEvent::BottomCopied { declaration }
        )) if declaration.player == PlayerId(1) && declaration.cards == hearts
    )));

    let reburied = session.game().unwrap().players()[1].hand[..rules.kitty_size()].to_vec();
    let finished = session.handle(
        connections[1],
        message(
            1,
            8,
            ClientCommand::Game(GameCommand::Shengji(ShengjiCommand::Bury {
                cards: reburied.clone(),
            })),
        ),
    );
    let copier_after_bury = game_snapshot(&finished, connections[1]);
    assert_eq!(copier_after_bury.your_buried, reburied);
    assert!(
        connections
            .iter()
            .enumerate()
            .filter(|(index, _)| *index != 1)
            .all(|(_, connection)| game_snapshot(&finished, *connection).your_buried.is_empty())
    );
    assert!(matches!(
        game_snapshot(&finished, connections[0]).phase,
        ShengjiPhaseView::Playing
    ));
    assert_eq!(session.game().unwrap().dealer(), Some(ShengjiPlayerId(0)));
}

#[test]
fn five_trump_crossing_commands_transfer_privately_and_only_once() {
    let rules = ShengjiRuleSet {
        five_trump_crossing: true,
        ..ShengjiRuleSet::default()
    };
    let (mut session, connections, target) = started_session_with_rules(rules);
    session.advance_time(DEAL_INTERVAL);
    session.handle(
        connections[0],
        message(
            0,
            5,
            ClientCommand::Game(GameCommand::Shengji(ShengjiCommand::Declare {
                cards: vec![target],
            })),
        ),
    );
    session.advance_time(DEAL_INTERVAL * 99);
    session.advance_time(BIDDING_GRACE);

    let game = session.game().unwrap();
    let trump = game.trump().unwrap();
    let dealer_hand = game.players()[0].hand.clone();
    let trump_count = dealer_hand
        .iter()
        .filter(|card| trump.is_trump(**card))
        .count();
    let trump_to_bury = trump_count.saturating_sub(5);
    assert!(trump_to_bury <= rules.kitty_size());
    let mut buried = dealer_hand
        .iter()
        .copied()
        .filter(|card| trump.is_trump(*card))
        .take(trump_to_bury)
        .collect::<Vec<_>>();
    let side_cards_to_bury = rules.kitty_size() - buried.len();
    buried.extend(
        dealer_hand
            .iter()
            .copied()
            .filter(|card| !trump.is_trump(*card))
            .take(side_cards_to_bury),
    );
    assert_eq!(buried.len(), rules.kitty_size());
    let deliveries = session.handle(
        connections[0],
        message(
            0,
            6,
            ClientCommand::Game(GameCommand::Shengji(ShengjiCommand::Bury { cards: buried })),
        ),
    );
    let snapshot = game_snapshot(&deliveries, connections[0]);
    let ShengjiPhaseView::FiveTrumpCrossing { eligible, .. } = snapshot.phase else {
        panic!("有主且庄家只剩五张主牌时应进入五主过江阶段");
    };
    assert!(eligible.contains(&PlayerId(0)));
    let mut waiting_for_decision = session.clone();
    assert!(
        waiting_for_decision
            .advance_time(Duration::from_secs(90))
            .is_empty()
    );
    assert!(matches!(
        waiting_for_decision.game().unwrap().phase(),
        Phase::FiveTrumpCrossing
    ));

    let hand = session.game().unwrap().players()[0].hand.clone();
    let mut outgoing = hand
        .iter()
        .copied()
        .filter(|card| trump.is_trump(*card))
        .collect::<Vec<_>>();
    let filler_count = 5 - outgoing.len();
    outgoing.extend(
        hand.iter()
            .copied()
            .filter(|card| !trump.is_trump(*card))
            .take(filler_count),
    );
    assert_eq!(outgoing.len(), 5);
    let mut final_decision = session.handle(
        connections[0],
        message(
            0,
            7,
            ClientCommand::Game(GameCommand::Shengji(
                ShengjiCommand::ChooseFiveTrumpCrossing {
                    cards: Some(outgoing.clone()),
                },
            )),
        ),
    );
    for player in eligible.into_iter().filter(|player| *player != PlayerId(0)) {
        final_decision = session.handle(
            connections[usize::from(player.0)],
            message(
                player.0,
                10,
                ClientCommand::Game(GameCommand::Shengji(
                    ShengjiCommand::ChooseFiveTrumpCrossing { cards: None },
                )),
            ),
        );
    }
    let partner = game_snapshot(&final_decision, connections[2]);
    assert!(outgoing.iter().all(|card| partner.your_hand.contains(card)));
    let opponent = game_snapshot(&final_decision, connections[1]);
    assert!(
        outgoing
            .iter()
            .any(|card| !opponent.your_hand.contains(card))
    );
    assert!(matches!(
        partner.phase,
        ShengjiPhaseView::FiveTrumpCrossing {
            stage: ShengjiFiveTrumpCrossingStage::Returning,
            ..
        }
    ));

    let mut waiting_for_return = session.clone();
    assert!(
        waiting_for_return
            .advance_time(Duration::from_secs(90))
            .is_empty()
    );
    assert!(matches!(
        waiting_for_return.game().unwrap().phase(),
        Phase::FiveTrumpCrossing
    ));

    let deliveries = session.handle(
        connections[2],
        message(
            2,
            11,
            ClientCommand::Game(GameCommand::Shengji(
                ShengjiCommand::ReturnFiveTrumpCrossing {
                    cards: outgoing.clone(),
                },
            )),
        ),
    );
    assert!(matches!(
        game_snapshot(&deliveries, connections[0]).phase,
        ShengjiPhaseView::Playing
    ));
    let rejected = session.handle(
        connections[0],
        message(
            0,
            12,
            ClientCommand::Game(GameCommand::Shengji(
                ShengjiCommand::ChooseFiveTrumpCrossing { cards: None },
            )),
        ),
    );
    assert!(rejected.iter().any(|delivery| matches!(
        delivery.message.event,
        ServerEvent::Rejected {
            reason: RejectReason::Game(GameViolation::Shengji(ShengjiViolation::WrongPhase))
        }
    )));
}

#[test]
fn power_outage_opens_ten_second_round_then_bottom_flip_is_publicly_held() {
    let rules = ShengjiRuleSet {
        power_outage_dealer: true,
        bottom_flip: true,
        ..ShengjiRuleSet::default()
    };
    let (mut session, connections, _) = started_session_with_rules(rules);
    let deliveries = session.advance_time(DEAL_INTERVAL * 100);
    let before = game_snapshot(&deliveries, connections[0]);
    assert!(matches!(
        before.phase,
        ShengjiPhaseView::BiddingGrace {
            power_outage: false,
            ..
        }
    ));
    let hand_lengths = before
        .players
        .iter()
        .map(|player| player.hand_len)
        .collect::<Vec<_>>();

    let deliveries = session.advance_time(BIDDING_GRACE);
    let outage = game_snapshot(&deliveries, connections[0]);
    assert_eq!(outage.dealer, Some(PlayerId(1)));
    assert_eq!(outage.bidding_level, ShengjiRank::Two);
    assert!(matches!(
        outage.phase,
        ShengjiPhaseView::BiddingGrace {
            milliseconds_remaining: 10_000,
            power_outage: true,
            ..
        }
    ));
    assert_eq!(
        outage
            .players
            .iter()
            .map(|player| player.hand_len)
            .collect::<Vec<_>>(),
        hand_lengths
    );
    assert!(
        session
            .room
            .players
            .iter()
            .all(|player| player.reference_points == 0)
    );

    let deliveries = session.advance_time(POWER_OUTAGE_BIDDING_GRACE);
    assert!(matches!(
        game_snapshot(&deliveries, connections[0]).phase,
        ShengjiPhaseView::BottomFlipping { reveal: None }
    ));
    let deliveries = session.advance_time(BOTTOM_FLIP_START_DELAY);
    let reveal = game_snapshot(&deliveries, connections[0]);
    let ShengjiPhaseView::BottomFlipping {
        reveal: Some(reveal_view),
    } = reveal.phase
    else {
        panic!("扳底应公开当前底牌及所有同牌");
    };
    assert_eq!(
        reveal_view.card,
        ShengjiCard::suited(1, ShengjiSuit::Spade, ShengjiRank::Nine)
    );
    assert_eq!(reveal_view.dealer, Some(PlayerId(2)));
    assert_eq!(reveal_view.matches.len(), 1);
    assert_eq!(reveal_view.matches[0].player, PlayerId(2));
    assert_eq!(reveal.dealer, Some(PlayerId(2)));
    assert_eq!(reveal.trump.unwrap().suit, Some(ShengjiSuit::Spade));

    // 旧的 1.4 秒节奏会在玩家交互演出完成前收起翻牌；
    // 现在到这个时点仍必须保持公开。
    session.advance_time(Duration::from_millis(1400));
    assert!(matches!(
        session.game_snapshot(PlayerId(0)).phase,
        ShengjiPhaseView::BottomFlipping { reveal: Some(_) }
    ));

    let deliveries = session.advance_time(BOTTOM_FLIP_HOLD_DURATION - Duration::from_millis(1400));
    assert!(matches!(
        game_snapshot(&deliveries, connections[0]).phase,
        ShengjiPhaseView::Burying
    ));
}

#[test]
fn three_deck_session_requires_and_accepts_the_rule_selected_deck() {
    let rules = ShengjiRuleSet {
        deck_count: 3,
        ..ShengjiRuleSet::default()
    };
    assert_eq!(
        ShengjiSession::new(ROOM, rules, build_deck()).unwrap_err(),
        HostError::InvalidDeckSize {
            expected: 162,
            actual: 108,
        }
    );
    let session = ShengjiSession::new(ROOM, rules, build_deck_for(3)).unwrap();
    assert_eq!(session.rules().deck_count, 3);
}

#[test]
fn four_deck_session_requires_and_accepts_the_rule_selected_deck() {
    let rules = ShengjiRuleSet {
        deck_count: 4,
        ..ShengjiRuleSet::default()
    };
    assert_eq!(
        ShengjiSession::new(ROOM, rules, build_deck_for(3)).unwrap_err(),
        HostError::InvalidDeckSize {
            expected: 216,
            actual: 162,
        }
    );
    let session = ShengjiSession::new(ROOM, rules, build_deck_for(4)).unwrap();
    assert_eq!(session.rules().deck_count, 4);
}
