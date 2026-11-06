use super::super::chips::tests::pot_test_snapshot;
use super::super::{TexasRaiseAdjustButton, TexasUiAction, TexasUiState};
use super::input::{
    focus_texas_raise_input, handle_texas_raise_slider, sync_texas_raise_input,
    sync_texas_raise_value,
};
use super::sync::sync_texas_raise_controls;
use super::view::add_texas_actions;
use super::{
    TexasRaiseFill, TexasRaiseInput, TexasRaiseSlider, raise_fraction, raise_from_fraction,
};
use crate::app::runtime::UiAssets;
use crate::app::shell::{PressedUiAction, UiAction, UiState};
use bevy::asset::AssetPlugin;
use bevy::ecs::message::Messages;
use bevy::input_focus::{FocusCause, InputFocus};
use bevy::picking::hover::PickingInteraction;
use bevy::prelude::*;
use bevy::scene::ScenePatch;
use bevy::text::EditableText;
use bevy::ui::{BackgroundGradient, RelativeCursorPosition};
use leocard_protocol::TexasSnapshot;
use leocard_texas_holdem::TexasRaiseSizing;
use leocard_ui::{TextInputAction, TextInputEvent, UiComposer, UiSceneProperties, UiSceneRegistry};

#[test]
fn slider_covers_legal_raise_steps_and_fixed_limit() {
    let sizing = TexasRaiseSizing::new(10, 6, 63);
    assert_eq!(raise_from_fraction(sizing, -0.5), 16);
    assert_eq!(raise_from_fraction(sizing, 1.5), 58);
    let mut previous = 0;
    for i in 0..=100 {
        let target = raise_from_fraction(sizing, i as f32 / 100.0);
        assert!(sizing.contains(target));
        assert!(target >= previous);
        previous = target;
    }
    assert_eq!(raise_fraction(sizing, 16), 0.0);
    assert_eq!(raise_fraction(sizing, 58), 1.0);
    let fixed = TexasRaiseSizing::new(10, 6, 16);
    assert_eq!(raise_from_fraction(fixed, 0.8), 16);
    assert_eq!(raise_fraction(fixed, 16), 0.0);
}

#[test]
fn raise_input_autofocuses_once_preserves_edits_and_submits_on_enter() {
    let mut app = App::new();
    app.init_resource::<InputFocus>()
        .init_resource::<TexasUiState>()
        .init_resource::<UiState>()
        .add_message::<TextInputEvent>()
        .add_message::<PressedUiAction>()
        .add_systems(
            Update,
            (focus_texas_raise_input, sync_texas_raise_input).chain(),
        );
    let sizing = TexasRaiseSizing::new(10, 6, 63);
    let editor = app
        .world_mut()
        .spawn((TexasRaiseInput(sizing), EditableText::new("16")))
        .id();
    app.update();
    assert_eq!(app.world().resource::<InputFocus>().get(), Some(editor));
    // A composition refresh updates bounds, retaining both the editor and raw typed value.
    app.world_mut()
        .entity_mut(editor)
        .insert((TexasRaiseInput(sizing), EditableText::new("29")));
    app.update();
    assert_eq!(
        app.world().get::<EditableText>(editor).unwrap().value(),
        "29"
    );
    assert_eq!(app.world().resource::<TexasUiState>().raise_to, 34);
    assert!(!app.world().resource::<UiState>().dirty);
    for _ in 0..2 {
        app.world_mut().write_message(TextInputEvent {
            entity: editor,
            action: TextInputAction::Submit,
        });
    }
    app.update();
    let actions: Vec<_> = app
        .world_mut()
        .resource_mut::<Messages<PressedUiAction>>()
        .drain()
        .collect();
    assert_eq!(actions.len(), 1);
    assert!(matches!(
        actions[0].0,
        UiAction::Texas(TexasUiAction::Raise)
    ));
    assert_eq!(app.world().resource::<InputFocus>().get(), None);

    // Unrelated UI refreshes must not grab focus again, nor may invalid amounts submit.
    let other = app.world_mut().spawn_empty().id();
    app.world_mut()
        .resource_mut::<InputFocus>()
        .set(other, FocusCause::Pressed);
    app.update();
    assert_eq!(app.world().resource::<InputFocus>().get(), Some(other));
    app.world_mut()
        .entity_mut(editor)
        .insert(EditableText::new(""));
    app.world_mut()
        .resource_mut::<InputFocus>()
        .set(editor, FocusCause::Pressed);
    app.world_mut().write_message(TextInputEvent {
        entity: editor,
        action: TextInputAction::Submit,
    });
    app.update();
    assert!(!app.world().resource::<TexasUiState>().raise_input_valid);
    assert_eq!(
        app.world_mut()
            .resource_mut::<Messages<PressedUiAction>>()
            .drain()
            .count(),
        0
    );
}

#[test]
fn dragging_updates_the_fill_and_adjustments_without_rebuilding_the_table() {
    let mut app = App::new();
    app.init_resource::<TexasUiState>()
        .init_resource::<UiState>()
        .init_resource::<InputFocus>()
        .init_resource::<ButtonInput<MouseButton>>()
        .add_systems(
            Update,
            (
                handle_texas_raise_slider,
                sync_texas_raise_controls,
                sync_texas_raise_value,
            )
                .chain(),
        );
    let sizing = TexasRaiseSizing::new(10, 6, 63);
    let editor = app
        .world_mut()
        .spawn((TexasRaiseInput(sizing), EditableText::new("16")))
        .id();
    let slider = app
        .world_mut()
        .spawn((
            TexasRaiseSlider(sizing),
            PickingInteraction::Pressed,
            RelativeCursorPosition {
                normalized: Some(Vec2::ZERO),
                ..default()
            },
        ))
        .id();
    let fill = app
        .world_mut()
        .spawn((
            TexasRaiseFill { sizing, target: 16 },
            Node::default(),
            BackgroundGradient::default(),
        ))
        .id();
    let plus = app
        .world_mut()
        .spawn((
            TexasRaiseAdjustButton {
                direction: 1,
                step: 6,
                minimum: 16,
                maximum: 58,
            },
            UiAction::Texas(TexasUiAction::SetRaiseTo(22)),
        ))
        .id();
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Left);
    app.update();
    assert_eq!(app.world().resource::<TexasUiState>().raise_to, 40);
    assert_eq!(
        app.world().get::<EditableText>(editor).unwrap().value(),
        "40"
    );
    assert!(matches!(
        app.world().get::<UiAction>(plus),
        Some(UiAction::Texas(TexasUiAction::SetRaiseTo(46)))
    ));
    assert!(!app.world().resource::<UiState>().dirty);
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .clear();
    app.world_mut()
        .get_mut::<RelativeCursorPosition>(slider)
        .unwrap()
        .normalized = Some(Vec2::new(0.5, 0.0));
    app.update();
    assert_eq!(app.world().resource::<TexasUiState>().raise_to, 58);
    assert_eq!(
        app.world().get::<EditableText>(editor).unwrap().value(),
        "58"
    );
    assert_eq!(app.world().get::<Node>(fill).unwrap().width, percent(100));
    assert!(!app.world().resource::<UiState>().dirty);
}

#[test]
fn raise_editor_lifecycle_and_transition_to_runout_voting() {
    #[derive(Resource)]
    struct Table {
        root: Entity,
        game: TexasSnapshot,
    }
    fn compose(
        mut commands: UiComposer,
        table: Res<Table>,
        assets: Res<UiAssets>,
        mut ui: ResMut<TexasUiState>,
    ) {
        commands.begin(table.root);
        add_texas_actions(
            &mut commands,
            table.root,
            &table.game,
            &table.game.players[0],
            &mut ui,
            &assets,
        );
        commands.finish();
    }
    fn labels(world: &mut World) -> Vec<String> {
        world
            .query::<&Text>()
            .iter(world)
            .map(|text| text.0.clone())
            .collect()
    }
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()))
        .init_resource::<Assets<ScenePatch>>()
        .init_resource::<UiSceneRegistry>()
        .init_resource::<UiSceneProperties>()
        .init_resource::<UiAssets>()
        .init_resource::<TexasUiState>()
        .add_systems(Update, compose);
    let root = app.world_mut().spawn(Node::default()).id();
    let game = pot_test_snapshot(&[(0, false, false), (2, false, false)]);
    app.insert_resource(Table { root, game });
    app.update();
    assert!(labels(app.world_mut()).contains(&"加注".to_owned()));
    assert_eq!(
        app.world_mut()
            .query::<&TexasRaiseInput>()
            .iter(app.world())
            .count(),
        0
    );
    app.world_mut()
        .resource_mut::<TexasUiState>()
        .raise_editor_open = true;
    app.update();
    let texts = labels(app.world_mut());
    assert!(texts.contains(&"确认".to_owned()) && texts.contains(&"取消".to_owned()));
    assert!(
        !texts
            .iter()
            .any(|text| text == "弃牌" || text == "加注" || text.starts_with("本次投入"))
    );
    assert_eq!(
        app.world_mut()
            .query::<&TexasRaiseInput>()
            .iter(app.world())
            .count(),
        1
    );
    let game = app.world().resource::<Table>().game.clone();
    {
        let mut ui = app.world_mut().resource_mut::<TexasUiState>();
        ui.raise_pending = Some((leocard_protocol::Revision(1), 0));
        ui.reconcile(&game, leocard_protocol::Revision(1), 0);
        assert!(ui.raise_pending.is_some());
    }
    app.update();
    assert!(labels(app.world_mut()).is_empty());
    // A server rejection restores the editor; an acknowledgement closes it.
    {
        let mut ui = app.world_mut().resource_mut::<TexasUiState>();
        ui.reconcile(&game, leocard_protocol::Revision(1), 1);
        assert!(ui.raise_pending.is_none() && ui.raise_editor_open);
        ui.raise_pending = Some((leocard_protocol::Revision(1), 1));
        ui.reconcile(&game, leocard_protocol::Revision(2), 1);
        assert!(ui.raise_pending.is_none() && !ui.raise_editor_open);
    }
    app.world_mut()
        .resource_mut::<TexasUiState>()
        .raise_editor_open = false;
    app.update();
    assert!(labels(app.world_mut()).contains(&"加注".to_owned()));
    assert_eq!(
        app.world_mut()
            .query::<&TexasRaiseInput>()
            .iter(app.world())
            .count(),
        0
    );

    // An all-in voter must see only runout choices, even with stale editor state.
    {
        let mut table = app.world_mut().resource_mut::<Table>();
        table.game.phase = leocard_protocol::TexasPhaseView::ChoosingRunout;
        table.game.players[0].all_in = true;
        table.game.players[0].stack = 0;
        table.game.raise_allowed = false;
        table.game.maximum_raise_to = None;
        table.game.all_in_allowed = false;
    }
    {
        let mut ui = app.world_mut().resource_mut::<TexasUiState>();
        ui.raise_editor_open = true;
        ui.raise_pending = Some((leocard_protocol::Revision(2), 1));
    }
    app.update();
    let mut texts = labels(app.world_mut());
    texts.sort();
    assert_eq!(texts, ["发一次", "发两次"]);
    let mut votes = app
        .world_mut()
        .query::<&UiAction>()
        .iter(app.world())
        .filter_map(|action| match action {
            UiAction::Texas(TexasUiAction::ChooseRunout(twice)) => Some(*twice),
            _ => None,
        })
        .collect::<Vec<_>>();
    votes.sort();
    assert_eq!(votes, [false, true]);
    assert_eq!(
        app.world_mut()
            .query::<&TexasRaiseInput>()
            .iter(app.world())
            .count(),
        0
    );
    assert!(super::raise_sizing(&app.world().resource::<Table>().game).is_none());
    let game = app.world().resource::<Table>().game.clone();
    {
        let mut ui = app.world_mut().resource_mut::<TexasUiState>();
        ui.reconcile(&game, leocard_protocol::Revision(2), 1);
        assert!(!ui.raise_editor_open && ui.raise_pending.is_none());
    }
    for excluded in 0..3 {
        {
            let mut table = app.world_mut().resource_mut::<Table>();
            table.game.players[0].auto_play = excluded == 0;
            table.game.players[0].folded = excluded == 1;
            table.game.current_player = (excluded != 2).then_some(table.game.you);
        }
        app.update();
        assert!(labels(app.world_mut()).is_empty());
    }
}
