use super::{CandidateQuery, McGuessUiState};
use leocard_game_common::PlayerId;
use leocard_mc_guess::{
    McGuessCatalogue, McGuessEntryKind, McGuessGraphCombo, McGuessGraphNodeView, McGuessGraphView,
    McGuessMatchState, McGuessMode, McGuessRoundView, McGuessRuleSet,
};
use leocard_protocol::{
    MatchId, McGuessPlayerView, McGuessSnapshot, PlayerGameProfiles, PlayerPublicInfo, ProfileId,
    SeatId,
};

fn snapshot() -> McGuessSnapshot {
    let rules = McGuessRuleSet {
        mode: McGuessMode::Graph,
        ..Default::default()
    };
    McGuessSnapshot {
        match_id: MatchId([1; 16]),
        host_port: 52300,
        you: PlayerId(0),
        host: PlayerId(0),
        players: vec![McGuessPlayerView {
            player: PlayerPublicInfo {
                id: PlayerId(0),
                profile_id: ProfileId([1; 32]),
                name: "玩家".to_owned(),
                avatar: None,
                reference_points: 0,
                completed_games: 0,
                game_profiles: PlayerGameProfiles::default(),
            },
            seat: SeatId(0),
            ready: false,
            connected: true,
        }],
        rules,
        progress: McGuessMatchState::new(vec![PlayerId(0), PlayerId(1)], rules).progress(),
        round: McGuessRoundView {
            completed: false,
            kind: McGuessEntryKind::Block,
            character_count: 0,
            current_player: Some(PlayerId(0)),
            guesses: Vec::new(),
            skipped_players: Vec::new(),
            answer: None,
            winner: None,
        },
        graph: Some(McGuessGraphView {
            nodes: vec![
                McGuessGraphNodeView {
                    number: 1,
                    kind: Some(McGuessEntryKind::Block),
                    character_count: None,
                    entry: McGuessCatalogue::bundled()
                        .find(McGuessEntryKind::Block, "minecraft:blue_wool")
                        .cloned(),
                    tag_name: None,
                },
                McGuessGraphNodeView {
                    number: 2,
                    kind: Some(McGuessEntryKind::Block),
                    character_count: Some(4),
                    entry: None,
                    tag_name: None,
                },
            ],
            edges: Vec::new(),
            selected: 1,
            combo: Some(McGuessGraphCombo {
                count: 1,
                targets: vec![2],
            }),
            guesses: Vec::new(),
        }),
        guess_seconds_remaining: Some(30),
        scores: Vec::new(),
        score_gains: Vec::new(),
        reference_changes: Vec::new(),
        mc_process_players: Vec::new(),
    }
}

#[test]
fn combo_candidates_follow_the_viewed_target_while_submission_waits_for_acknowledgement() {
    let mut game = snapshot();
    let mut state = McGuessUiState {
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
    game.round.current_player = Some(PlayerId(1));
    assert!(!CandidateQuery::new(&state, &game, &state.viewed_round(&game)).active);
    game.round.current_player = Some(PlayerId(0));
    game.rules.graph.options.candidate_hints = false;
    assert!(!CandidateQuery::new(&state, &game, &state.viewed_round(&game)).active);
}
