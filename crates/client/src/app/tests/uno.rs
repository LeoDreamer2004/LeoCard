use super::prelude::*;
use leocard_protocol::{PlayerId, UnoPendingSwapView};
use leocard_uno::{Mode, UnoCard, UnoColor, UnoFace, UnoRuleSet, build_flip_deck};
use std::path::Path;

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
    let mut ui = UnoUiState::default();
    ui.selected.insert(card);
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
        Some(UiAction::Uno(UnoUiAction::UpdateRules(rules))) if !rules.swap_pack && rules.reverse_pack
    )));
    assert!(host.iter().any(|(_, _, action)| matches!(
        action,
        Some(UiAction::Uno(UnoUiAction::UpdateRules(rules))) if rules.swap_pack && !rules.reverse_pack
    )));
    assert!(host.iter().any(|(_, _, action)| matches!(
        action,
        Some(UiAction::Uno(UnoUiAction::UpdateRules(rules))) if rules.swap_pack && rules.reverse_pack && !rules.stack_pack
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
