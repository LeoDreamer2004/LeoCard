use super::{
    retention::{TextInputRetention, restore_editors},
    state::{RetainedEditors, TextInputKey, TextInputSlot},
};
use bevy::{
    ecs::system::RunSystemOnce,
    input_focus::InputFocus,
    prelude::*,
    text::{EditableText, FontCx, LayoutCx, TextEdit, apply_text_edits},
};
use bevy_clipboard::Clipboard;

#[derive(Component)]
struct Page;

fn rebuild(
    mut commands: Commands,
    mut retention: TextInputRetention,
    pages: Query<Entity, With<Page>>,
) {
    retention.capture(&mut commands);
    for page in &pages {
        commands.entity(page).despawn();
    }
    let editor = commands
        .spawn((TextInputKey("name"), EditableText::new("新页面初始值")))
        .id();
    let slot = commands
        .spawn(TextInputSlot {
            key: TextInputKey("name"),
            editor,
        })
        .add_child(editor)
        .id();
    commands.spawn(Page).add_child(slot);
}

#[test]
fn page_rebuild_keeps_the_actual_editor_focus_and_composition() {
    let mut app = App::new();
    app.init_resource::<RetainedEditors>()
        .init_resource::<FontCx>()
        .init_resource::<LayoutCx>()
        .insert_resource(Clipboard::default());
    let mut input = EditableText::new("玩家");
    input.queue_edit(TextEdit::ImeSetCompose {
        value: "ni".into(),
        cursor: None,
    });
    let editor = app.world_mut().spawn((TextInputKey("name"), input)).id();
    app.world_mut().spawn(Page).add_child(editor);
    app.insert_resource(InputFocus::from_entity(editor));
    app.world_mut().run_system_once(apply_text_edits).unwrap();
    app.world_mut().run_system_once(rebuild).unwrap();
    app.world_mut().run_system_once(restore_editors).unwrap();

    let input = app.world().get::<EditableText>(editor).unwrap();
    assert!(input.is_composing());
    assert_eq!(input.value().to_string(), "玩家");
    assert_eq!(app.world().resource::<InputFocus>().get(), Some(editor));
    let mut slots = app.world_mut().query::<&TextInputSlot>();
    assert_eq!(slots.single(app.world()).unwrap().editor, editor);

    // 离开含输入框的页面时，释放编辑实体和焦点。
    app.world_mut()
        .run_system_once(
            |mut commands: Commands,
             mut retention: TextInputRetention,
             pages: Query<Entity, With<Page>>| {
                retention.capture(&mut commands);
                for page in &pages {
                    commands.entity(page).despawn();
                }
            },
        )
        .unwrap();
    app.world_mut().run_system_once(restore_editors).unwrap();
    assert!(app.world().get_entity(editor).is_err());
    assert_eq!(app.world().resource::<InputFocus>().get(), None);
}
