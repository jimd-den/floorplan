# ADR 0009: Realism — pitched roofs, trim, grounds, and interiors

## Status

Accepted. Implements SRD-001 §4.3 (FR-17 – FR-21) and the M6 exit
criterion AC-7. The screenshots are seed 42 Modern and seed 0
Colonial, regenerated with `floorplan render`.

## Context

The house mesh was a flat roof cap, one wall material, openings with
no frames, a porch slab, and rooms with no baseboards or fixtures
beyond furniture. It read as a plan extruded upward, not as a house
from the street or a room from inside. M6 spends geometry on the
cues that make those two reads work, while keeping the mesh inside
the first-person budget.

## Decision

### Roofs (`use_cases/roof.rs`, `use_cases/porch.rs`)

One pitched gable over the footprint bounding box (footprint rects
plus the garage), with a 0.4 m eave overhang. The ridge runs along
the longer axis; the two slopes are quads and the gable ends are
siding triangles. Pitch is 0.5 of the half-span. The porch gets its
own flat roof at 2.35 m, two posts, two steps, and a slab, so a front
door reads as a front door. The top floor keeps an interior ceiling
slab under the roof.

### Materials (`use_cases/materials.rs`)

The first 29 ids stay stable (structural, program, furniture, door).
Ten materials append after them: `trim`, `siding`, `window_frame`,
`glass`, `baseboard`, `door_frame`, `driveway`, `trunk`, `leaf`,
`clutter`. Street and lot append last in the neighborhood table, so
their ids move with the table rather than staying pinned.

### Walls, frames, linings, baseboards

`wall_boxes_toned` classifies each segment by how many rooms own it:
one edge is exterior (siding), two is interior (painted wall). A wall
box has one material, so `use_cases/lining.rs` adds a thin painted
lining on the room side of exterior walls, split around openings, so
siding never shows inside. `use_cases/trim.rs` frames every window
(four members and a glass pane), every door (two jambs and a head),
and runs baseboards along the inner face of every room. Full-height
wall boxes reach 0.1 m below the floor slab, so the slab edge is not
a coloured band around the house.

### Grounds (`use_cases/grounds.rs`)

A driveway is a pad on the garage's east side, plus a connector to
the street for south-, north-, and east-facing houses. A west-facing
house is skipped: its connector would cross the house. One or two
trees stand in the back yard of every lot, clear of the house's back
setback and inside the lot. Placement is deterministic in the
neighborhood seed and the lot index.

### Clutter (`use_cases/clutter.rs`)

Books and dishes sit on tables, desks, counters, and nightstands,
never on the floor, so circulation is untouched. A moved piece
carries its clutter with it.

### Budgets and view

The house guard moves to 7,000 vertices / 10,000 faces and the
neighborhood guard to 90,000 / 140,000, because M6 deliberately
spends geometry on reads-as-a-house. NFR-1 remains a separate,
release-built measurement. The exterior screenshot is a three-quarter
view (the eye is offset east, the target stays centred), so the roof
and one side read instead of a flat elevation.

## Bug found by the pass

Vertical-wall window and door frames used the opening's x coordinate
for their z span, flinging frames metres outside the house. The
symptom was floating white posts in the exterior view; the material
bounds in the OBJ showed `window_frame` at z −1.15 and z 19.70 for a
house spanning z 0..10.3. Fixed in `trim.rs`; the regression test is
`a_window_frame_on_a_vertical_wall_stays_at_its_opening`.

## Alternatives

- **Keep the flat cap.** Rejected: the house does not read as a
  house from the street, which is AC-7.
- **A two-material wall box.** Rejected: doubles the wall geometry
  and still needs opening-aware splitting; the lining is thinner.
- **Random tree placement.** Rejected: NFR-2 requires determinism.

## Consequences

- Seed 42 Modern reads as a ranch with a gable roof, framed windows,
  a porch-less front door, and a painted interior with baseboards.
- Seed 0 Colonial shows the porch, steps, posts, and a two-storey
  gable.
- The Node harness is 550 checks, 0 failures, after the material
  table update; it checks the house faces against the native OBJ and
  lets grounds append after the core.
- The screenshots in `docs/screenshots/` are regenerated from the
  same binary.

## Revisit if

Release-built NFR-1 timing misses the 8 ms/frame budget, or the
shell needs a smaller payload than the new guard.
