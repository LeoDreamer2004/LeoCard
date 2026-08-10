# Client architecture

The executable entry point is intentionally tiny. `src/main.rs` delegates to the
Bevy application in `src/app`, while the transport/state model remains in
`src/lib.rs` so it can be tested without rendering.

## Application modules

| Module | Responsibility |
| --- | --- |
| `app/mod.rs` | Shared Bevy resources/components and update ordering |
| `bootstrap.rs` | Preferences, local profile, avatar normalization, asset loading |
| `input.rs` | Keyboard/clipboard input, scaling, card selection, file pickers |
| `controller.rs` | UI commands, network events, hints, errors, summary state |
| `interaction.rs` | Chat bubbles, social items, cooldowns, score-capture animation |
| `play_effects.rs` | Runtime systems for card-play effects and deal sounds |
| `screens.rs` | Connection, settings, lobby, and top-level screen composition |
| `table.rs` | Table layout, seats, cards, score areas, and turn clocks |
| `chat_view.rs` | Chat drawer, quick voices, and developer hand input |
| `summary_view.rs` | End-of-game summary presentation |
| `effect_view.rs` | Effect entity construction, errors, reconnect overlay |
| `widgets.rs` | Reusable buttons, panels, cards, avatars, and labels |

The application modules expose functions only to their parent with
`pub(super)`. Protocol and authoritative rules must not be added here: shared
messages belong in `leocard-protocol`, rules in `leocard-qigui523`, and host-side
state transitions in `leocard-host`.

## Change routing

- A network or reconnect change starts in `src/lib.rs`, then is reflected by
  `app/controller.rs`.
- A visual-only feature should stay in the relevant view/effect module and must
  not mutate authoritative game state.
- Reusable UI construction belongs in `widgets.rs`; screen-specific layout
  remains beside its screen.
- New resources/components belong in `app/mod.rs` only when more than one
  feature module owns their lifecycle.

The chained Bevy update order in `app/mod.rs` is deliberate: input is applied
first, model/network state is synchronized next, and transient presentation is
updated before a dirty UI tree is rebuilt.
