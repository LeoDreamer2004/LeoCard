use super::super::libraries::{LibrarySelection, WordLibraryManager};
use crate::app::shell::{ConfigurationScroll, scroll_modal_content};
use bevy::camera::RenderTarget;
use bevy::ecs::system::RunSystemOnce;
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::input::touch::TouchPhase;
use bevy::picking::hover::PickingInteraction;
use bevy::picking::pointer::{Location, PointerId, PointerLocation};
use bevy::prelude::*;
use bevy::text::{EditableText, TextEdit};
use bevy::ui::RelativeCursorPosition;
use leocard_client::LocalWordLibrary;
use leocard_draw_guess::WordLibrary;

pub(super) fn manager() -> WordLibraryManager {
    let mut library = WordLibrary::from_lines("自然与风景", "向日葵\n彩虹\n月亮\n星空\n大海\n瀑布\n雪山\n森林\n沙漠\n樱花\n竹林\n闪电\n雨伞\n雪花\n蘑菇\n仙人掌\n小溪\n冰山\n火山\n萤火虫").unwrap();
    library
        .words
        .extend((1..=60).map(|index| format!("景物{index}")));
    let mut manager = WordLibraryManager {
        open: true,
        progress: 1.0,
        ..default()
    };
    manager.store.entries.push(LocalWordLibrary {
        id: "preview.json".into(),
        library,
    });
    manager.select(LibrarySelection::Local("preview.json".into()));
    manager
}

pub(super) fn assert_multiline_editor(app: &mut App, target: &RenderTarget) {
    let (entity, position) = app
        .world_mut()
        .query::<(Entity, &EditableText, &ComputedNode, &UiGlobalTransform)>()
        .iter(app.world())
        .find(|(_, editor, _, _)| editor.allow_newlines)
        .map(|(entity, editor, node, transform)| {
            assert!(editor.value().to_string().contains("向日葵\n彩虹"));
            assert!(node.size().y > 100.0);
            (entity, transform.transform_point2(Vec2::ZERO))
        })
        .unwrap();
    app.world_mut()
        .get_mut::<EditableText>(entity)
        .unwrap()
        .queue_edit(TextEdit::ScrollBy(Vec2::new(0.0, -100_000.0)));
    let pointer = app
        .world_mut()
        .spawn((
            PointerId::Touch(u64::MAX - 1),
            PointerLocation::new(Location {
                target: target.normalize(None).unwrap(),
                position,
            }),
        ))
        .id();
    for _ in 0..3 {
        app.update();
    }
    assert_ne!(
        *app.world().get::<PickingInteraction>(entity).unwrap(),
        PickingInteraction::None
    );
    wheel(app, -3.0);
    for _ in 0..3 {
        app.update();
    }
    let offset = app
        .world()
        .get::<EditableText>(entity)
        .unwrap()
        .viewport
        .offset
        .y;
    assert!(offset > 0.0, "wheel should scroll the multiline input");
    wheel(app, 3.0);
    for _ in 0..3 {
        app.update();
    }
    assert!(
        app.world()
            .get::<EditableText>(entity)
            .unwrap()
            .viewport
            .offset
            .y
            < offset
    );
    app.world_mut().despawn(pointer);
}

pub(super) fn assert_builtin_scrolls(app: &mut App) {
    let entity = app
        .world_mut()
        .query_filtered::<(Entity, &ComputedNode), With<ConfigurationScroll>>()
        .iter(app.world())
        .find(|(_, node)| node.content_size().y > node.size().y)
        .map(|(entity, _)| entity)
        .unwrap();
    app.world_mut()
        .entity_mut(entity)
        .insert(RelativeCursorPosition {
            cursor_over: true,
            normalized: Some(Vec2::ZERO),
        });
    wheel(app, -3.0);
    app.world_mut()
        .run_system_once(scroll_modal_content::<ConfigurationScroll>)
        .unwrap();
    assert!(
        app.world().get::<ScrollPosition>(entity).unwrap().y > 0.0,
        "default library should scroll"
    );
    app.world_mut().get_mut::<ScrollPosition>(entity).unwrap().y = 0.0;
}

fn wheel(app: &mut App, y: f32) {
    app.world_mut().write_message(MouseWheel {
        unit: MouseScrollUnit::Line,
        phase: TouchPhase::Moved,
        x: 0.0,
        y,
        window: Entity::PLACEHOLDER,
    });
}
