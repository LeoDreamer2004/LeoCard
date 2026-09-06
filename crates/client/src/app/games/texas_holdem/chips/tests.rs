use super::ledger::{change_for, initial_chip_denominations, visual_pots};
use super::view::{
    add_chip_zone_panel, add_fold_card_feedback, pot_divider_visual, pot_eligibility_breath,
};
use super::*;
use leocard_protocol::ProfileId;

fn pot_test_snapshot(committed: &[(u32, bool, bool)]) -> TexasHoldemSnapshot {
    let players = committed
        .iter()
        .enumerate()
        .map(
            |(index, &(amount, all_in, folded))| TexasHoldemPlayerState {
                id: PlayerId(index as u8),
                profile_id: ProfileId([index as u8; 32]),
                name: format!("P{index}"),
                avatar: None,
                seat: SeatId(index as u8),
                stack: if all_in {
                    0
                } else {
                    20_u32.saturating_sub(amount)
                },
                hand_start_stack: 20,
                committed_street: amount,
                committed_total: amount,
                folded,
                all_in,
                connected: true,
                auto_play: false,
                ready: false,
                reference_points: 0,
                completed_games: 0,
                game_profiles: PlayerGameProfiles::default(),
            },
        )
        .collect::<Vec<_>>();
    TexasHoldemSnapshot {
        match_id: MatchId([1; 16]),
        hand_number: 1,
        host_port: 5230,
        you: PlayerId(0),
        host: PlayerId(0),
        players,
        your_hole_cards: Vec::new(),
        revealed_hands: Vec::new(),
        community: Vec::new(),
        draw_pile_len: 52,
        dealer: PlayerId(0),
        small_blind: PlayerId(1),
        big_blind: PlayerId(2),
        current_player: Some(PlayerId(0)),
        blind_to_post: None,
        current_bet: committed.iter().map(|entry| entry.0).max().unwrap_or(0),
        minimum_raise_to: 1,
        amount_to_call: 0,
        raise_allowed: true,
        pot: committed.iter().map(|entry| entry.0).sum(),
        phase: TexasHoldemPhaseView::Betting {
            street: TexasHoldemStreet::PreFlop,
        },
    }
}

#[test]
fn initial_distributions_keep_five_single_chips_and_exact_value() {
    let expected = [
        (5, (0, 0, 5)),
        (10, (0, 1, 5)),
        (20, (0, 3, 5)),
        (30, (1, 3, 5)),
        (40, (2, 3, 5)),
        (50, (3, 3, 5)),
    ];
    for (total, (tens, fives, ones)) in expected {
        let chips = initial_chip_denominations(total);
        assert_eq!(
            chips.iter().map(|chip| u32::from(*chip)).sum::<u32>(),
            total
        );
        assert_eq!(chips.iter().filter(|chip| **chip == 10).count(), tens);
        assert_eq!(chips.iter().filter(|chip| **chip == 5).count(), fives);
        assert_eq!(chips.iter().filter(|chip| **chip == 1).count(), ones);
    }
}

#[test]
fn every_supported_chip_can_be_changed_exactly() {
    for denomination in [5, 10, 25, 100] {
        let change = change_for(denomination);
        assert_eq!(
            change.iter().map(|chip| u32::from(*chip)).sum::<u32>(),
            u32::from(denomination)
        );
        assert!(change.iter().all(|chip| *chip < denomination));
    }
}

#[test]
fn side_pots_only_split_at_all_in_caps_and_keep_correct_eligibility() {
    let no_all_in = visual_pots(&pot_test_snapshot(&[
        (5, false, false),
        (10, false, false),
        (20, false, false),
    ]));
    assert_eq!(no_all_in.len(), 1);
    assert_eq!(no_all_in[0].amount, 35);

    let split = visual_pots(&pot_test_snapshot(&[
        (5, true, false),
        (10, true, false),
        (20, false, false),
    ]));
    assert_eq!(
        split.iter().map(|pot| pot.amount).collect::<Vec<_>>(),
        vec![15, 10, 10]
    );
    assert_eq!(
        split[0].eligible,
        vec![PlayerId(0), PlayerId(1), PlayerId(2)]
    );
    assert_eq!(split[1].eligible, vec![PlayerId(1), PlayerId(2)]);
    assert_eq!(split[2].eligible, vec![PlayerId(2)]);
}

#[test]
fn pot_dividers_remove_old_layout_before_drawing_new_layout() {
    assert_eq!(pot_divider_visual(true, 0.0), (1.0, 1.0));
    assert_eq!(pot_divider_visual(true, 0.18).0, 0.0);
    assert_eq!(pot_divider_visual(false, 0.16), (0.0, 0.0));
    let appeared = pot_divider_visual(false, 0.48);
    assert!((appeared.0 - 1.0).abs() < f32::EPSILON);
    assert!((appeared.1 - 1.0).abs() < f32::EPSILON);
}

#[test]
fn eligible_player_glow_uses_a_smooth_breathing_cycle() {
    assert!(pot_eligibility_breath(0.0).abs() < 0.000_001);
    assert!((pot_eligibility_breath(0.85) - 1.0).abs() < 0.000_001);
    assert!(pot_eligibility_breath(1.7).abs() < 0.000_001);
}

#[test]
fn chip_zone_panel_uses_procedural_glass_layers_without_a_felt_image() {
    fn setup(mut commands: Commands) {
        let table = commands.spawn(Node::default()).id();
        add_chip_zone_panel(
            &mut commands,
            table,
            ChipZoneLayout {
                left: 10.0,
                top: 20.0,
                width: 160.0,
                height: 82.0,
            },
        );
    }

    let mut app = App::new();
    app.add_systems(Startup, setup);
    app.update();

    let (node, background, background_gradient, border_gradient, shadow, image) = app
        .world_mut()
        .query_filtered::<(
            &Node,
            &BackgroundColor,
            &BackgroundGradient,
            &BorderGradient,
            &BoxShadow,
            Option<&ImageNode>,
        ), With<TexasChipZonePanel>>()
        .single(app.world())
        .unwrap();
    assert_eq!(node.border, UiRect::all(px(1)));
    assert_eq!(background.0, TEXAS_CHIP_ZONE_FILTER);
    assert_eq!(background_gradient.0.len(), 1);
    assert_eq!(border_gradient.0.len(), 1);
    assert_eq!(shadow.0.len(), 1);
    assert!(image.is_none());
}

#[test]
fn centre_panel_keeps_a_gap_above_the_own_chip_zone() {
    let centre = texas_center_zone_panel();
    let own = texas_player_chip_zone(0);
    assert_eq!(own.top - (centre.top + centre.height), 12.0);
}

#[test]
fn authoritative_action_audio_is_not_requeued_by_an_empty_ui_refresh() {
    let snapshot = pot_test_snapshot(&[(0, false, false), (0, false, false), (0, false, false)]);
    let mut state = TexasChipTableState::default();
    state.initialize(&snapshot);
    state.observe(
        &snapshot,
        vec![TexasHoldemEvent::ActionApplied {
            player: PlayerId(1),
            action: TexasHoldemAction::Check,
            amount: 0,
        }],
    );
    assert_eq!(state.audio_cues.len(), 1);
    assert_eq!(state.audio_cues[0].kind, TexasSoundKind::Check);

    state.audio_cues.clear();
    state.observe(&snapshot, Vec::new());
    assert!(state.audio_cues.is_empty());
}

#[test]
fn action_feedback_uses_distinct_motion_for_check_raise_and_all_in() {
    let settled_check = action_feedback_visual(ActionFeedbackKind::Check, 1.0);
    assert_eq!(settled_check.translation, Vec2::ZERO);
    assert_eq!(settled_check.alpha, 1.0);
    assert!(action_feedback_visual(ActionFeedbackKind::Raise, 0.14).scale > 1.0);
    let all_in = action_feedback_visual(ActionFeedbackKind::AllIn, 0.17);
    assert!(all_in.scale > 1.0);
    assert_ne!(all_in.translation.x, 0.0);
    assert_eq!(
        action_feedback_visual(ActionFeedbackKind::Fold, 2.0).alpha,
        1.0
    );

    let own_start = fold_card_visual(0, 2, 0.0, true);
    let own_flipped = fold_card_visual(0, 2, 0.30, true);
    let own_settled = fold_card_visual(0, 2, 0.72, true);
    let own_rebuilt = fold_card_visual(0, 2, 8.0, true);
    assert!(own_start.face_visible);
    assert!(!own_flipped.face_visible);
    assert!(own_start.transform.scale.y > own_settled.transform.scale.y);
    assert_eq!(own_settled.transform.scale, own_rebuilt.transform.scale);
    assert_eq!(
        own_settled.transform.translation,
        own_rebuilt.transform.translation
    );
}

#[test]
fn omaha_opponent_fold_renders_four_hidden_cards() {
    fn setup(mut commands: Commands, assets: Res<UiAssets>) {
        let table = commands.spawn(Node::default()).id();
        let zone = commands.spawn(Node::default()).id();
        commands.entity(table).add_child(zone);
        add_fold_card_feedback(&mut commands, table, zone, 0.0, None, 4, &assets);
    }

    let mut app = App::new();
    app.init_resource::<UiAssets>();
    app.add_systems(Startup, setup);
    app.update();

    let cards = app
        .world_mut()
        .query::<&TexasFoldCard>()
        .iter(app.world())
        .map(|card| (card.index, card.total, card.own))
        .collect::<Vec<_>>();
    assert_eq!(cards.len(), 4);
    assert!(cards.iter().all(|(_, total, own)| *total == 4 && !own));

    let positions = (0..4)
        .map(|index| {
            fold_card_visual(index, 4, 0.58, false)
                .transform
                .translation
                .x
        })
        .collect::<Vec<_>>();
    assert!(positions.windows(2).all(|pair| pair[0] != pair[1]));
}
