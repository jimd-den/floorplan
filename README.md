# floorplan

A clean-architecture floor plan generator. One u32 seed in, one house
out — deterministic, so the same seed always gives the same house,
byte-identical SVG and JSON.

## The model

```
footprint -> BSP subdivision -> programs -> doors -> windows -> House
```

- **Styles** (six, each a real bias, not a label): Ranch (single
  storey, wide and low, open living/kitchen), Colonial (two-storey,
  symmetric, central hall), Craftsman (wide porch, fireplace core),
  Modern (flat, open plan, large glass), Cottage (small, steep,
  cozy), Townhouse (narrow, deep, vertical stack).
- **Footprints**: rectangle, L, U and T, with dimensions drawn from
  the style's range on a 0.1 m grid.
- **Rooms**: BSP subdivision of each footprint rectangle, with a
  2.4 m minimum side so there are no sliver rooms. Programs are
  assigned by rule: the largest leaf near the entry is living, the
  kitchen sits next to living, bedrooms cluster farthest from living,
  baths are small and next to bedrooms, at least one bath per two
  bedrooms.
- **Doors**: one door on every BSP split and every footprint join, so
  every room is reachable from the entry by construction. The entry
  door is wider (1.0 m) and sits on the front wall.
- **Windows**: on exterior walls only, sized by style (Modern large,
  Cottage small), at least one per habitable room. The assignment is
  daylight-aware, and a repair swaps habitable programs off leaves
  that cannot hold a window — including leaves the garage would
  orphan, since the garage slides along the east wall to the spot
  that orphans the fewest rooms.

## 3D houses

The same generator extrudes each house to a real mesh — one
internal `Mesh`, written as OBJ (with `.mtl`) and glTF 2.0 (JSON,
buffer embedded as base64) — not voxels.

- **Walls** are boxes (0.15 m thick, 2.7 m per storey) split around
  real openings: a door/window is an actual gap in the wall, with
  a lintel above it and a sill below windows. Opening honesty is
  tested geometrically over 200 seeds. Exterior walls are siding;
  interior walls get a painted lining, baseboards, and door frames.
- **Per floor**: a floor slab and a ceiling slab per room.
  **Stairs** for two-storey houses are solid stepped boxes in the
  ground hall.
- **Roof**: a pitched gable over the footprint and garage, with a
  0.4 m eave overhang and the ridge on the long axis; gable ends are
  siding. The top floor keeps an interior ceiling under it. A porch
  gets its own roof, posts, and steps.
- **Windows and doors**: four-member window frames with a glass pane,
  two jambs and a head per door, so an opening reads as a window or a
  door from both sides.
- **Grounds** (neighborhood mesh): a driveway pad on the garage side
  with a connector to the street where the route is clear, and one or
  two trees per lot in the back yard.
- **Materials**: one per room program (living, kitchen, dining,
  bedroom, bath, hall, entry, study, laundry, garage) plus
  wall/ceiling/roof/stairs/porch, siding, trim, window frame, glass,
  baseboard, door frame, driveway, trunk, leaf, and clutter.
- **Budget**: a house stays under 7,000 vertices / 10,000 faces and
  the 2×2 neighborhood under 90,000 / 140,000, guarded by
  `tests/mesh_budget.rs`.

```sh
floorplan generate --seed 42 --style modern --obj house.obj --gltf house.gltf
# house.obj + house.mtl + house.gltf; both flags optional.
# --obj/--gltf cannot be combined with --count.
```

## Neighborhoods

Random layouts on top of the same house generator — no second house
generator. Three layout families (grid, offset, cul-de-sac), picked
by `seed % 3`, each with its own street pattern. Streets (8 m, at
least one through-street plus cross streets), lots along the streets
with setbacks (front 5 m, back 3 m, sides 2 m), one house per lot,
each facing its street. Lot size biases the style: small lots get
Cottage/Townhouse, wide lots get Ranch/Colonial/Modern/Craftsman.
Every house fits its lot (the maximum house extent is 26 × 16 m);
every lot has street frontage; no two houses overlap. The default
is 2 × 2 blocks, ~16–24 houses. See ADR 0004.

```sh
floorplan neighborhood --seed 42
# neighborhood-42.svg (streets, lots, footprints in place, style + seed labels)

floorplan neighborhood --seed 42 --blocks 3x3 --out nb.svg --json nb.json
```

## Streaming (M4)

The neighborhood is unbounded. A **chunk** is one 2 × 2-block
neighborhood derived only from the world seed and its coordinates
(`use_cases::chunk`). Chunk (0,0) is the fixed neighborhood above,
unchanged; other chunks are generated from a seed mixed from the
world seed and `(cx, cz)` and translated to a 400 m grid.

`use_cases::stream::Streamer` is a pure lifecycle state machine the
shell advances once per frame: request the 3 × 3 ring, generate at
most one chunk, install at most two, evict beyond two chunks away.
A chunk about to be entered is installed before arrival, and a
revisit rebuilds byte-identical content. The working set is bounded
at 25 chunks, so memory plateaus. See ADR 0007.

The WASM boundary exposes it as protocol v2:

```text
fp_chunk_json(seed, cx, cz)   chunk JSON: {chunk, state, neighborhood}
fp_chunk_mesh(seed, cx, cz)   the chunk's FPM1 mesh
```

Every v1 export is unchanged. Chunk (0,0) embeds the fixed
neighborhood's JSON and mesh byte-for-byte.

## WASM

The core compiles to `wasm32-unknown-unknown` with zero crates and
no wasm-bindgen — a thin adapter (`src/adapters/wasm.rs`) over the
same generator. The web shell calls raw `extern "C"` exports and
reads the results out of linear memory (protocol v2, see
`docs/adr/0004-neighborhood-wasm.md` and `docs/adr/0007-streaming-chunks.md`): `fp_version`,
`fp_alloc`/`fp_free`, `fp_last_len`, `fp_house_json`,
`fp_neighborhood_json`, `fp_house_mesh`, `fp_neighborhood_mesh`,
`fp_chunk_json`, `fp_chunk_mesh`, `fp_house_lights`,
`fp_house_lights_toggled`, `fp_daylight`, `fp_light_toggle_state`,
`fp_house_tasks`, `fp_haunt_step`, `fp_house_interact_mesh`.
JSON payloads are the same text the CLI writers emit; mesh payloads
are a binary `FPM1` blob (positions, indices, material ids, material
table). The built module is committed as base64 at
`wasm/floorplan.wasm.b64` (release, 277,218 bytes) — the binary
itself can't travel through this repo's text-only push path — and
decodes back with one command:

```sh
make wasm            # debug module + Node harness (571 checks vs native)
# Release module:
RUSTFLAGS="-C panic=abort" cargo build --release --lib --target wasm32-unknown-unknown
# Decode the committed copy back to the binary:
base64 -d wasm/floorplan.wasm.b64 > floorplan.wasm
# tools/wasm_check.mjs --wasm <path> --bin <native binary> verifies a build.
```

## Screenshots

`floorplan render` draws a first-person PNG of a house or a street with
a std-only software rasterizer — no browser, no crates. It is an
inspection tool, not the runtime renderer; the WASM shell still draws
the frames (ADR 0006).

```sh
floorplan render --seed 42 --style modern --view exterior
floorplan render --seed 42 --style modern --view interior --out in.png
floorplan render --seed 42 --view street --blocks 2x2 --out street.png
```

Views are `exterior` (a three-quarter view of the footprint), `interior`
(the largest ground-floor room, standing clear of furniture), and
`street` (the busiest street, looking along it). Output is 1280×720,
default `render-<seed>-<view>.png`. `docs/screenshots/` holds:

- `seed42-modern-exterior.png` — gable roof, siding, framed windows.
- `seed42-modern-interior.png` — painted walls, baseboards, window,
  door frame, table and clutter.
- `seed42-neighborhood-street.png` — the M6 neighborhood, with trees.
- `seed0-colonial-exterior.png` — two storeys and a porch with steps,
  posts, and its own roof.

The M6 pass is ADR 0009; the screenshots regenerate from the same
binary with the commands above.

## Fear engine (M5)

The procedural sandbox horror layer is a **dial, not a story**. Pure
use cases over `DreadState` (SRD-001 §4.2, ADR 0005):

- **Dread** rises from darkness and time spent in it, falls from light
  and from completed routine tasks. `DreadRung` (`Calm < Unease <
  Wrong < Event`) classifies 0–100 once, in the domain.
- **The dial** caps what the engine may spend: 0 is silent and freezes
  dread, 1–33 spends unease details, 34–66 wrong details, 67–100 event
  details.
- **Tasks** name real rooms and furniture in the house being played
  (`CheckLights`, `RestorePower`, `Catalog`, `CloseDoors`); a kind with
  no target is skipped, never invented.
- **Wrong details** are seed-derived (`DoorAjar`, `ChairMoved`,
  `LampOn`, `LightFlicker`) and append-only in the event log.

The shell supplies the systems (darkness, dark minutes, tasks done,
light relief) as data — M3 light play fills those values without
changing the interface (ADR 0008). `tests/haunt.rs` proves the
dial-0 property over 10 seeds × 100 actions.

## Light play (M3)

Lights are domain objects (`domain/light.rs`): ceiling, lamp, porch,
flashlight, and window, each with a position, radius, and state
(on / off / flicker / broken). Every room gets a ceiling light and
every lamp a light of its own; windows admit daylight by sun
position. `House.lights` is derived after furnishing, so the mesh
and the lighting cannot disagree.

- **Switches** are commands through the one command model:
  `ToggleLight { light_id }`. A broken light is refused, never
  repaired. Every room is navigable: a working ceiling light or the
  carried flashlight (AC-8).
- **Daylight** is a 24-hour cycle (`Daylight::at`, `advance`,
  `held_at`). Night is genuinely dark; the sandbox can hold it at
  day.
- **Dread coupling** lives in `use_cases/light_dread.rs`: darkness
  and light relief are complements, and a flicker is offered as a
  dread event, not a render bug (FR-25).

The WASM boundary (protocol v2) exposes `fp_house_lights`,
`fp_house_lights_toggled`, `fp_daylight`, and `fp_light_toggle_state`.

## Room-size caps

Bath is capped at 6 sq m and study at 12 sq m. Over-cap leaves are
re-tiled (re-split and reassigned) — over 200 seeds, 0 baths and
0 studies over cap (one style per seed). The strict reading (all six
styles × 200 seeds) still has 5 studies at 12.0–13.9 sq m in thin
rects. See ADR 0003.

## Variety

The honest lower bound is **≈ 4.5 × 10⁹ distinct houses** (see
`docs/reports/variety-count.md` for the counted product). The binding
cap is the 2³² seed space, still ~4,300× the million-house
requirement. Program assignment is deterministic and is not counted
as a factor. Empirically, 1000 seeds give 1000 distinct plan hashes
(the hash excludes the seed, so that means 1000 distinct plans).

## Running it

```sh
make build            # release binary at target/release/floorplan
make check            # cargo test (single-threaded) + clippy -D warnings
make wasm             # debug WASM module + Node harness (364 checks)

# One house, style picked by the seed:
floorplan generate --seed 42

# A named style, SVG and JSON out:
floorplan generate --seed 42 --style craftsman --out house.svg --json house.json

# Bare --json writes house-<seed>.json next to the SVG.
# N houses (seeds 100..103) into a directory:
floorplan generate --seed 100 --count 4 --dir houses

# The styles:
floorplan --list-styles
```

Bad input (an unknown style, a non-numeric seed, an unknown flag)
exits non-zero with a plain error on stderr.

## Layout

```
src/domain/            the plan model: House, Room, Opening, Style, Mesh,
                       Neighborhood (pure data)
src/use_cases/         the pipeline: footprint, bsp, programs, doors,
                       windows, generate, plus the variety hash, the
                       extrude and caps use cases, and neighborhood
src/adapters/          SVG renderer, JSON writer, OBJ/glTF writers,
                       neighborhood SVG/JSON, CLI
src/frameworks_drivers/ splitmix64 RNG on std only
tests/                 determinism, reachability (200 seeds), variety
                       (1000 seeds), walls (200 seeds), neighborhood
                       invariants (100 seeds), CLI end to end
docs/adr/0001-generator-model.md   the core boundary
docs/adr/0002-3d-and-neighborhood.md  mesh, caps, neighborhoods
docs/reports/variety-count.md      the honest variety count
examples/              one SVG per style, plus a 3D house (OBJ/glTF)
                       and a neighborhood SVG
```

House rules: Rust edition 2024, std only, zero external crates,
200-line ceiling per file including tests, strict TDD (a failing
test seen red first for every behaviour), `make check` green before
any claim of done.
