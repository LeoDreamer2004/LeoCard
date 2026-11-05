use super::super::super::{TablePlayer, TexasAdapter, TexasSession};
use super::super::{
    HOST_CONNECTION, ROOM, SECOND_CONNECTION, THIRD_CONNECTION, connection_for, join_command,
    message,
};
use crate::Delivery;
use leocard_game_common::PlayerId;
use leocard_protocol::{
    ClientCommand, MatchId, ProfileId, SeatId, ServerEvent, TexasCommand, TexasEvent,
    TexasPhaseView,
};
use leocard_texas_holdem::{
    TexasAction, TexasCard, TexasRank, TexasRuleSet, TexasSuit, build_deck,
};

pub(super) const HOST: PlayerId = PlayerId(20);
pub(super) const LEFT: PlayerId = PlayerId(30);
pub(super) const RIGHT: PlayerId = PlayerId(10);

pub(super) fn rules() -> TexasRuleSet {
    TexasRuleSet {
        side_pots: false,
        run_it_twice: true,
        ..TexasRuleSet::default()
    }
}

pub(super) fn adapter() -> TexasAdapter {
    let players = [(RIGHT, 2), (HOST, 0), (LEFT, 1)]
        .into_iter()
        .map(|(id, seat)| TablePlayer {
            id,
            profile_id: ProfileId([id.0; 32]),
            name: format!("player{}", id.0),
            avatar: None,
            seat: SeatId(seat),
            connected: true,
            reference_points: 0,
            completed_games: 0,
        })
        .collect();
    let mut adapter = TexasAdapter::new(
        MatchId([7; 16]),
        52300,
        HOST,
        players,
        rules(),
        HOST,
        build_deck(false),
    )
    .unwrap();
    post_blinds(&mut adapter);
    adapter
}

pub(super) fn post_blinds(game: &mut TexasAdapter) {
    for _ in 0..2 {
        game.act(game.current_player().unwrap(), TexasAction::PostBlind)
            .unwrap();
    }
}

pub(super) fn start_voting(game: &mut TexasAdapter) -> Vec<TexasEvent> {
    game.act(game.current_player().unwrap(), TexasAction::AllIn)
        .unwrap();
    game.act(game.current_player().unwrap(), TexasAction::Call)
        .unwrap();
    game.act(game.current_player().unwrap(), TexasAction::Call)
        .unwrap()
}

// Clockwise from the dealer after one uncontested hand: [27, 19, 14].
// The dealer need not occupy core index 0 in a randomly seated room.
// Next preflop starts with 19, followed by a short shove for 14 and
// the large stack's long call for 19.
pub(super) fn unequal_stacks(game: &mut TexasAdapter) {
    for action in [
        TexasAction::RaiseTo(6),
        TexasAction::Fold,
        TexasAction::Call,
        TexasAction::Fold,
    ] {
        game.act(game.current_player().unwrap(), action).unwrap();
    }
    assert_eq!(
        (0..game.players().len())
            .map(|offset| {
                game.game().players()
                    [(game.game().dealer().index() + offset) % game.players().len()]
                .stack()
            })
            .collect::<Vec<_>>(),
        vec![27, 19, 14]
    );
    game.start_next_hand(build_deck(false)).unwrap();
    post_blinds(game);
}

pub(super) fn session() -> TexasSession {
    session_with_deck(build_deck(false))
}

pub(super) fn session_with_deck(deck: Vec<TexasCard>) -> TexasSession {
    session_with_setup(deck, None)
}

pub(super) fn session_with_seats(seats: [SeatId; 3]) -> TexasSession {
    session_with_setup(build_deck(false), Some(seats))
}

fn session_with_setup(deck: Vec<TexasCard>, seats: Option<[SeatId; 3]>) -> TexasSession {
    let mut session = TexasSession::new(ROOM, 52300, rules(), deck).unwrap();
    for (connection, name, token) in [
        (HOST_CONNECTION, "host", 1),
        (SECOND_CONNECTION, "left", 2),
        (THIRD_CONNECTION, "right", 3),
    ] {
        session.handle(connection, message(1, join_command(name, token)));
    }
    if let Some(seats) = seats {
        // Pin lobby seats only; normal startup must derive the core table order.
        for (participant, seat) in session.room.players.iter_mut().zip(seats) {
            participant.seat = Some(seat);
        }
    }
    for connection in [SECOND_CONNECTION, THIRD_CONNECTION] {
        session.handle(
            connection,
            message(2, ClientCommand::SetReady { ready: true }),
        );
    }
    session.handle(HOST_CONNECTION, message(2, ClientCommand::StartGame));
    post_blinds(session.game.as_mut().unwrap());
    assert_eq!(session.room_id(), ROOM);
    session
}

pub(super) fn command(
    session: &mut TexasSession,
    request: u64,
    player: PlayerId,
    command: TexasCommand,
) -> Vec<Delivery> {
    let connection = connection_for(session, player);
    session.handle(
        connection,
        message(request, ClientCommand::Game(command.into())),
    )
}

pub(super) fn vote(session: &mut TexasSession, request: u64, twice: bool) -> Vec<Delivery> {
    let player = session.game().unwrap().current_player().unwrap();
    let deliveries = command(
        session,
        request,
        player,
        TexasCommand::ChooseRunout { twice },
    );
    assert!(
        !deliveries
            .iter()
            .any(|d| matches!(d.message.event, ServerEvent::Rejected { .. }))
    );
    deliveries
}

pub(super) fn voting_session() -> TexasSession {
    voting_session_with_deck(build_deck(false))
}

pub(super) fn voting_session_with_deck(deck: Vec<TexasCard>) -> TexasSession {
    let mut session = session_with_deck(deck);
    for (request, action) in [
        (10, TexasAction::AllIn),
        (11, TexasAction::Call),
        (12, TexasAction::Call),
    ] {
        let player = session.game().unwrap().current_player().unwrap();
        let deliveries = command(&mut session, request, player, TexasCommand::Act { action });
        assert!(
            !deliveries
                .iter()
                .any(|d| matches!(d.message.event, ServerEvent::Rejected { .. }))
        );
    }
    session
}

// The dealer has AA, the other players KK and QQ. The dealer wins both boards,
// making a pair on the first and three of a kind on the second.
pub(super) fn winning_deck(swap_boards: bool) -> Vec<TexasCard> {
    let mut prefix = [
        (TexasSuit::Spade, TexasRank::King),
        (TexasSuit::Heart, TexasRank::Queen),
        (TexasSuit::Club, TexasRank::Ace),
        (TexasSuit::Diamond, TexasRank::King),
        (TexasSuit::Club, TexasRank::Queen),
        (TexasSuit::Diamond, TexasRank::Ace),
        (TexasSuit::Club, TexasRank::Two),
        (TexasSuit::Diamond, TexasRank::Three),
        (TexasSuit::Heart, TexasRank::Seven),
        (TexasSuit::Spade, TexasRank::Eight),
        (TexasSuit::Club, TexasRank::Nine),
        (TexasSuit::Diamond, TexasRank::Four),
        (TexasSuit::Club, TexasRank::Five),
        (TexasSuit::Spade, TexasRank::Seven),
        (TexasSuit::Heart, TexasRank::Nine),
        (TexasSuit::Heart, TexasRank::Ace),
    ]
    .map(|(suit, rank)| TexasCard::new(suit, rank));
    if swap_boards {
        for index in 0..5 {
            prefix.swap(6 + index, 11 + index);
        }
    }
    prefix
        .into_iter()
        .chain(
            build_deck(false)
                .into_iter()
                .filter(|card| !prefix.contains(card)),
        )
        .collect()
}

pub(super) fn assert_hand_completion(session: &TexasSession, tournament_complete: bool) {
    let host = session.room.player_id(HOST_CONNECTION).unwrap();
    let snapshot = session.game_snapshot(host);
    assert!(matches!(snapshot.phase, TexasPhaseView::HandComplete {
        tournament_complete: complete, ref reference_changes, ..
    } if complete == tournament_complete
        && reference_changes.len() == if complete { 3 } else { 0 }));
    assert_eq!(snapshot.second_community.len(), 5);
    let mut stacks = snapshot.players.iter().map(|p| p.stack).collect::<Vec<_>>();
    stacks.sort_unstable();
    assert_eq!(
        stacks,
        if tournament_complete {
            vec![0, 0, 60]
        } else {
            vec![20, 20, 20]
        }
    );
    for participant in &session.room.players {
        // Hand-level counters are tracked immediately, but persistent profiles
        // are merged only at tournament completion, not after a tied hand.
        let current = &session.match_profile_stats[participant.id.index()];
        assert_eq!(current.hands_played, 1);
        assert_eq!(current.hand_category_counts.iter().sum::<u32>(), 1);
        assert_eq!(
            participant.ready,
            participant.is_bot || participant.auto_play
        );
        if tournament_complete {
            let profile = participant.game_profiles.texas.as_ref().unwrap();
            assert_eq!(profile.hands_played, 1);
            assert_eq!(profile.completed_games, 1);
            assert_eq!(profile.hand_category_counts, current.hand_category_counts);
        } else {
            assert!(participant.game_profiles.texas.is_none());
        }
    }
}
