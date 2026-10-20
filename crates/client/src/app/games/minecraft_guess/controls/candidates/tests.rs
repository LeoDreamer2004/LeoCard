use super::{CandidateQuery, MinecraftGuessUiState};
use leocard_minecraft_guess::{
    Catalogue, EntryKind, GraphCombo, GraphNodeView, GraphView, GuessMode, MatchState,
    MinecraftGuessRuleSet, RoundView,
};
use leocard_protocol::{
    LobbyPlayer, MatchId, MinecraftGuessSnapshot, PlayerGameProfiles, PlayerId, ProfileId, SeatId,
};

fn snapshot() -> MinecraftGuessSnapshot {
    let rules = MinecraftGuessRuleSet {
        mode: GuessMode::Graph,
        ..Default::default()
    };
    MinecraftGuessSnapshot {
        match_id: MatchId([1; 16]),
        host_port: 52300,
        you: PlayerId(0),
        host: PlayerId(0),
        players: vec![LobbyPlayer {
            id: PlayerId(0),
            profile_id: ProfileId([1; 32]),
            name: "玩家".to_owned(),
            avatar: None,
            seat: Some(SeatId(0)),
            ready: false,
            connected: true,
            reference_points: 0,
            completed_games: 0,
            game_profiles: PlayerGameProfiles::default(),
        }],
        rules,
        progress: MatchState::new(vec![0, 1], rules).progress(),
        round: RoundView {
            completed: false,
            kind: EntryKind::Block,
            character_count: 0,
            current_player: Some(0),
            guesses: Vec::new(),
            skipped_players: Vec::new(),
            answer: None,
            winner: None,
        },
        graph: Some(GraphView {
            nodes: vec![
                GraphNodeView {
                    number: 1,
                    kind: Some(EntryKind::Block),
                    character_count: None,
                    entry: Catalogue::bundled()
                        .find(EntryKind::Block, "minecraft:blue_wool")
                        .cloned(),
                    tag_name: None,
                },
                GraphNodeView {
                    number: 2,
                    kind: Some(EntryKind::Block),
                    character_count: Some(4),
                    entry: None,
                    tag_name: None,
                },
            ],
            edges: Vec::new(),
            selected: 1,
            combo: Some(GraphCombo {
                count: 1,
                targets: vec![2],
            }),
            guesses: Vec::new(),
        }),
        guess_seconds_remaining: Some(30),
        scores: Vec::new(),
        score_gains: Vec::new(),
        reference_changes: Vec::new(),
        minecraft_process_players: Vec::new(),
    }
}

#[test]
fn combo_candidates_follow_the_viewed_target_while_submission_waits_for_acknowledgement() {
    let mut game = snapshot();
    let mut state = MinecraftGuessUiState {
        input: "羊毛".to_owned(),
        ..Default::default()
    };
    state.graph.selected = Some(2);
    let before = CandidateQuery::new(&state, &game, &state.viewed_round(&game));
    assert!(before.active);
    assert!(!before.can_submit);
    assert_eq!(before.node, Some(2));

    game.graph.as_mut().unwrap().selected = 2;
    let after = CandidateQuery::new(&state, &game, &state.viewed_round(&game));
    assert!(after.active && after.can_submit);
    assert!(before != after);

    game.graph
        .as_mut()
        .unwrap()
        .combo
        .as_mut()
        .unwrap()
        .targets
        .clear();
    assert!(!CandidateQuery::new(&state, &game, &state.viewed_round(&game)).active);
    game.graph
        .as_mut()
        .unwrap()
        .combo
        .as_mut()
        .unwrap()
        .targets
        .push(2);
    game.round.current_player = Some(1);
    assert!(!CandidateQuery::new(&state, &game, &state.viewed_round(&game)).active);
    game.round.current_player = Some(0);
    game.rules.graph.options.candidate_hints = false;
    assert!(!CandidateQuery::new(&state, &game, &state.viewed_round(&game)).active);
}
