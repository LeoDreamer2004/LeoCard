use super::*;
use leocard_protocol::{
    MatchId, PlayerGameProfiles, PlayerId, ProfileId, SeatId, UnoEvent, UnoPendingSwapView,
    UnoPhaseView, UnoPlayerState, UnoSnapshot,
};
use leocard_uno::{Mode, UnoCard, UnoColor, UnoDirection, UnoFace, UnoRuleSet, build_flip_deck};
use std::collections::{HashMap, HashSet};
use std::path::Path;

#[test]
fn uno_interaction_and_presentation_systems_initialize_without_conflicts() {
    let mut app = App::new();
    app.insert_resource(Time::<()>::default());
    app.insert_resource(UiState::default());
    app.insert_resource(UiAssets::default());
    app.insert_resource(UnoPresentationState::default());
    app.insert_resource(UnoAudioState::default());
    app.insert_resource(Assets::<UnoPaletteMaterial>::default());
    app.add_systems(
        Update,
        (
            animate_uno_hand_cards,
            spawn_uno_presentation_effects,
            animate_uno_flying_cards,
            sync_uno_discard_reveal,
            animate_uno_palette_effects,
            animate_uno_palette_selected_sectors,
            animate_uno_palette_color_rings,
            animate_uno_palette_particles,
            animate_uno_reverse_effects,
        )
            .chain(),
    );

    app.update();
}

#[test]
fn uno_presentation_waits_until_rebuilt_anchors_are_laid_out() {
    let node = ComputedNode::default();
    let transform = UiGlobalTransform::default();
    assert!(uno_anchor_in_layer(&node, &transform, &node, &transform).is_none());
}

#[test]
fn played_uno_card_settles_at_the_discard_cards_exact_scale() {
    assert_eq!(uno_flying_card_scale(false, 1.0), 1.0);
    assert!(uno_flying_card_scale(false, 0.5) < 1.0);
}

#[test]
fn jump_in_selection_allows_single_card_or_identical_pair() {
    let first = UnoCard::number(UnoColor::Red, 7, 0);
    let second = UnoCard::number(UnoColor::Red, 7, 1);
    let mut game = UnoSnapshot {
        match_id: MatchId([9; 16]),
        host_port: 52300,
        you: PlayerId(0),
        host: PlayerId(0),
        rules: UnoRuleSet {
            action_stacking: true,
            jump_in: true,
            ..UnoRuleSet::default()
        },
        players: vec![UnoPlayerState {
            id: PlayerId(0),
            profile_id: ProfileId([0; 32]),
            name: "玩家".to_owned(),
            avatar: None,
            seat: SeatId(0),
            hand_len: 3,
            inactive_hand: Vec::new(),
            ready: false,
            connected: true,
            auto_play: false,
            reference_points: 0,
            completed_games: 0,
            game_profiles: PlayerGameProfiles::default(),
            skipped_turns: 0,
            eliminated: false,
        }],
        your_hand: vec![first, second, UnoCard::number(UnoColor::Blue, 3, 0)],
        draw_pile_len: 80,
        draw_pile_inactive_cards: Vec::new(),
        discard_top: UnoCard::number(UnoColor::Red, 4, 0),
        discard_pile: vec![UnoCard::number(UnoColor::Red, 4, 0)],
        current_color: Some(UnoColor::Red),
        flip_side: None,
        current_player: Some(PlayerId(0)),
        direction: UnoDirection::Clockwise,
        pending_draw: 0,
        pending_kind: None,
        challenge_offender: None,
        pending_skip: 0,
        pending_swap: None,
        your_drawn_card: None,
        your_jump_in_card: None,
        uno_exposed: Vec::new(),
        uno_declared: Vec::new(),
        can_call_uno: false,
        phase: UnoPhaseView::Playing,
    };

    let other = UnoCard::number(UnoColor::Blue, 3, 0);
    let mut selected = HashSet::new();
    toggle_uno_selection(Some(&game), &mut selected, first);
    assert_eq!(selected, HashSet::from([first]));
    toggle_uno_selection(Some(&game), &mut selected, second);
    assert_eq!(selected, HashSet::from([first, second]));
    toggle_uno_selection(Some(&game), &mut selected, first);
    assert_eq!(selected, HashSet::from([second]));
    toggle_uno_selection(Some(&game), &mut selected, other);
    assert_eq!(selected, HashSet::from([other]));

    game.uno_declared.push(game.you);
    assert_eq!(uno_pair_for_selection(&game, first), None);
}

#[test]
fn swap_pack_target_selection_replaces_one_target_but_requires_deselecting_a_full_pair() {
    let you = PlayerId(0);
    let first = PlayerId(1);
    let second = PlayerId(2);
    let third = PlayerId(3);
    let mut selected = vec![first];

    toggle_uno_swap_target_selection(
        Some(UnoPendingSwapView::SwapOneTarget { player: you }),
        you,
        second,
        &mut selected,
    );
    assert_eq!(selected, vec![second]);
    toggle_uno_swap_target_selection(
        Some(UnoPendingSwapView::SwapOneTarget { player: you }),
        you,
        second,
        &mut selected,
    );
    assert!(selected.is_empty());

    selected = vec![first, second];
    toggle_uno_swap_target_selection(
        Some(UnoPendingSwapView::ForceTrade { player: you }),
        you,
        third,
        &mut selected,
    );
    assert_eq!(selected, vec![first, second]);
    toggle_uno_swap_target_selection(
        Some(UnoPendingSwapView::ForceTrade { player: you }),
        you,
        first,
        &mut selected,
    );
    toggle_uno_swap_target_selection(
        Some(UnoPendingSwapView::ForceTrade { player: you }),
        you,
        third,
        &mut selected,
    );
    assert_eq!(selected, vec![second, third]);
}

#[test]
fn every_extension_card_has_hand_hover_help() {
    assert!(uno_extension_card_help(UnoFace::SwapOne).is_some());
    assert!(uno_extension_card_help(UnoFace::RefreshHand).is_some());
    assert!(uno_extension_card_help(UnoFace::WildForceTrade).is_some());
    assert!(uno_extension_card_help(UnoFace::WildPassHands).is_some());
    assert!(uno_extension_card_help(UnoFace::ReverseDrawTwo).is_some());
    assert!(uno_extension_card_help(UnoFace::ReverseSkip).is_some());
    assert!(uno_extension_card_help(UnoFace::WildPowerReverse).is_some());
    assert!(uno_extension_card_help(UnoFace::WildNoU).is_some());
    assert!(uno_extension_card_help(UnoFace::StackOne).is_some());
    assert!(uno_extension_card_help(UnoFace::StackTwo).is_some());
    assert!(uno_extension_card_help(UnoFace::WildStackThree).is_some());
    assert!(uno_extension_card_help(UnoFace::WildStackNumber).is_some());
    assert!(uno_extension_card_help(UnoFace::DrawFour).is_some());
    assert!(uno_extension_card_help(UnoFace::DrawFive).is_some());
    assert!(uno_extension_card_help(UnoFace::SkipEveryone).is_some());
    assert!(uno_extension_card_help(UnoFace::Flip).is_some());
    assert!(uno_extension_card_help(UnoFace::WildDrawColor).is_some());
    assert!(uno_extension_card_help(UnoFace::DiscardAll).is_some());
    assert!(uno_extension_card_help(UnoFace::WildReverseDrawFour).is_some());
    assert!(uno_extension_card_help(UnoFace::WildDrawSix).is_some());
    assert!(uno_extension_card_help(UnoFace::WildDrawTen).is_some());
    assert!(uno_extension_card_help(UnoFace::WildColorRoulette).is_some());
    assert!(uno_extension_card_help(UnoFace::Number(7)).is_none());
    assert!(uno_extension_card_help(UnoFace::WildDrawFour).is_none());
}

#[test]
fn every_stack_pack_face_resolves_to_a_valid_card_texture() {
    let mut cards = Vec::new();
    for color in UnoColor::ALL {
        cards.push(UnoCard::action(color, UnoFace::StackOne, 0));
        cards.push(UnoCard::action(color, UnoFace::StackTwo, 0));
    }
    cards.push(UnoCard::wild(UnoFace::WildStackThree, 0));
    cards.push(UnoCard::wild(UnoFace::WildStackNumber, 0));

    let assets = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets");
    for card in cards {
        let path = assets.join(uno_card_asset_path(card));
        let image = image::open(&path)
            .unwrap_or_else(|error| panic!("{} 无法解码：{error}", path.display()));
        assert_eq!(image.width(), 256);
        assert_eq!(image.height(), 400);
    }
}

#[test]
fn every_flip_face_resolves_to_a_valid_card_texture() {
    let assets = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets");
    for card in build_flip_deck() {
        for face in [Some(card.public_face()), card.opposite_public_face()] {
            let face = face.expect("FLIP cards have two faces");
            let path = assets.join(uno_card_asset_path(face));
            let image = image::open(&path)
                .unwrap_or_else(|error| panic!("{} 无法解码：{error}", path.display()));
            assert_eq!(image.width(), 256);
            assert_eq!(image.height(), 400);
        }
    }
}

#[test]
fn noninteractive_jump_in_card_uses_the_normal_selected_lift() {
    let card = UnoCard::number(UnoColor::Red, 7, 1);
    let mut app = App::new();
    let mut time = Time::<()>::default();
    time.advance_by(std::time::Duration::from_millis(100));
    app.insert_resource(time);
    let mut ui = UiState::default();
    ui.uno.selected.insert(card);
    app.insert_resource(ui);
    app.add_systems(Update, animate_uno_hand_cards);
    let noninteractive_slot = app.world_mut().spawn_empty().id();
    let face = app
        .world_mut()
        .spawn((
            UnoHandCardVisual {
                button: noninteractive_slot,
                card,
                selected: true,
                hover_amount: 0.0,
                selected_amount: 0.0,
            },
            UiTransform::IDENTITY,
            Outline::default(),
            BoxShadow::new(Color::BLACK, px(0), px(0), px(0), px(0)),
            BorderColor::all(Color::NONE),
        ))
        .id();

    app.update();

    let visual = app.world().get::<UnoHandCardVisual>(face).unwrap();
    let transform = app.world().get::<UiTransform>(face).unwrap();
    assert!(visual.selected_amount > 0.0);
    assert!(matches!(transform.translation.y, Val::Px(y) if y < 0.0));
}

#[test]
fn authoritative_uno_discard_waits_for_its_flying_card_to_land() {
    let card = UnoCard::number(UnoColor::Red, 7, 0);
    let other = UnoCard::number(UnoColor::Blue, 7, 0);
    let mut presentation = UnoPresentationState::default();
    presentation.events.push_back(UnoEvent::CardPlayed {
        player: PlayerId(0),
        card,
        chosen_color: None,
        play_index: 0,
        play_count: 1,
    });

    assert!(uno_discard_should_be_hidden(
        card,
        &presentation,
        std::iter::empty()
    ));
    assert!(!uno_discard_should_be_hidden(
        other,
        &presentation,
        std::iter::empty()
    ));

    presentation.events.clear();
    assert!(uno_discard_should_be_hidden(
        card,
        &presentation,
        [Some(card)].into_iter()
    ));
    assert!(!uno_discard_should_be_hidden(
        card,
        &presentation,
        std::iter::empty()
    ));
}

#[test]
fn retained_uno_discard_cards_keep_their_pose_when_the_six_card_window_slides() {
    let cards = [
        UnoCard::number(UnoColor::Red, 1, 0),
        UnoCard::number(UnoColor::Yellow, 2, 0),
        UnoCard::number(UnoColor::Green, 3, 0),
        UnoCard::number(UnoColor::Blue, 4, 0),
        UnoCard::action(UnoColor::Red, UnoFace::Reverse, 0),
        UnoCard::action(UnoColor::Blue, UnoFace::Skip, 1),
        UnoCard::wild(UnoFace::Wild, 0),
    ];
    let before = cards[..6]
        .iter()
        .copied()
        .map(|card| (card, uno_discard_pose(card)))
        .collect::<HashMap<_, _>>();
    let after = cards[1..]
        .iter()
        .copied()
        .map(|card| (card, uno_discard_pose(card)))
        .collect::<HashMap<_, _>>();

    for card in &cards[1..6] {
        assert_eq!(before.get(card), after.get(card));
    }
}

#[test]
fn selected_uno_palette_sector_starts_at_base_size_then_grows() {
    assert_eq!(uno_palette_selected_scale(0.0), 1.0);
    assert_eq!(uno_palette_selected_scale(0.18), 1.0);
    assert!(uno_palette_selected_scale(0.70) >= 1.31);
    assert!(uno_palette_selected_scale(1.0) > 1.2);
}
#[test]
fn uno_expansion_settings_only_frames_the_hosts_status_control() {
    fn setup(mut commands: Commands, assets: Res<UiAssets>) {
        let host_root = commands.spawn(Node::default()).id();
        render_uno_expansion_settings(
            &mut commands,
            host_root,
            UnoRuleSet {
                swap_pack: true,
                reverse_pack: true,
                stack_pack: true,
                ..default()
            },
            true,
            &assets,
        );
        let guest_root = commands.spawn(Node::default()).id();
        render_uno_expansion_settings(
            &mut commands,
            guest_root,
            UnoRuleSet::default(),
            false,
            &assets,
        );
        let no_mercy_root = commands.spawn(Node::default()).id();
        render_uno_expansion_settings(
            &mut commands,
            no_mercy_root,
            UnoRuleSet {
                mode: Mode::NoMercy,
                swap_pack: true,
                reverse_pack: true,
                stack_pack: true,
                ..UnoRuleSet::default()
            },
            true,
            &assets,
        );
    }

    let mut app = App::new();
    app.init_resource::<UiAssets>();
    app.add_systems(Startup, setup);
    app.update();

    let mut statuses = app.world_mut().query::<(
        &UnoExpansionStatus,
        Option<&Button>,
        Option<&UnoExpansionStatusFrame>,
        Option<&UiAction>,
    )>();
    let statuses = statuses
        .iter(app.world())
        .map(|(_, button, frame, action)| (button.is_some(), frame.is_some(), action.cloned()))
        .collect::<Vec<_>>();
    let host = statuses
        .iter()
        .filter(|(button, frame, _)| *button && *frame)
        .collect::<Vec<_>>();
    assert_eq!(host.len(), 3);
    assert!(host.iter().any(|(_, _, action)| matches!(
        action,
        Some(UiAction::UpdateUnoRules(rules)) if !rules.swap_pack && rules.reverse_pack
    )));
    assert!(host.iter().any(|(_, _, action)| matches!(
        action,
        Some(UiAction::UpdateUnoRules(rules)) if rules.swap_pack && !rules.reverse_pack
    )));
    assert!(host.iter().any(|(_, _, action)| matches!(
        action,
        Some(UiAction::UpdateUnoRules(rules)) if rules.swap_pack && rules.reverse_pack && !rules.stack_pack
    )));
    let guests = statuses
        .iter()
        .filter(|(button, frame, action)| !*button && !*frame && action.is_none())
        .count();
    assert_eq!(guests, 3);

    let labels = app
        .world_mut()
        .query::<&Text>()
        .iter(app.world())
        .map(|text| text.0.as_str())
        .collect::<Vec<_>>();
    for label in [
        "扩展包设置",
        "Swap Pack",
        "Reverse Pack",
        "Stack Pack",
        "✓",
        "×",
    ] {
        assert!(labels.contains(&label));
    }
    assert!(
        labels
            .iter()
            .any(|label| label.contains("以交换手牌为特色"))
    );
    assert!(
        labels
            .iter()
            .any(|label| label.contains("以改变方向为特色"))
    );
    assert!(labels.contains(&"当前尚未加入 No Mercy 扩展包。"));
}

#[test]
fn uno_mode_switch_uses_a_translucent_three_option_dropdown() {
    fn setup(mut commands: Commands, assets: Res<UiAssets>) {
        let root = commands
            .spawn(Node {
                width: px(1280),
                height: px(720),
                ..default()
            })
            .id();
        let parent = commands.spawn(Node::default()).id();
        commands.entity(root).add_child(parent);
        render_uno_mode_dropdown(
            &mut commands,
            root,
            parent,
            UnoRuleSet {
                action_stacking: true,
                swap_pack: true,
                ..default()
            },
            true,
            true,
            &assets,
        );
    }

    let mut app = App::new();
    app.init_resource::<UiAssets>();
    app.add_systems(Startup, setup);
    app.update();

    let backgrounds = app
        .world_mut()
        .query_filtered::<&BackgroundColor, With<UnoModeDropdownPanel>>()
        .iter(app.world())
        .collect::<Vec<_>>();
    assert_eq!(backgrounds.len(), 1);
    assert!(backgrounds[0].0.to_srgba().alpha < 1.0);

    let mode_buttons = app
        .world_mut()
        .query_filtered::<(Option<&ImageNode>, Option<&BackgroundColor>, &UiAction), With<Button>>()
        .iter(app.world())
        .filter(|(_, _, action)| {
            matches!(
                action,
                UiAction::ToggleUnoModeMenu | UiAction::UpdateUnoRules(_)
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(mode_buttons.len(), 4);
    assert!(
        mode_buttons
            .iter()
            .all(|(image, background, _)| image.is_none() && background.is_some())
    );

    let rules = app
        .world_mut()
        .query_filtered::<&UiAction, With<Button>>()
        .iter(app.world())
        .filter_map(|action| match action {
            UiAction::UpdateUnoRules(rules) => Some(*rules),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(rules.len(), 3);
    assert_eq!(
        rules.iter().map(|rules| rules.mode).collect::<Vec<_>>(),
        vec![Mode::Classic, Mode::NoMercy, Mode::Flip,]
    );
    assert!(
        rules
            .iter()
            .all(|rules| rules.action_stacking && rules.swap_pack)
    );
}

#[test]
fn image_less_card_buttons_keep_their_transparent_background() {
    let mut app = App::new();
    app.add_systems(Update, update_button_tints);
    let card = app
        .world_mut()
        .spawn((
            Button,
            Interaction::Hovered,
            BackgroundColor(Color::NONE),
            ButtonTint {
                normal: Color::WHITE,
                hovered: Color::srgb(1.0, 0.92, 0.66),
                pressed: Color::srgb(0.78, 0.84, 0.72),
            },
        ))
        .id();

    app.update();

    assert_eq!(
        app.world().get::<BackgroundColor>(card).unwrap().0,
        Color::NONE
    );
}

#[test]
fn uno_expansion_settings_entry_centers_its_label_in_the_full_width_button() {
    fn setup(mut commands: Commands, assets: Res<UiAssets>) {
        let root = commands.spawn(Node::default()).id();
        let button = add_action_button(
            &mut commands,
            root,
            "扩展包设置",
            UiAction::ToggleUnoExpansionSettings,
            ButtonKind::Secondary,
            &assets,
        );
        commands.entity(button).insert(Node {
            width: percent(100),
            min_width: px(0),
            height: px(56),
            padding: UiRect::axes(px(18), px(8)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        });
    }

    let mut app = App::new();
    app.init_resource::<UiAssets>();
    app.add_systems(Startup, setup);
    app.update();

    let mut buttons = app
        .world_mut()
        .query_filtered::<(&Node, &UiAction), With<Button>>();
    let (node, _) = buttons
        .iter(app.world())
        .find(|(_, action)| matches!(action, UiAction::ToggleUnoExpansionSettings))
        .expect("扩展包设置入口应是可点击按钮");
    assert_eq!(node.width, percent(100));
    assert_eq!(node.height, px(56));
    assert_eq!(node.align_items, AlignItems::Center);
    assert_eq!(node.justify_content, JustifyContent::Center);
}
