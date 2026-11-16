# Curved-streets build report

Milestone: organic fluid streets and sidewalks — curves replace the
grid (spec `spec-curved-streets.md`, ADR 0010). Built in
`/home/hatch/tmp/floorplan-curved/tree`, a copy of pushed head
017d1f9a. **Not pushed to GitHub.**

## Gate results

- `make check`: **exit 0** — 387 tests passed, 0 failed, across 36
  suites (lib 208); clippy `-D warnings` clean.
- `cargo check --all-targets`: exit 0.
- Line budget: no file over 200 lines (checked by hand).
- `make wasm` / Node harness `tools/wasm_check.mjs`: **571 passed,
  0 failed**.
- Seed 42 render check: see P3 below.

## P0 — Geometry (test-first, 11 tests)

New `domain/curve.rs` (Pt, Polyline: arc length, point_at,
tangent_at, normal_at, offset), `domain/spline.rs`
(`sample_catmull_rom`, `catrom`), `domain/polygon.rs`
(signed_area, point_in_poly, segments_intersect, polys_overlap).
RED seen first: 10 assertion failures before the real
implementations. `curve.rs` hit 223 lines and was split at the
real seam (sampler → `spline.rs`). Winding rule: up-facing is
counterclockwise in plan (x east, y south), i.e. signed_area < 0.

## P1 — Domain model, street network, lots, consumers

- **Domain** (`domain/neighborhood.rs`): `Street { control,
  samples, bulb_at, bulb_center, junctions }`; `Lot { outline,
  street, center_s, side, frontage, depth }`; `PlacedHouse {
  house, lot, rotation, center, walkway }`; `Neighborhood {
  seed, blocks, streets, lots, houses }` (no `layout`).
  Rotation is degrees, clockwise from south (0 south, 90 west,
  180 north, -90 east). `domain/layout.rs` now holds only
  `Density`; `use_cases/layout.rs` deleted.
- **Street network** (`use_cases/street_network.rs`): spine across
  the area (control every ~85 m, lateral jitter ±22 m); branches
  1 + `range_usize(cols.min(rows))`, alternating north/south, each
  ending in a 300° bulb arc, radius 14 m. 6 tests (deterministic,
  seeds differ, spine spans, branches join, bulbs, smooth
  < 30° turns, every street ≥ 40 m).
- **Lots** (`use_cases/lots.rs`, `lot_geometry.rs`,
  `walkway.rs`, `place_house.rs`): frontage by density, depth by
  density; overlap filter keeps lots == houses; fit check
  (all 4 house corners inside the lot polygon, else downgrade to
  Townhouse once, else skip). Walkway ribbon 1.2 m from the
  sidewalk outer edge (offset 5.5 m) to the entry door.
- **Consumers**: `chunk.rs` translate, `scene.rs` bounds,
  `viewpoint.rs` street_pose, `variety.rs` neighborhood_hash and
  the re-derived bound, `adapters/wasm.rs` PROTOCOL_VERSION 2 → 3,
  `adapters/neighborhood_json.rs` and `neighborhood_svg.rs`
  rewritten for polygons, `use_cases/grounds.rs` and
  `neighborhood_mesh.rs` rewritten (street asphalt ±4 m, sidewalk
  ribbons 4 → 5.5 m at y 0.04, lot ground as a ruled strip,
  walkway ribbon at y -0.004 in DRIVEWAY material).

### Geometric corrections found by measurement

- **Back-arc lot outline.** The old straight back edge was proven
  broken: on a 14 m-radius bulb a lot spanning ~2 rad has its back
  chord cutting across behind the lot's own centre, so the polygon
  did not contain the house — even a 6 m Townhouse failed on a
  28 m bulb lot. The outline is now front arc points followed by
  back arc points (same count, split at half; first back point is
  back-right, last is back-left). Both edges follow the curve.
  `grounds.rs` fence and `neighborhood_mesh.rs` lot ground were
  updated for it.
- **Rotation bug.** `rotation_for` returned -180 for north instead
  of the canonical 180 (`atan2(-0.0, -1.0)` returns -π); fixed by
  normalizing `deg <= -180.0` to `deg + 360.0`. The 180° porch test
  also picked the wrong corner; fixed to the porch's south edge.
- **True rotation, not reflection.** The old 90°/270° placement
  was a reflection (det -1), so old east/west houses were mirror
  images. The new continuous rotation is a proper rotation — a
  genuine fix.

### House-count tuning (2×2 Low, 20 seeds, target 8–40)

- First real `lay_lots`: mean 6.5, min 5 — count test FAILED.
- Back-arc fix: mean 7.1, min 5. Fit check cost little after it;
  the overlap filter cost ~2.1 lots.
- Two clearance ideas were tried and **reverted** after measuring
  them as washes: an asymmetric junction clearance (34 m on the
  branch's side, via `junction_sides`) and a 32 m branch start
  clearance. Each saved ~1.5 overlap drops but cost ~1.5 raw lots.
- What worked: branch length 50–90 m → **70–110 m** (shortest
  branches were nearly empty) and Low frontage 38–48 m →
  **32–42 m**. Final: **mean 10.4, min 8, max 13** (seeds 0–19).
  Med mean 14.7 (min 12), High mean 23.9 (min 19, max 30).

## P2 — Integration tests

- `tests/neighborhood.rs` rewritten: curved invariants over 100
  seeds (house corners inside lot polygon, front setback, no
  house/lot overlaps, lot fronts street geometrically, houses
  face street with align > 0.99, 100 distinct hashes, SVG/JSON
  byte-identical). 10 passed.
- `tests/layout.rs` and `tests/neighborhood_layouts.rs` deleted
  (families retired).
- `tests/density.rs` rewritten (ordering, curved invariants, lot
  sizes). **High threshold changed 32 → 18** with justification:
  the curved frame's street length (~370 m for 2×2) caps High at
  19–30 over seeds 0–19 (mean 23.9); the grid-era 32 assumed the
  grid's denser street network and is unreachable here. The
  ordering invariant (high > med > low) still holds every seed.
- `tests/grounds.rs` rewritten (driveways, trees inside lots,
  walkway reaches door, fence 1.1 m, sidewalks material 41,
  determinism). 8 passed.
- `tests/chunk.rs` rewritten (the old `rect` API): translation
  compares a chunk against its own untranslated neighborhood
  (chunk seeds depend on coordinates), invariants via polygons,
  span via all geometry points. 8 passed.
- `tests/scene.rs` and `tests/viewpoint.rs` rewritten for the
  curved frame (ground box covers every street sample and lot
  point; the street eye is within 4 m of a centerline sample).
- `tests/wasm.rs`: PROTOCOL_VERSION 2 → 3. `tests/variety_bounds.rs`:
  neighbourhood bound 332,474,904 → **110,824,968**
  (201 × 41³ × 8).

## P3 — WASM harness and seed 42 render check

- Harness updates: `fp_version is 2` → 3; `'sidewalk'` added to
  `groundNames`; the ground-names check now expects the last three
  materials to be street, lot, sidewalk (the mesh appends sidewalk
  last, id 41). Result: **571 passed, 0 failed**.
- Seed 42, 2×2: 2 streets (spine, 4 control points; branch, 3
  control points, bulb), 10 lots, 10 houses. All rotations
  continuous and non-cardinal (166.0, 173.0, -173.1, -13.8, -4.2,
  6.4, 8.3, 77.6, -0.3, -77.7). Walkway on all 10 houses.
- Seed 42 mesh: **45,376 verts, 61,708 faces, 42 materials**.
  Sidewalk 744 faces, driveway (walkways) 122, street 372, lot
  386. Every house faces its street (worst alignment 1.000).
- Seed 42 SVG renders (2 street, 10 lot, 10 house elements).

## Docs

- README "Neighborhoods" rewritten for spline streets
  (families gone; sidewalks and walkways described).
- `docs/reports/variety-count.md`: added a Neighborhoods section
  with the 110,824,968 bound.
- ADR 0010 (`docs/adr/0010-curved-streets.md`): spline streets
  replace the grid, protocol v3.

## Splits (line ceiling)

- `domain/placement.rs` (new, 150 lines): the placement cluster
  (`house_extent`, `placed_point`, `house_corners`,
  `rotation_for`, `front_for` + tests) moved out of
  `domain/neighborhood.rs` (215 → 79 lines). 12 import sites
  updated.
- `use_cases/place_mesh.rs` (new, 62 lines): `append_placed` /
  `place_point` + test moved out of
  `use_cases/neighborhood_mesh.rs` (208 → 162 lines).

## Clippy fixes (make check runs clippy -D warnings)

- `FRAC_1_SQRT_2` constant instead of the 0.7071 literal.
- Two collapsible `if let` nests in `lots.rs` collapsed with
  let-chains.
- `#[allow(clippy::should_implement_trait)]` on `Pt::add`/`sub`
  with a reason: the geometry code chains `add`/`sub`/`scale`
  in one expression; the operator traits would need `Mul` too
  and churn 35 call sites.

## Open / for the parent

- The three.js shell still asserts protocol v2 in `index.html`;
  it needs updating to v3 when this build lands (the JSON format
  changed).
- High-density floor is now 18, not 32 — see P2. If Jim wants
  more High houses, the lever is more street length, not a
  threshold change.
- Seed is still u32; the 110,824,968 neighbourhood bound is a
  paper floor, and reachable houses remain capped at 2³².
