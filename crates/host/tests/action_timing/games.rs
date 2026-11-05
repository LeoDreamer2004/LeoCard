use super::*;
use leocard_doudizhu::{DoudizhuAction, DoudizhuMode, DoudizhuRuleSet};
use leocard_guandan::GuandanAction;
use leocard_mahjong_link::MahjongLinkAction;
use leocard_protocol::{
    DoudizhuCommand, GameCommand, GuandanCommand, MahjongCommand, MahjongLinkCommand,
    McGuessCommand, QiGui523Command, ShengjiCommand, ShengjiPhaseView, TexasCommand, UnoCommand,
};
use leocard_qigui523::QiGuiRuleSet;
use leocard_texas_holdem::TexasAction;
use leocard_uno::UnoColor;

fn rules() -> [GameRules; 10] {
    [
        GameRules::QiGui523(QiGuiRuleSet {
            player_count: 4,
            ..Default::default()
        }),
        GameRules::Texas(Default::default()),
        GameRules::Shengji(Default::default()),
        GameRules::Uno(Default::default()),
        GameRules::Mahjong(Default::default()),
        GameRules::McGuess(Default::default()),
        GameRules::Blackjack(Default::default()),
        GameRules::Doudizhu(DoudizhuRuleSet {
            mode: DoudizhuMode::Four,
            ..Default::default()
        }),
        GameRules::Guandan(Default::default()),
        GameRules::MahjongLink(Default::default()),
    ]
}

fn decision(snapshot: &GameSnapshot) -> GameCommand {
    match snapshot {
        GameSnapshot::QiGui523(s) => QiGui523Command::PlayCards {
            cards: vec![s.starting_card.card],
        }
        .into(),
        GameSnapshot::Texas(_) => TexasCommand::Act {
            action: TexasAction::Fold,
        }
        .into(),
        GameSnapshot::Shengji(s) => {
            assert!(matches!(s.phase, ShengjiPhaseView::BiddingGrace { .. }));
            ShengjiCommand::ConfirmBidPass.into()
        }
        GameSnapshot::Uno(s) => {
            if s.current_color.is_none() {
                UnoCommand::ChooseInitialColor {
                    color: UnoColor::Red,
                }
                .into()
            } else if s.pending_draw > 0 {
                UnoCommand::AcceptDrawPenalty.into()
            } else if s.pending_skip > 0 {
                UnoCommand::ResolveSkip.into()
            } else {
                UnoCommand::DrawCard.into()
            }
        }
        GameSnapshot::Mahjong(s) => MahjongCommand::Discard {
            tile: s.your_hand[0],
        }
        .into(),
        GameSnapshot::McGuess(_) => McGuessCommand::Skip.into(),
        GameSnapshot::Blackjack(_) => BlackjackCommand::Act {
            action: BlackjackAction::Bet { amount: 2 },
        }
        .into(),
        GameSnapshot::Doudizhu(_) => DoudizhuCommand::Act(DoudizhuAction::Bid(1)).into(),
        GameSnapshot::Guandan(s) => {
            GuandanCommand::Act(GuandanAction::Play(vec![s.your_hand[0]])).into()
        }
        GameSnapshot::MahjongLink(_) => MahjongLinkCommand::Act {
            action: MahjongLinkAction::Pass,
        }
        .into(),
    }
}

#[test]
fn all_ten_dispatchers_record_only_the_required_human_and_publish_committed_stats() {
    for rules in rules() {
        let mut table = Table::new_for(rules);
        let mut view = table.view();
        // Deal/reveal animations are not decisions. Progress in small steps so that
        // simultaneous confirmation windows are observed before their timeout.
        for _ in 0..500 {
            if !view.1.clocks.is_empty() {
                break;
            }
            table.session.advance_time(Duration::from_millis(100));
            view = table.view();
        }
        assert!(
            !view.1.clocks.is_empty(),
            "no decision for {:?}",
            view.0.kind()
        );
        let kind = view.0.kind();
        let actor = view.1.clocks[0].player;
        let connection = u64::from(actor.0) + 1;
        let command = decision(&table.view_for(connection).0);
        let before = table
            .view_for(connection)
            .1
            .clocks
            .into_iter()
            .find(|c| c.player == actor)
            .unwrap()
            .elapsed_millis;
        table.session.advance_time(Duration::from_millis(750));
        let accepted = table.send(connection, ClientCommand::Game(command));
        assert!(
            !accepted
                .iter()
                .any(|d| matches!(d.message.event, ServerEvent::Rejected { .. })),
            "{kind:?}: {accepted:?}"
        );
        // This must already be in the command's own snapshots, not just a later poll.
        for delivery in &accepted {
            if let ServerEvent::GameSnapshot(snapshot) = &delivery.message.event {
                let times = snapshot.player(actor).unwrap().game_profiles.action_times;
                let stats = times.get(kind);
                assert_eq!(
                    (
                        stats.total_millis,
                        stats.completed_actions,
                        stats.longest_millis
                    ),
                    (before + 750, 1, before + 750),
                    "{kind:?}"
                );
                assert_eq!(times.longest_millis(), before + 750);
            }
        }
        let (snapshot, clocks) = table.view();
        for player in snapshot.players() {
            let stats = player.game_profiles.action_times.get(kind);
            assert_eq!(
                stats.completed_actions,
                u64::from(player.id == actor),
                "{kind:?}"
            );
        }
        if let Some(next) = clocks.clocks.iter().find(|c| c.player == actor) {
            assert_eq!(
                next.elapsed_millis, 0,
                "same actor starts a new decision in {kind:?}"
            );
        }
    }
}

#[test]
fn simultaneous_shengji_confirmations_keep_other_players_elapsed_windows() {
    let mut table = Table::new_for(GameRules::Shengji(Default::default()));
    for _ in 0..500 {
        if table.view().1.clocks.len() == 4 {
            break;
        }
        table.session.advance_time(Duration::from_millis(100));
    }
    let initial = table.view().1;
    assert_eq!(initial.clocks.len(), 4);
    let first = initial.clocks[0];
    let second = initial.clocks[1];
    table.session.advance_time(Duration::from_millis(500));
    table.send(
        u64::from(first.player.0) + 1,
        ClientCommand::Game(ShengjiCommand::ConfirmBidPass.into()),
    );
    let next = table.view().1;
    assert_eq!(next.clocks.len(), 3);
    assert_eq!(
        next.clocks
            .iter()
            .find(|c| c.player == second.player)
            .unwrap()
            .elapsed_millis,
        second.elapsed_millis + 500
    );
    table.session.advance_time(Duration::from_millis(750));
    table.send(
        u64::from(second.player.0) + 1,
        ClientCommand::Game(ShengjiCommand::ConfirmBidPass.into()),
    );
    let stats = table
        .view()
        .0
        .player(second.player)
        .unwrap()
        .game_profiles
        .action_times
        .get(GameKind::Shengji);
    assert_eq!(
        (stats.total_millis, stats.completed_actions),
        (second.elapsed_millis + 1250, 1)
    );
}

#[test]
fn guandan_accepts_a_minute_sample_without_exposing_private_cards_in_timing() {
    let mut table = Table::new_for(GameRules::Guandan(Default::default()));
    let actor = table.view().1.clocks[0].player;
    let connection = u64::from(actor.0) + 1;
    table.session.advance_time(Duration::from_millis(60_250));
    let (snapshot, timing) = table.view_for(connection);
    assert_eq!(timing.clocks[0].elapsed_millis, 60_250);
    let accepted = table.send(connection, ClientCommand::Game(decision(&snapshot)));
    assert!(
        !accepted
            .iter()
            .any(|d| matches!(d.message.event, ServerEvent::Rejected { .. }))
    );
    let stats = table
        .view()
        .0
        .player(actor)
        .unwrap()
        .game_profiles
        .action_times
        .get(GameKind::Guandan);
    assert_eq!(
        (
            stats.total_millis,
            stats.completed_actions,
            stats.longest_millis
        ),
        (60_250, 1, 60_250)
    );
    // Different recipients have different hands but the exact same public timing DTO.
    assert_eq!(table.view_for(1).1, table.view_for(2).1);
}

#[test]
fn mahjong_link_push_neither_completes_nor_restarts_the_remove_or_skip_window() {
    let mut table = Table::new_for(GameRules::MahjongLink(Default::default()));
    // Remove an outside pair to make a legal row push possible on the next turn.
    let GameSnapshot::MahjongLink(snapshot) = table.view().0 else {
        unreachable!()
    };
    let (first, second) = snapshot.board.best_pair().expect("initial pair");
    let first_action = MahjongLinkAction::Match { first, second };
    let actor = snapshot.current_player.unwrap();
    table.send(
        u64::from(actor.0) + 1,
        ClientCommand::Game(
            MahjongLinkCommand::Act {
                action: first_action,
            }
            .into(),
        ),
    );
    let GameSnapshot::MahjongLink(snapshot) = table.view().0 else {
        unreachable!()
    };
    let push = (0..8)
        .flat_map(|row| {
            [
                leocard_mahjong_link::PushSide::Left,
                leocard_mahjong_link::PushSide::Right,
            ]
            .map(move |from| (row, from))
        })
        .find_map(|(row, from)| {
            (snapshot.board.maximum_push(row, from) > 0).then_some(MahjongLinkAction::Push {
                row,
                from,
                distance: 1,
            })
        })
        .expect("space after removal");
    let actor = snapshot.current_player.unwrap();
    let connection = u64::from(actor.0) + 1;
    table.session.advance_time(Duration::from_secs(2));
    let pushed = table.send(
        connection,
        ClientCommand::Game(MahjongLinkCommand::Act { action: push }.into()),
    );
    assert!(
        !pushed
            .iter()
            .any(|d| matches!(d.message.event, ServerEvent::Rejected { .. }))
    );
    let (snapshot, timing) = table.view();
    assert_eq!(timing.clocks[0].elapsed_millis, 2000);
    assert_eq!(
        snapshot
            .player(actor)
            .unwrap()
            .game_profiles
            .action_times
            .get(GameKind::MahjongLink)
            .completed_actions,
        0
    );
    table.session.advance_time(Duration::from_secs(1));
    table.send(
        connection,
        ClientCommand::Game(
            MahjongLinkCommand::Act {
                action: MahjongLinkAction::Pass,
            }
            .into(),
        ),
    );
    let stats = table
        .view()
        .0
        .player(actor)
        .unwrap()
        .game_profiles
        .action_times
        .get(GameKind::MahjongLink);
    assert_eq!(
        (
            stats.total_millis,
            stats.completed_actions,
            stats.longest_millis
        ),
        (3000, 1, 3000)
    );
}
