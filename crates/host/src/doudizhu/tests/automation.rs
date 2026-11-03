use super::*;
use leocard_doudizhu::auto_play_action;
use std::thread;
use std::time::Instant;

#[test]
fn human_takeover_keeps_the_existing_policy_while_robots_use_workers() {
    for mode in [DoudizhuMode::Three, DoudizhuMode::Four] {
        let mut session = started(mode, false);
        let player = session.game.as_ref().unwrap().current;
        session.apply_action(player, DoudizhuAction::Bid(3), None);
        let expected = auto_play_action(session.game.as_ref().unwrap());
        for (auto_play, connected, left) in [
            (true, true, false),
            (false, false, false),
            (false, false, true),
        ] {
            let participant = &mut session.room.players[usize::from(player)];
            participant.auto_play = auto_play;
            participant.connected = connected;
            participant.left = left;
            assert_eq!(session.choose_automatic_action(), expected);
        }
        let participant = &mut session.room.players[usize::from(player)];
        participant.is_bot = true;
        participant.left = false;
        participant.connected = true;
        assert!(session.choose_automatic_action().is_none());
        let start = Instant::now();
        let action = loop {
            if let Some(action) = session.choose_automatic_action() {
                break action;
            }
            assert!(start.elapsed() < Duration::from_secs(5));
            thread::sleep(Duration::from_millis(2));
        };
        session.apply_action(player, action.clone(), None);
        assert_eq!(session.bot_history, vec![(player, action)]);
    }
}

#[test]
fn four_player_bidding_keeps_the_existing_policy() {
    let mut session = started(DoudizhuMode::Four, false);
    let player = session.game.as_ref().unwrap().current;
    session.room.players[usize::from(player)].is_bot = true;
    assert_eq!(
        session.choose_automatic_action(),
        bot_action(session.game.as_ref().unwrap())
    );
}
