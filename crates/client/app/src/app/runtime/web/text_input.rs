use crate::app::shell::{ModalInputScope, UiState};
use bevy::input_focus::{FocusCause, InputFocus};
use bevy::prelude::*;
use bevy::text::{EditableText, TextReadWriteMode};
use bevy::ui::{ComputedStackIndex, InteractionDisabled};
use bevy::window::PrimaryWindow;
use leocard_ui::{PlatformEditor, TextInputAction, TextInputEvent};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "/src/app/runtime/web/text_input.js")]
extern "C" {
    #[wasm_bindgen(js_name = publishEditors)]
    fn publish_editors(specifications: &str);
    #[wasm_bindgen(js_name = takeEdits)]
    fn take_edits() -> String;
}

#[derive(Deserialize)]
struct BrowserEdit {
    entity: String,
    value: String,
    action: u8,
}

pub(super) fn apply_browser_edits(
    mut editors: Query<
        (
            Entity,
            &mut EditableText,
            &PlatformEditor,
            Option<&TextReadWriteMode>,
        ),
        Without<InteractionDisabled>,
    >,
    mut focus: ResMut<InputFocus>,
    mut actions: MessageWriter<TextInputEvent>,
    mut ui: ResMut<UiState>,
) {
    let edits: Vec<BrowserEdit> = serde_json::from_str(&take_edits()).unwrap_or_default();
    for edit in edits {
        let Some((entity, mut editor, spec, mode)) = editors
            .iter_mut()
            .find(|(entity, _, _, _)| entity.to_bits().to_string() == edit.entity)
        else {
            continue;
        };
        if edit.action == 4 {
            focus.set(entity, FocusCause::Pressed);
        } else {
            let value: String = edit
                .value
                .chars()
                .filter(|character| (spec.filter)(*character))
                .take(spec.max_characters)
                .collect();
            if mode.is_none_or(|mode| *mode == TextReadWriteMode::Editable) {
                editor.editor_mut().set_text(&value);
            }
            if matches!(edit.action, 1 | 2) {
                actions.write(TextInputEvent {
                    entity,
                    action: if edit.action == 1 {
                        TextInputAction::Submit
                    } else {
                        TextInputAction::Cancel
                    },
                });
            }
            if edit.action != 0 && focus.get() == Some(entity) {
                focus.clear();
            }
        }
        ui.dirty = true;
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct EditorSpec<'a> {
    entity: String,
    title: &'a str,
    value: Option<String>,
    rect: [f32; 4],
    font_size: f32,
    order: u32,
    focused: bool,
    numeric: bool,
    multiline: bool,
    password: bool,
    max_characters: u32,
    readonly: bool,
}

type BrowserEditorQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        Ref<'static, EditableText>,
        &'static PlatformEditor,
        &'static ComputedNode,
        &'static UiGlobalTransform,
        &'static TextFont,
        &'static InheritedVisibility,
        Option<&'static CalculatedClip>,
        &'static ComputedStackIndex,
        Option<&'static TextReadWriteMode>,
    ),
    Without<InteractionDisabled>,
>;

pub(super) fn publish_browser_editors(
    editors: BrowserEditorQuery,
    scope: ModalInputScope,
    windows: Query<&Window, With<PrimaryWindow>>,
    scale: Res<UiScale>,
    focus: Res<InputFocus>,
    mut published: Local<HashSet<Entity>>,
) {
    let Ok(window) = windows.single() else { return };
    let physical_to_css = 1.0 / window.scale_factor();
    let mut specifications = Vec::new();
    let mut visible = HashSet::new();
    for (entity, editor, options, node, transform, font, visibility, clip, order, mode) in &editors
    {
        if mode.is_some_and(|mode| *mode == TextReadWriteMode::Static)
            || !visibility.get()
            || !scope.allows(entity)
            || clip.is_some_and(CalculatedClip::is_fully_clipped)
        {
            continue;
        }
        let mut bounds = Rect::from_center_size(transform.translation, node.size());
        for clip in clip.and_then(CalculatedClip::rects).unwrap_or_default() {
            let to_world = clip.world_to_clip_local.inverse();
            let corners = [
                clip.rect.min,
                clip.rect.max,
                Vec2::new(clip.rect.min.x, clip.rect.max.y),
                Vec2::new(clip.rect.max.x, clip.rect.min.y),
            ]
            .map(|point| to_world.transform_point2(point));
            let minimum = corners
                .iter()
                .copied()
                .fold(Vec2::splat(f32::INFINITY), Vec2::min);
            let maximum = corners
                .iter()
                .copied()
                .fold(Vec2::splat(f32::NEG_INFINITY), Vec2::max);
            bounds = bounds.intersect(Rect::from_corners(minimum, maximum));
        }
        bounds = bounds.intersect(Rect::from_corners(
            Vec2::ZERO,
            window.physical_size().as_vec2(),
        ));
        if bounds.width() < 2.0 || bounds.height() < 2.0 {
            continue;
        }
        visible.insert(entity);
        specifications.push(EditorSpec {
            entity: entity.to_bits().to_string(),
            title: &options.title,
            value: (editor.is_changed() || !published.contains(&entity))
                .then(|| editor.value().to_string()),
            rect: [
                bounds.min.x * physical_to_css,
                bounds.min.y * physical_to_css,
                bounds.width() * physical_to_css,
                bounds.height() * physical_to_css,
            ],
            font_size: font.font_size.eval(window.size(), node.rem_size) * scale.0,
            order: order.0,
            focused: focus.get() == Some(entity),
            numeric: options.numeric,
            multiline: options.multiline,
            password: options.password,
            max_characters: options.max_characters.min(1_000_000) as u32,
            readonly: mode.is_some_and(|mode| *mode == TextReadWriteMode::ReadOnly),
        });
    }
    *published = visible;
    if let Ok(json) = serde_json::to_string(&specifications) {
        publish_editors(&json);
    }
}
