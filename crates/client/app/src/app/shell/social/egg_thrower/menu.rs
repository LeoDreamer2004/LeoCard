use super::super::buttons::interaction_button;
use super::icon::add_egg_thrower_button;
use crate::app::runtime::UiAssets;
use crate::app::shell::{InteractionMenuPanel, PressedUiAction, SocialUiAction, UiAction};
use bevy::picking::events::{PointerClick, PointerPress};
use bevy::picking::hover::PickingInteraction;
use bevy::picking::pointer::PointerButton;
use bevy::prelude::*;
use bevy::ui::OverrideClip;
use leocard_game_common::PlayerId;
use leocard_protocol::PlayerInteractionKind;
use leocard_ui::UiComposer;
use std::time::Duration;

const HOLD_DURATION: Duration = Duration::from_millis(550);

#[derive(Component, Clone)]
pub(crate) struct ShoeButton(PlayerId);

#[derive(Component)]
pub(crate) struct ShoeHold {
    popup: Entity,
    started_at: Option<Duration>,
    expanded: bool,
}

pub(crate) fn add_shoe_button(
    commands: &mut UiComposer,
    parent: Entity,
    target: PlayerId,
    assets: &UiAssets,
) -> Entity {
    let button = interaction_button(
        assets,
        ShoeButton(target),
        Some("拖鞋"),
        Some(assets.social.interaction_images[&(PlayerInteractionKind::Shoe, false)].clone()),
    )
    .mount(commands, parent, &assets.font)
    .0;
    let popup = add_egg_thrower_button(commands, button, target, assets);
    commands.entity(popup).insert((
        Visibility::Hidden,
        InteractionMenuPanel(target),
        OverrideClip,
        GlobalZIndex(1510),
    ));
    commands.entity(button).init(ShoeHold {
        popup,
        started_at: None,
        expanded: false,
    });
    button
}

pub(crate) fn begin_shoe_hold(
    mut press: On<PointerPress>,
    time: Res<Time<Real>>,
    mut buttons: Query<&mut ShoeHold>,
) {
    if press.entity == press.original_event_target()
        && press.button == PointerButton::Primary
        && let Ok(mut hold) = buttons.get_mut(press.entity)
    {
        press.propagate(false);
        hold.started_at = Some(time.elapsed());
        hold.expanded = false;
    }
}

pub(crate) fn click_shoe_button(
    mut click: On<PointerClick>,
    mut buttons: Query<(&ShoeButton, &mut ShoeHold)>,
    mut actions: MessageWriter<PressedUiAction>,
) {
    if click.entity != click.original_event_target() || click.button != PointerButton::Primary {
        return;
    }
    let Ok((button, mut hold)) = buttons.get_mut(click.entity) else {
        return;
    };
    click.propagate(false);
    if click.duration >= HOLD_DURATION {
        hold.expanded = true;
    } else if !hold.expanded {
        actions.write(PressedUiAction(UiAction::Social(
            SocialUiAction::SendInteraction {
                target: button.0,
                kind: PlayerInteractionKind::Shoe,
            },
        )));
    }
}

pub(crate) fn update_shoe_holds(
    time: Res<Time<Real>>,
    mut buttons: Query<(&PickingInteraction, &InheritedVisibility, &mut ShoeHold)>,
    mut popups: Query<&mut Visibility>,
) {
    for (interaction, visible, mut hold) in &mut buttons {
        if !visible.get() {
            hold.started_at = None;
            hold.expanded = false;
        } else if *interaction == PickingInteraction::Pressed {
            if hold
                .started_at
                .is_some_and(|start| time.elapsed().saturating_sub(start) >= HOLD_DURATION)
            {
                hold.expanded = true;
            }
        } else {
            hold.started_at = None;
        }
        if let Ok(mut popup) = popups.get_mut(hold.popup) {
            popup.set_if_neq(if hold.expanded {
                Visibility::Visible
            } else {
                Visibility::Hidden
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::camera::NormalizedRenderTarget;
    use bevy::picking::backend::HitData;
    use bevy::picking::events::Pointer;
    use bevy::picking::pointer::{Location, PointerId};

    fn pointer() -> Pointer {
        Pointer::new(
            PointerId::Mouse,
            Location {
                target: NormalizedRenderTarget::None {
                    width: 1280,
                    height: 720,
                },
                position: Vec2::ZERO,
            },
        )
    }

    #[test]
    fn holding_opens_the_consumable_without_throwing_a_shoe_but_a_short_click_throws_once() {
        let mut app = App::new();
        app.init_resource::<Time<Real>>()
            .add_message::<PressedUiAction>()
            .add_observer(begin_shoe_hold)
            .add_observer(click_shoe_button)
            .add_systems(Update, update_shoe_holds);
        let popup = app.world_mut().spawn(Visibility::Hidden).id();
        let button = app
            .world_mut()
            .spawn((
                ShoeButton(PlayerId(1)),
                ShoeHold {
                    popup,
                    started_at: None,
                    expanded: false,
                },
                PickingInteraction::Pressed,
                InheritedVisibility::VISIBLE,
            ))
            .id();
        app.world_mut().entity_mut(button).add_child(popup);
        app.world_mut().trigger(PointerPress {
            entity: button,
            pointer: pointer(),
            button: PointerButton::Primary,
            hit: HitData::new(Entity::PLACEHOLDER, 0.0, None, None),
            count: 1,
        });
        app.world_mut()
            .resource_mut::<Time<Real>>()
            .advance_by(HOLD_DURATION);
        app.update();
        assert_eq!(
            app.world().get::<Visibility>(popup),
            Some(&Visibility::Visible)
        );
        app.world_mut().trigger(PointerClick {
            entity: button,
            pointer: pointer(),
            button: PointerButton::Primary,
            hit: HitData::new(Entity::PLACEHOLDER, 0.0, None, None),
            duration: HOLD_DURATION,
            count: 1,
        });
        assert!(
            app.world()
                .resource::<Messages<PressedUiAction>>()
                .is_empty()
        );

        app.world_mut()
            .entity_mut(button)
            .insert(InheritedVisibility::HIDDEN);
        app.update();
        assert_eq!(
            app.world().get::<Visibility>(popup),
            Some(&Visibility::Hidden)
        );
        app.world_mut()
            .entity_mut(button)
            .insert(InheritedVisibility::VISIBLE);
        app.world_mut().trigger(PointerPress {
            entity: button,
            pointer: pointer(),
            button: PointerButton::Primary,
            hit: HitData::new(Entity::PLACEHOLDER, 0.0, None, None),
            count: 1,
        });
        app.world_mut().trigger(PointerClick {
            entity: button,
            pointer: pointer(),
            button: PointerButton::Primary,
            hit: HitData::new(Entity::PLACEHOLDER, 0.0, None, None),
            duration: Duration::from_millis(100),
            count: 1,
        });
        let actions: Vec<_> = app
            .world_mut()
            .resource_mut::<Messages<PressedUiAction>>()
            .drain()
            .collect();
        assert_eq!(actions.len(), 1);
        assert!(matches!(
            actions[0].0,
            UiAction::Social(SocialUiAction::SendInteraction {
                target: PlayerId(1),
                kind: PlayerInteractionKind::Shoe,
            })
        ));
        app.world_mut().trigger(PointerClick {
            entity: popup,
            pointer: pointer(),
            button: PointerButton::Primary,
            hit: HitData::new(Entity::PLACEHOLDER, 0.0, None, None),
            duration: Duration::from_millis(100),
            count: 1,
        });
        assert!(
            app.world()
                .resource::<Messages<PressedUiAction>>()
                .is_empty()
        );
    }
}
