# Framing + basements rework — narrower doors, trim that stops at doors, real concrete

Working tree: `/home/hatch/tmp/floorplan-functional/tree` (branch
`feat/functional-houses`). Steps 1–2 (framing skeleton, basements)
landed earlier; this report covers steps 3–5, the realistic-house
corrections. Ready for the parent to push; do not push from here.
The shell artifact is untouched. Protocol stays **v5** — the JSON
and mesh shapes did not change, only values and box counts.

## What changed

**Step 3 — narrower doors.** `src/use_cases/door_width.rs` now
encodes realistic residential widths: entry door **0.9 m** single,
whatever the front; interior standard **0.8 m**; small room
(area < 6.0 m² — closet/bath/laundry) **0.7 m**; clamp
`[0.6, opening − 0.6]`, quantized to 0.1 m. The old
size-by-opening table (1.6 m double / 1.0 m wide) is gone. A double
door is at most **one per floor**, capped at **1.2 m** (two 0.6 m
leaves), and only on the **living/dining opening** — the one place
a real house puts one. A long shared wall elsewhere does not make
a double doorway; those are singles. `place_doors` designates the
double from the backbone links (`link_joins` for Living+Dining);
`door_panel.rs` splits on `is_double_door` (the 1.4 m panel
threshold is gone). Seed 42 proves it: **12×0.8, 1×0.6 (clamped
tiny opening), 1×0.9 entry, 0 doubles** — the living and dining
rooms are not adjacent in seed 42, so it has no double at all.
`tests/doors.rs` is new (the `doors.rs` unit tests moved there
whole, so `doors.rs` is back under the ceiling at 126 lines).

**Step 4 — trim stops at doors.** `src/use_cases/trim.rs`:
`baseboard_boxes` is now a public function returning the band
boxes, and each band is split at door openings on that room edge
(`openings_on` + `room.rect.edges()`). Only bands whose height
overlaps the 2.1 m door head split — baseboard (0–0.1) and chair
rail (0.90–0.94) stop at the door and resume past it; the crown
(2.62–2.68) runs continuously over the door. `tests/trim.rs`
proves it, including two doors on one wall splitting the band
twice. The seed 42 interior render shows the chair rail ending
at the door frame instead of crossing the brown panel.

**Step 5 — concrete reads as foundation.** New
`src/use_cases/foundation.rs`: a concrete **stem wall** around
the ground floor's true outline, from the footing (y −0.3) to
**0.2 m above grade**, 0.2 m thick (the basement foundation-wall
thickness), with gaps at exterior doors. Its inner face is flush
with the wall's inner face, so it never shows inside a room; it
stands proud of the siding by 0.05 m outside. The outline is
computed by overlap, not by exact segment equality: a long wall
beside two short rooms is shared along its whole length, so
`wall_segments_toned`'s count==1 rule wrongly called those
interior walls exterior (the first stem-wall build put 51 boxes
on seed 42, 34 of them inside rooms, and showed as a grey strip
indoors). `exposed_ranges` subtracts every neighbour's overlap,
so seed 42 now gets 17 stem-wall boxes, 0 intruding. The porch
is a **wood deck** now, not concrete: the PORCH material went
from grey `[0.70, 0.70, 0.68]` to wood `[0.55, 0.42, 0.28]`,
so it can never be confused with CONCRETE `[0.62, 0.62, 0.60]`.
The buried porch steps (tops −0.04, −0.08) are removed — the deck
is at grade and takes no steps. The base slab stays buried
(y −0.3 to 0); concrete is now foundation-only, the lot stays
grass.

## Gate and budgets

- `make check` exit 0: **493 passed, 0 failed** (was 489 after
  step 3; +2 trim tests, +1 foundation test, +1 porch test).
- `cargo check --all-targets` exit 0. No file over 200 lines.
- Mesh budget moved twice, with cause, in `tests/mesh_budget.rs`:
  the trim split added 24 boxes on Colonial (8692 vertices), the
  stem wall 63 more (9196). Budget is now **9500 vertices /
  14200 faces**; Colonial, the largest house, is 9196 / 13724.
  The neighbourhood budget (115k) still passes.
- `tests/extrude.rs` updated, not weakened: the highest concrete
  is now the stem wall top (0.2 m), not the slab top — the test
  asserts both the slab top at floor level and the stem wall top.

## WASM

- Release build: `RUSTFLAGS="-C panic=abort" cargo build
  --release --lib --target wasm32-unknown-unknown`.
- `wasm/floorplan.wasm.b64` regenerated.
- WASM sha256: `da77d4610d67587d787d823dec93237e2d2d6a799662c93bdf88ac8cef79555a`
  (410,654 bytes).
- `node tools/wasm_check.mjs --wasm
  target/wasm32-unknown-unknown/release/floorplan.wasm --bin
  target/release/floorplan`: **726 passed, 0 failed**.
- Protocol version: **5** (unchanged).

## Screenshots (seed 42, release binary)

Under `/home/hatch/workspace/floorplan-framing-renders/`:

- `step3-doors-interior.png` / `step3-doors-exterior.png` —
  doors are 0.8 m singles; the entry is a proportional 0.9 m.
  (Caveat: the left interior door still looks large — the camera
  stands 1.4 m from it in the narrow 2.8 m Ranch living room, so
  a correct 0.8 m door fills the frame. The JSON widths are the
  proof; a viewpoint change to stand the camera back was tried
  and reverted — a corner eye is closer to the walls than the
  centre, and made it worse.)
- `step4-trim-interior.png` / `step4-trim-exterior.png` — the
  chair rail stops at the door frame on both visible doors.
- `step5-concrete-interior.png` /
  `step5-concrete-exterior.png` / `step5-concrete-basement.png` —
  no grey rectangle on the grass (the porch is a brown wood
  deck), a concrete foundation strip at the base of the walls,
  basement camera intact.
- `final-interior.png` / `final-exterior.png` /
  `final-basement-view.png` — copies of the step 5 renders.
  - final-interior sha256:
    `95bf653e6942ba621eaea7d840e7a32edfa077c05ae0d57e7b01c1d1bfd5694c`
    (identical to the step 4 interior — the stem wall does not
    show indoors).
  - final-exterior sha256:
    `3f5b6e45d62ef7576c9fc4233926e60d27774a600492db168ef9605e9e5bc681`
  - final-basement-view sha256:
    `dde57dc7867559c5512b1a45828f0a25978e2104e855067a9d9ee4f0e6d56590`

## Kept

Framing skeleton, basements (including the `--view basement`
camera), base-first order, walls then doors, rooms traversable,
furniture removed, program caps and no-jumpscare / horror-dial-0
rules untouched. Nothing committed or pushed; the shell is
untouched (it still runs the v5 WASM `ed36f44e…` — a shell update
to this build is a separate step, and the parent should push the
new WASM first).
