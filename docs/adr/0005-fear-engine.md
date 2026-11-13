# 0005 — The fear engine core (M5)

Status: accepted · Date: 2026-11-12

## Context

SRD-001 §4.2 (spec M5) asks for a procedural horror layer that is a
**dial, not a story**: dread, routine tasks, and small deniable wrong
details, all seed-derived. The sandbox must stay neutral at dial 0.
Light play (M3) is the stated dependency for the darkness coupling, but
M3 is not built in this checkout; the shell that would supply live light
state is also outside this repo. This ADR records the core that can be
built and proved now, with the light input left as data.

## Decision

### Domain (`src/domain/haunt.rs`)

Pure data plus one classification:

- `DreadRung` — `Calm < Unease < Wrong < Event`; thresholds 0–24,
  25–49, 50–74, 75–100. Ordered by severity so "at or below the dial's
  cap" is a comparison, not a table lookup at every call site.
- `Task { id, kind, target, done }`, `TaskKind` (`CheckLights`,
  `RestorePower`, `Catalog`, `CloseDoors`), `TaskTarget` (`Room` |
  `Furniture`).
- `Event { kind, target }`, `EventKind` (`DoorAjar`, `ChairMoved`,
  `LampOn`, `LightFlicker`).
- `DreadState { level, dial, rung, event_log }`.

### Use cases (split by responsibility, §4.4)

- `use_cases/haunt.rs` — the dread coupling: `Systems` (darkness,
  dark minutes, tasks done, light relief), `new_dread`, `allowed_rung`,
  `may_spend_event`, `step`, `complete_task`, and the shared `pick`
  helper.
- `use_cases/haunt_tasks.rs` — `generate_tasks` and the house
  inspection that finds real targets (lamp rooms, hall/entry, every
  piece, rooms adjacent to a door).
- `use_cases/haunt_events.rs` — `wrong_details` and the house
  inspection that finds real doors, chairs, lamps, and rooms.

The split was forced by the 200-line ceiling: the first draft was 204
lines. The seam is responsibility (coupling vs. content generation),
not line range; tests moved with their functions and behaviour did not
change.

### The dial

Three bands, so a low dial can never buy a high-rung detail:

| Dial | Highest rung it will spend |
|---|---|
| 0 | Calm (nothing; dread frozen at 0) |
| 1–33 | Unease |
| 34–66 | Wrong |
| 67–100 | Event |

`step` spends a candidate only when `may_spend_event` is true. At dial 0
it returns before the gain is computed, so the sandbox cannot drift.

### The coupling, in one place

```
gain = darkness·10 + dark_minutes·0.5 − light_relief·8
       − tasks_done·6 + spent_events·3
level = clamp(level + gain, 0, 100)
rung  = DreadRung::from_level(level)
```

The constants are v1 tuning, not measured facts. `complete_task` also
grounds by six points and refuses an unknown or already-done task.

### Content

- Tasks name real rooms or pieces; a kind with no target is skipped.
  This is what makes a task checkable rather than scripted (AC-5).
- Wrong details are seed-derived and their count rises with the dial
  band (0 / 1 / 2 / 3). Every target is a real door, chair, lamp, or
  room. The event log is the verifiability surface (AC-6).

## Alternatives considered

- **A `Haunt` struct holding house + state + RNG.** Rejected: it hides
  dependencies and makes the one-line-test criterion (§3.3) harder.
  Pure functions over `DreadState` keep every rule callable with
  literals.
- **Deriving the rung in the use case only.** Rejected: the thresholds
  are domain classification, and `DreadRung` ordering is what makes
  the dial cap a one-line comparison.
- **Coupling light directly to `LightSource` now.** Rejected: M3 is
  not built, and the core would depend on a boundary that does not
  exist. `Systems` is the seam; M3 replaces its values, not its shape.
- **A global event budget.** Rejected for v1: the dial band plus the
  rung cap is enough to satisfy AC-1 and AC-6, and a budget would need
  a session concept the core does not own yet.

## Consequences

- The M5 core is pure, deterministic, and testable with literal data.
  AC-1 is a property test over 10 seeds × 100 actions; AC-5 and AC-6
  are covered by target-existence and reproducibility tests.
- The shell must supply `Systems` and `wrong_details` candidates; it
  never computes dread. That keeps the shell/core split in SRD §6.3.
- M3 light play will replace the supplied darkness/relief values with
  real light state. No interface change is expected: `Systems` is
  already plain data.
- No sound, no jumpscares (NFR-4), no fail state. The event log is
  append-only; there is no "undo a scare".

## Revisit if

M3 lands and the shell needs per-light coupling; or the dial bands
prove too coarse in play; or the event budget needs to be per session
rather than per step.
