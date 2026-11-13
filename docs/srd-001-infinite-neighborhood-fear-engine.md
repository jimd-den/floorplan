# SRD-001: Infinite Neighborhood Fear Engine

**Status:** draft, written 2026-11-12.
**Project:** `floorplan` (`~/workspace/floorplan`) — Rust, edition 2024, std only, zero crates.
**Formalises:** the engine spec (`~/workspace/goals/floorplan-3d-houses-and-neighborhoods/files/spec-engine.md`)
§4.5 (streaming), §4.7 (horror), §4.3 (realism) — spec phases **M4, M5, M6**.
**Depends on:** spec §4.4 (M2, built) and §4.6 (M3, not yet built — see §1.3).
**Does not contradict the spec.** Where the spec leaves a decision open, this SRD
records it as open (§9) and does not decide it. Numbers marked **TARGET** are
numbers to hit, stated as such — not measured facts.

## 1. Purpose and scope

### 1.1 Purpose

The `floorplan` generator hosts a **procedural sandbox horror mechanic engine**
(spec §1). This SRD is the requirements document for the slice of that engine
that makes the neighborhood infinite, makes it frightening in the
Mortuary's Assistant mode, and makes its houses read as real houses:

- an **infinite neighborhood** that streams and generates as you move;
- a **fear engine** — dread and atmosphere, routine tasks in a mundane place,
  wrong details — dialed in, never forced;
- **realistic interiors and exteriors** — furnished, interactive, lit houses
  on streets that read as streets.

### 1.2 What it is

- A sandbox first. You walk a house and a neighborhood, open things, move
  things, turn lights on and off. Nothing forces a scare. No fail state by
  default.
- A horror engine second, as a **dial**, not a story. At intensity 0 there
  are zero horror events. As the dial rises, the engine spends its budget on
  dread: wrong details, light misbehavior, tasks that take you into the dark.
- Procedural all the way down. Every house, every piece of furniture, every
  event comes from a seed. Same seed, same house, same neighborhood, every
  time.
- Its own standalone thing. **Not voxels, not folded into vackrooms2.** It
  may borrow vackrooms2's proven streaming *pattern* — worker pool, request
  ledger, metered installs — but the code and the engine are floorplan's.

### 1.3 What it is not

- A jumpscare game. No loud stingers, no face in the dark by default. See
  NFR-4: no jumpscares, ever, without a separate decision and ADR.
- A fixed story or a scripted house. No authored level. The horror comes
  from systems (light, tasks, wrong details) acting on generated houses.
- **Not scary by default.** The sandbox is neutral; horror is a mechanic
  layer dialed in (spec §4.8).
- Not M0–M3. Re-tiling and the furniture model (M0) and furniture mesh and
  interaction (M1) are built and verified (§2). The WASM runtime (M2) is
  built and verified (§2). **Light play (M3) is not yet built** — the light
  requirements in §4.4 are stated here because the fear engine depends on
  them, but they are built in M3, not in this SRD's phases.

## 2. Baseline — what is already built and verified

This SRD builds on measured, verified work, not aspirations. Figures are
from the sources listed in the header.

- **Generator:** 6 styles (Ranch, Colonial, Craftsman, Modern, Cottage,
  Townhouse), rectangle/L/U/T footprints on a 0.1 m grid, BSP subdivision
  with a 2.4 m minimum side (no slivers), programs by rule, doors on a
  connectivity graph (every room reachable), windows on exterior walls only.
  1,000 seeds → 1,000 distinct plan hashes; honest lower bound
  ≈ 4.5 × 10⁹ distinct houses (`docs/reports/variety-count.md`).
- **Re-tiling (M0):** bath ≤ 6 m², study ≤ 12 m². Over 200 seeds (one style
  per seed): 0 baths and 0 studies over cap. **Residual, carried openly:**
  the strict reading (all six styles × 200 seeds) still has 5 studies at
  12.0–13.9 m² in thin rects (seeds 47 and 146 Ranch, 133, 145, 186
  Modern). Whether the strict 5 must also go to zero is Jim's call.
- **Furniture (M0+M1, ADR 0003):** 12 kinds, placed per room program
  against walls, never overlapping, never blocking a door swing or a
  window (tall pieces), every piece reachable. 5,790 pieces across 200
  houses, 0 empty. Placement rate ≈ 92% (baths with a tub 1,495/1,626,
  bedrooms with a bed 2,630/2,853); the ~8% are small rooms where the key
  piece cannot fit without blocking a door swing. The integration test
  gates the rules plus a ≥ 90% placement rate. Interaction is the command
  model: `open`/`close` door, `toggle` lamp, `move` chair/table — refused
  when invalid, state persisted, mesh follows state. Seed 42 Modern house:
  22 pieces, 11 kinds, mesh 1,400 verts / 2,100 faces, 29 materials.
- **Neighborhoods (ADR 0004):** 3 layout families — grid, offset,
  cul-de-sac — picked by `seed % 3`, each with its own street pattern.
  Streets 8 m; lots with setbacks (front 5 m, back 3 m, sides 2 m); one
  house per lot, each facing its street; lot size biases style. Invariants
  (inside lot with setbacks, no overlaps, one house per lot, every lot
  street-fronted, every house facing its street) hold over 100 seeds **per
  family**; each family gives 100 distinct layout hashes over 100 seeds.
  Seed 42, 2×2 blocks, Grid: 22 houses, 6 streets, 22 lots; neighborhood
  mesh 39,888 verts / 59,832 faces / 31 materials.
- **WASM adapter (ADR 0004, protocol v1):** std-only core compiled to
  `wasm32-unknown-unknown`, no wasm-bindgen, raw `extern "C"` exports,
  `FPM1` binary mesh format, caller-owned pointers via `fp_alloc` /
  `fp_free`, length via `fp_last_len()`. Node harness: **364 checks, 0
  failures** against the native CLI. Release `.wasm` 218,419 bytes
  (sha256 `aa9398b6…`, committed as base64 at `wasm/floorplan.wasm.b64`).
- **Web shell:** three.js page loading the release `.wasm`, first-person
  walk (1.7 m eye, pointer-lock + drag fallback, WASD/Shift/Esc), seed
  input, 2×2/3×3, roof toggle, per-house door-gap collision (884 wall
  segments, seed 42). Verified: seed 42 regenerates to Grid 22 houses;
  seed 44 to Cul-de-sac 20 houses.
- **Decided (Jim, 2026-11-12):** first-person perspective, three.js
  renderer, clean architecture with small composable functions.

## 3. Terms

- **Chunk.** One unit of generated neighborhood, derived only from the
  world seed and its coordinates. The exact granularity — one city block
  with its lots and houses, or one street segment — is a design decision
  inside M4, not fixed here (spec §4.5). A chunk has a lifecycle:
  requested → generating → ready → installed → evicted.
- **Neighborhood.** The unbounded set of chunks around the player. The
  fixed 2×2 neighborhood already built becomes chunk (0,0).
- **Layout family.** Grid, offset, or cul-de-sac street/block pattern,
  picked by `seed % 3` (ADR 0004). A chunk's family derives from the
  world seed and the chunk's coordinates, so the same chunk always has
  the same family.
- **Dread state.** A number 0–100 per house, with named rungs —
  calm / unease / wrong / event — driven by systems (§4.2). The engine
  only spends events at the rung the horror dial allows.
- **Wrong detail.** A small, deniable, seed-derived event: a chair moved
  since you last saw it, a door ajar that you closed, a lamp on in a room
  you left dark, a light that flickers only when you are not looking at
  it. Each is written to an event log.
- **Task.** A routine, checkable, grounding job generated from a real
  house — it names real rooms and real objects. Kinds: check_lights,
  restore_power, catalog, close_doors (spec §5).
- **Horror dial.** A first-class setting, 0–100, saved with the session.
  0 = neutral sandbox: zero horror events, dread frozen at 0. Higher
  settings raise the event budget, the wrong-detail rate, and the
  darkness coupling.
- **Light source.** A domain object: ceiling fixture, lamp, porch light,
  flashlight (player-carried), or window daylight. Each has a position, a
  radius, a state (on / off / flicker / broken), and a switch or control
  that owns it.

## 4. Functional requirements

Every requirement is testable. A requirement without a test is a wish;
the test for each is named in §7.

### 4.1 Streaming — the infinite neighborhood (spec M4)

- **FR-1.** The neighborhood is generated in chunks. A chunk's content
  derives only from the world seed and its coordinates — never from
  visit order, wall-clock time, or player state.
- **FR-2.** Chunks append asynchronously. Generation runs off the main
  path (worker pool); player movement never blocks waiting for a chunk.
- **FR-3.** Chunk installs are metered: a small fixed number per tick
  (**TARGET:** at most 2 per tick), so a chunk arriving never hitches
  the frame.
- **FR-4.** Generation is just-in-time-plus-one: the ring ahead of the
  player is always being built. The chunk the player is about to enter
  is ready and installed before they reach it — no pop-in, no stall on
  a chunk boundary.
- **FR-5.** Chunks behind the player are evicted (their mesh and JSON
  released). Leaving a chunk and coming back regenerates byte-identical
  content (FR-1 + NFR-2).
- **FR-6.** The neighborhood keeps generating distinct houses and
  layouts at distance. Chunk content hashes are distinct across the
  chunks visited in a session — the player never walks into a repeated
  neighborhood. (Revisits are the exception, and are byte-identical by
  FR-5.)
- **FR-7.** The existing neighborhood invariants hold **per chunk**, over
  seeds: every lot street-fronted, one house per lot, no overlaps, every
  house inside its lot with setbacks, every house enterable.
- **FR-8.** The fixed 2×2 neighborhood becomes chunk (0,0). Existing
  output is unchanged: seed 42 still gives Grid, 22 houses, 6 streets,
  22 lots.
- **FR-9.** The WASM boundary exposes chunk generation by (world seed,
  chunk coordinates), returning chunk JSON and chunk mesh. Protocol v1
  (`fp_neighborhood_json(seed, cols, rows)`) does not take chunk
  coordinates; the chunk interface is a protocol extension — it bumps
  the protocol version and keeps every v1 call's semantics unchanged.

### 4.2 Fear engine — Mortuary's Assistant mode (spec M5)

- **FR-10.** Dread is a number 0–100 per house, with rungs
  calm / unease / wrong / event. It is driven only by the specified
  systems: darkness (+), time spent in one dark room (+), wrong
  details (+), completed tasks (−, grounding), light (+, relief).
  The coupling lives in one place, so "too dark" and "too scary" are
  never confused.
- **FR-11.** The engine only spends events at the rung the dial
  allows. At dial 0: zero horror events, dread frozen at 0.
- **FR-12.** The dial is a first-class setting (0–100), saved with the
  session. Turning the dial up adds systems; it never removes sandbox
  abilities. A player who never touches the dial never meets the
  horror.
- **FR-13.** Tasks are the loop: routine, checkable, grounding.
  Generated tasks name real rooms and real objects in the house the
  player is in — never scripted, never naming a room or object that
  does not exist. Kinds: check_lights, restore_power, catalog,
  close_doors. No fail state by default; no timer unless a task
  itself carries one (none does in v1).
- **FR-14.** Wrong details are small, deniable, seed-derived events.
  Each one is written to an event log, so the player (and the
  developer) can verify it happened rather than imagined it. The
  same seed reproduces the same wrong details.
- **FR-15.** The dread systems work silent in v1. No horror mechanic
  depends on sound (sound is an open decision, §9).
- **FR-16.** The default session is a neutral sandbox at dial 0 with
  daytime held: walk houses, open doors, move chairs, toggle lamps,
  wander the neighborhood. No objective unless the player takes a
  task.

### 4.3 Realism — interiors and exteriors (spec M6)

- **FR-17.** Pitched roofs. (Explicitly out of scope in ADR 0002; in
  scope here, spec §4.3.) A house reads as a house from the street,
  not a box with a flat cap.
- **FR-18.** Exteriors: siding and trim materials, framed windows with
  sills, real front doors with porches, driveways, garage doors that
  match the garage program, yards with paths, fences, and trees as
  placed (not painted) objects.
- **FR-19.** Interiors: furniture (§2), fixtures (sinks, tubs,
  counters, shelves), floor and wall materials per program (tile in
  baths, wood in living), baseboards and door frames, clutter as a
  light pass (books, dishes) that never blocks circulation.
- **FR-20.** Proportions are the realism floor: zero rooms over their
  program cap over 200 seeds (bath ≤ 6 m², study ≤ 12 m²) — the M0
  gate, carried, not re-decided. The strict-reading residual (5
  studies, §2) stays recorded until Jim calls it.
- **FR-21.** Every house is enterable and walkable first-person, and
  its furniture stays interactive through the command model
  (open / toggle / move), with the mesh following state (ADR 0003).
  The realism pass adds no furniture kind that breaks a placement
  rule.

### 4.4 Light play (spec M3 — dependency of M5, built in M3)

- **FR-22.** Light sources are domain objects: ceiling fixture, lamp,
  porch light, flashlight, window daylight — each with position,
  radius, and state (on / off / flicker / broken).
- **FR-23.** Switches are interactions through the command model. Every
  light has a switch the player can find and use. No room is ever
  unwinnably dark: there is always a working light or the flashlight.
- **FR-24.** Daylight is a cycle. Time of day moves; windows admit
  daylight by sun position; night is genuinely dark. The player can
  hold the cycle at day in the sandbox.
- **FR-25.** Light couples to dread (FR-10): darkness raises dread, a
  lit room lowers it, a flickering light is a dread event, not a
  render bug.

## 5. Non-functional requirements

- **NFR-1 — Performance budgets (spec §4.4, gates not aspirations).**
  Release-built, in-process: one house ≤ 5 ms; one neighborhood chunk
  ≤ 50 ms; generation work ≤ 8 ms/frame — generation never stalls a
  frame; anything bigger is split or deferred. Measured baseline
  (2026-11-12, debug build, this machine): 200 houses in 0.72 s wall
  (3.6 ms/house including process spawn and file IO); one 2×2
  neighborhood in 0.03 s. A phase fails its gate if it breaks these.
- **NFR-2 — Determinism.** Same seed → same house, same furniture,
  same chunk, byte-identical. Horror events derive from the seed too,
  so a haunting is reproducible and debuggable. The only
  nondeterminism allowed is the player's own input.
- **NFR-3 — Memory at distance.** Evicted chunks release their mesh
  and JSON. Memory does not grow linearly with distance walked: after
  the working set (the ring around the player) is full, memory
  plateaus. Verified by the streaming soak (§7, AC-2).
- **NFR-4 — No jumpscares, ever, without a separate decision.**
  The engine never does a loud sting plus a face. This is a hard
  rule, not a default. If a jumpscare-like event is ever wanted, it
  needs its own decision and its own ADR — it is not in this SRD.
- **NFR-5 — Variety does not shrink.** The variety lower bound
  (≈ 4.5 × 10⁹ houses) and the per-family layout distinctness (100
  distinct layout hashes over 100 seeds per family) hold after M4–M6
  land. Chunk content hashes stay distinct across a session (FR-6).
- **NFR-6 — Hot path stays allocation-light.** Generation is pure
  computation over the seeded RNG; no IO in the core, no per-frame
  regeneration. Meshes are built once per house/chunk and cached by
  seed; the renderer reuses them.

## 6. Data model and interfaces

### 6.1 Domain types (spec §5 — new types live in `domain/`)

```
LightSource     { id, kind (ceiling | lamp | porch | flashlight | window),
                  room_id | carried, radius, state (on | off | flicker | broken) }
Chunk           { cx, cz, blocks, lots, houses, state (requested | generating
                  | ready | installed | evicted) }
DreadState      { level 0–100, rung (calm | unease | wrong | event),
                  dial 0–100, event_log }
Task            { id, kind (check_lights | restore_power | catalog | close_doors),
                  target (room_id | furniture_id), done }
Interaction     open(door_id) | close(door_id) | toggle(light_id)
                | move(furniture_id, rect) | take(task_item) — commands, spec §3
```

Existing types stand: `House, Room, Opening, Style, Furniture,
FurnitureKind, FurnitureState, Mesh, Neighborhood, Layout`. All new
behavior goes in `use_cases/` (furnish, interact, light, stream,
haunt); all IO in `adapters/`; each new use case that draws a boundary
gets an ADR (furnish, light, stream, haunt).

### 6.2 The WASM boundary (protocol v1, ADR 0004 — extended, not replaced)

What crosses the boundary, and in which direction:

| Direction | Payload | Notes |
|---|---|---|
| Shell → core | world seed, chunk coordinates (cx, cz) | New in the chunk extension (FR-9). v1 takes (seed, cols, rows) for a fixed neighborhood. |
| Shell → core | interaction commands (`open`, `toggle`, `move`, `take`) | The same command model the CLI uses — one command model, every surface riding it (spec §3). |
| Shell → core | horror dial setting, time-of-day hold | Settings, not generation inputs; they never change a chunk's seeded content. |
| Core → shell | chunk JSON | Rooms, furniture, openings, streets, lots, placed houses with rotation and seed — everything the shell needs for geometry and collision. Same text the CLI writers emit. |
| Core → shell | chunk mesh (`FPM1` binary) | Positions, indices, material ids, material table. One mesh per chunk, built once and cached by seed. |
| Core → shell | house state updates | `open_doors`, furniture states, light states — the state the renderer shows. |
| Core → shell | dread state + event log | Dread level and rung, and the log of wrong details (FR-14). The shell displays them; it never computes them. |

### 6.3 Shell vs core responsibilities

- **Core owns:** generation (houses, chunks, furniture, tasks, events),
  all state (doors, furniture, lights, dread), determinism, and the
  event log. The core never draws, never touches the DOM or WebGL, and
  does no IO.
- **Shell owns:** rendering (three.js), lights and shadows as rendered,
  input (pointer lock, WASD), the day/night visuals, and presenting the
  dread state and event log the core hands it. The shell never generates
  content and never decides whether an event happens.

## 7. Acceptance criteria

- **AC-1 — Dial 0 produces zero events.** A long simulated session
  (**TARGET:** 1,000 player actions across 10 seeds, dial 0) produces
  zero horror events and dread stays at 0 throughout. This is the M5
  exit test, and it is also the regression test that runs on every
  later change to the fear engine.
- **AC-2 — Streaming soak.** Walk in one direction for **TARGET:** 10
  minutes. No frame in which generation work exceeds 8 ms/frame
  (NFR-1). No visible pop-in. No repeated neighborhood: every chunk
  visited has a distinct content hash. Memory plateaus after the
  working set is full (NFR-3).
- **AC-3 — Chunk boundary walk.** Cross a chunk boundary on foot: no
  hitch (generation work ≤ 8 ms on the crossing frames) and no
  pop-in — the next chunk is already installed when the player
  arrives (FR-4).
- **AC-4 — Chunk revisit is byte-identical.** Leave a chunk, walk away,
  come back: the chunk's JSON and mesh are byte-identical to the first
  visit (FR-1, FR-5, NFR-2).
- **AC-5 — Tasks name real things.** At dial > 0, generated tasks name
  real rooms and real objects in the house the player is in, and
  completing a task lowers dread (FR-13, FR-10).
- **AC-6 — Wrong details are verifiable.** Every wrong detail that
  occurs appears in the event log, and the same seed reproduces the
  same wrong details (FR-14, NFR-2).
- **AC-7 — Realism, judged by Jim.** The seed 42 house reads as a
  realistic house from the street and from inside, judged by Jim in
  the viewer. (Spec M6 exit criterion — the judge is named, not
  approximated.)
- **AC-8 — No unwinnably dark room.** Every room has a reachable
  working light or the flashlight; a dark room at night is navigable
  by flashlight alone (FR-23, spec M3 exit).
- **AC-9 — Budgets hold.** One house ≤ 5 ms, one chunk ≤ 50 ms,
  generation ≤ 8 ms/frame, measured release-built and in-process
  (NFR-1).
- **AC-10 — Invariants per chunk.** FR-7's invariants hold per chunk
  over seeds (100 seeds per layout family, as ADR 0004 tested the
  fixed neighborhood).

## 8. Traceability

| Requirement | Spec phase | Spec section |
|---|---|---|
| FR-1 – FR-9 (streaming, chunks, WASM chunk interface) | M4 | §4.5, §6 M4 |
| FR-10 – FR-16 (dread, dial, tasks, wrong details, silent, neutral default) | M5 | §4.7, §4.8, §6 M5 |
| FR-17 – FR-21 (pitched roofs, exteriors, interiors, caps, walkable/interactive) | M6 | §4.3, §6 M6 |
| FR-22 – FR-25 (light sources, switches, day/night, dread coupling) | M3 (dependency of M5) | §4.6, §6 M3 |
| NFR-1 (budgets) | M2–M4 gates | §4.4, §7 |
| NFR-2 (determinism) | standing | §3, §7 |
| NFR-3 (memory at distance) | M4 | §4.5 |
| NFR-4 (no jumpscares) | standing rule | §4.7, §7 |
| NFR-5 (variety) | standing | §4.2, §7 |
| NFR-6 (allocation-light hot path) | M2–M4 | §4.4 |
| AC-1 (dial-0 test) | M5 exit | §6 M5, §7 |
| AC-2 – AC-4, AC-10 (soak, boundary, revisit, invariants) | M4 exit | §6 M4, §7 |
| AC-5, AC-6 (tasks, wrong details) | M5 exit | §6 M5 |
| AC-7 (realism judged by Jim) | M6 exit | §6 M6 |
| AC-8 (light) | M3 exit | §6 M3 |
| AC-9 (budgets) | M2–M4 gates | §4.4, §7 |

## 9. Open decisions

Recorded, not decided. The spec's defaults are noted where it states one.

1. **Sound in or out of v1.** Spec default: out — dread must work
   silent first (§4.7, FR-15). If Jim wants atmosphere audio from the
   start, it becomes an M3-adjacent workstream.
2. **Horror dial default.** Spec default: 0 (neutral sandbox). Jim may
   want the default higher, or a first-run question.
3. **What tasks pay.** Spec: rewardless in v1 — grounding is the whole
   reward. Do tasks pay anything (progress, unlocks, a collection)?
   Undecided.

Also open, as design decisions inside M4 (not Jim-level decisions):
chunk granularity (one city block vs one street segment, spec §4.5)
and the exact metered-install count per tick (TARGET: ≤ 2, FR-3).
The soak duration (TARGET: 10 minutes, AC-2) and the dial-0 simulated
session length (TARGET: 1,000 actions / 10 seeds, AC-1) are targets to
confirm when the tests are written, not measured facts.

## 10. Constraints

- **Std-only core, zero crates.** The core compiles to
  `wasm32-unknown-unknown` with no dependency added. The WASM adapter
  stays thin; no wasm-bindgen (ADR 0004).
- **200-line file ceiling**, including tests. Splits are by cohesion,
  as in ADR 0003 (furnish → furnish_clear, furniture_mesh →
  furniture_parts).
- **Strict TDD.** A failing test seen red first for every behaviour;
  never weaken a test.
- **`make check` green** (cargo test single-threaded + clippy
  `-D warnings`) before any claim of done, and before the next phase
  starts. Each phase (M4, M5, M6) is done when its exit criteria pass
  (§7), not when its code exists.
- **ADR for every new boundary** (stream, haunt, light).
- **Determinism everywhere** (NFR-2); **no jumpscares** (NFR-4).
