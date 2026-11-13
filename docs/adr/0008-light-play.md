# ADR 0008: Light play and the dread coupling

## Status

Accepted. Implements SRD-001 §4.4 (FR-22 – FR-25) and the M3 exit
criterion AC-8. Fills the `Systems` seam ADR 0005 left open without
changing its shape.

## Context

The fear engine takes darkness, dark minutes, tasks done, and light
relief as data; nothing produced them. The sandbox needs lights the
player can find and switch, a day/night cycle with genuinely dark
nights, and a rule that no room is ever unwinnably dark. The core is
stateless across WASM calls, so a session's light list lives in the
shell.

## Decision

### Domain (`domain/light.rs`)

- `LightKind`: ceiling, lamp, porch, flashlight, window.
- `LightState`: on, off, flicker, broken. `toggle()` flips on/off,
  switches a flicker off, and never repairs broken.
- `LightSource`: id, kind, optional room, position, radius, state.
- `Daylight`: minutes since midnight (0–1440), `held`, and pure
  `elevation()` = sin((minutes − 360)/1440 · τ). Six o'clock is the
  horizon; `phase()` names night/dawn/day/dusk. `advance()` is a
  no-op while held, which is the sandbox hold (FR-24).

### Derivation and switches (`use_cases/light.rs`)

`lights_for_house` runs after furnishing and derives one source per
concept: a ceiling per room, a lamp per lamp piece, a window per room
that has one, a porch light when the footprint has a porch, and the
carried flashlight. Ids occupy disjoint ranges (1, 2, 1000+, 2000+,
3000+) so every light is addressable.

`Interaction::ToggleLight { light_id }` joins the existing command
model. `light::toggle` refuses a missing or broken light.
`sync_lamp` mirrors a lamp light's state onto its furniture, so the
mesh and the lighting cannot disagree. The older `ToggleLamp`
command is unchanged.

### Dread coupling (`use_cases/light_dread.rs`)

The lit fraction of a room is the strongest of its own lights and
the daylight its windows admit; the carried flashlight contributes
0.8. Darkness and light relief are complements, so `Systems` sees
one number:

```
darkness   = 1 − level
light_relief = level
```

A flickering light is offered to the engine as an
`EventKind::LightFlicker` candidate — a dread event, not a render
bug (FR-25). Flicker is deterministic in the tick the shell passes
in, so a haunting stays reproducible (NFR-2).

### WASM protocol v2

Four exports: `fp_house_lights(seed, style)`, 
`fp_house_lights_toggled(seed, style, light_id)`,
`fp_daylight(minutes, held)`, and `fp_light_toggle_state(code)`.
The adapter stays stateless: the shell holds the light list, the
core owns the state machine. A refused switch returns a null pointer
and length 0, like every other invalid call.

## Alternatives

- **Lights only where furniture is.** Rejected: rooms with no lamp
  would have no switch, breaking FR-23.
- **A global WASM session with mutable state.** Rejected: no threads
  in wasm32, the adapter is deliberately stateless, and ADR 0005
  established the shell-supplies-systems pattern.
- **Random flicker.** Rejected: NFR-2 requires every event to be
  seed-reproducible.

## Consequences

- Every room has a working ceiling light and the flashlight is on,
  so a dark room at night is navigable by flashlight alone (AC-8,
  tested over seeds and styles).
- The shell renders light positions from JSON and applies switches
  through the command model; it never decides how dark a room is.
- The harness grew from 529 to 560 checks, all passing.

## Revisit if

The shell needs light-radius falloff, shadows, or networked light
state; none of that changes the `Systems` coupling.
