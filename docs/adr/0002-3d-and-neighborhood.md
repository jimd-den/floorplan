# ADR 0002 — 3D extrusion and neighborhood layouts

Status: accepted.

## Context

The 2D plans were the test, not the product. The generator must
prove it can produce 3D houses for gameplay and different
neighborhood layouts — both, as its own standalone thing, not
voxels, not folded into another engine. The 3D houses must be real
mesh (OBJ and glTF), and the neighborhoods must be street-grid
layouts on top of the existing house generator. A known gap rides
along: room-size rules were not enforced, so a Modern house had a
5.6 × 8.1 m bath (45 sq m) and a 6.9 × 9 m kitchen.

## Decision

### One mesh, two writers

The domain gains one internal mesh representation
(`src/domain/mesh.rs`): `Mesh { vertices, faces, materials,
material_table }` — triangles with a per-face material id. The use
case `extrude` (House -> Mesh) builds it once; two adapters write
it: OBJ (with a companion `.mtl`, one material per room program
plus wall/ceiling/roof/stairs/porch) and glTF 2.0 (JSON, single
buffer, base64 embedded, hand-rolled std-only encoder). OBJ and
glTF are never two separate extrusions.

### Walls with real openings

Per floor, per room: a floor slab and a ceiling slab. Each wall is
a box (0.15 m thick, 2.7 m per storey) split around its openings —
full-height boxes between openings, a lintel above every door and
window, a sill below every window. A door/window opening is an
actual gap in the wall geometry, never a decal; opening honesty is
tested geometrically over 200 seeds. The SVG and the mesh now read
the same wall segments (`wall_segments` in `use_cases/walls.rs`);
the SVG's private duplicate was deleted.

Stairs for two-storey houses are solid stepped boxes in the ground
hall (0.25 m treads, 1.0 m wide, along the hall's longer side).
The roof is a simple flat slab cap: the top floor's ceiling slab,
and any ground ceiling with no room above it (an exposed wing like
the garage), is the roof material. **A pitched roof is explicitly
out of scope** — it is not built, and the README says so.

### Room-size caps, and their honest limit

Bath is capped at 6 sq m and study at 12 sq m (`use_cases/caps.rs`).
A bath leaf over the cap is never assigned bath, and bedrooms never
take a sub-cap leaf while a larger one can hold one. Where a leaf
can be strip-cut (a 2.4 m strip across the long side, remainder
Dining if no dining exists on the floor and the remainder keeps an
exterior stretch, else Laundry), the cap is enforced by splitting;
otherwise the leaf is reassigned.

The residual is geometric, not a bug, and is reported honestly:
over 200 seeds, 30 of 310 over-cap baths and 32 of 37 over-cap
studies are fixed; the rest cannot be fixed by reassignment (267
floors have no leaf ≤ 6 sq m at all) or by strip-cutting (a ≤ 6 sq m
bath needs essentially a 2.4 × 2.5 m room; most bath leaves are too
small in one dimension to host one without a sliver). A strict ≤ 6
sq m bath cap is impossible for most generated houses without a
re-tiling step (redrawing leaf boundaries across neighbours) or a
smaller minimum side. Neither is built here.

### The circulation extra-door fix

The walls tests found a plan flaw: a circulation extra door could
be placed on a 0.1 m shared wall, spilling across neighbouring
walls. `circulation::extras` now skips adjacencies with a segment
under 0.6 m (the minimum door width). Regression-tested.

### Neighborhoods

The use case `neighborhood` (seed -> `Neighborhood { streets,
lots, houses }`) lays a street grid (8 m streets, at least one
through-street plus cross streets, block size 76–96 m from the
seed), subdivides each block into lots along its streets, and puts
one house per lot from the existing generator — no second house
generator. Each house faces its street (rotation 0/90/180/270),
with setbacks front 5 m, back 3 m, sides 2 m.

Lot sizes are chosen so any generated house fits with setbacks:
the maximum house extent is 26 × 16 m (Ranch + garage wide,
Townhouse deep), proven over 200 seeds × 6 styles in
`domain::neighborhood`. North/south lots are ≥ 38 × 24 m; east/west
lots are 24 m deep and get only Cottage/Townhouse (the Craftsman,
up to 24 m wide with a garage, breaks the side setback there).
Lot size biases the style: small lots get Cottage/Townhouse, wide
lots get Ranch/Colonial/Modern/Craftsman.

Invariants, tested over 100 seeds: every house inside its lot
(with setbacks), no two houses overlap, every lot has street
frontage, every lot has exactly one house. Variety: 100 seeds give
100 distinct neighborhood hashes (FNV-1a over the seed-free
neighborhood JSON, which carries each house's plan). The default
is 2 × 2 blocks, ~16–24 houses, so the first render is readable;
`--blocks WxH` (1..10) grows it.

### Adapters and CLI

- `floorplan generate --seed <u32> [--style <name>] --obj <path.obj> --gltf <path.gltf>`
  (both optional; `--obj`/`--gltf` with `--count` is an error).
- `floorplan neighborhood --seed <u32> [--blocks WxH] [--out <path.svg>] [--json <path.json>]`
  (default blocks 2×2, default SVG `neighborhood-<seed>.svg`).
- Neighborhood SVG: streets as bands, lots as outlines, each
  house's footprint drawn in place (footprint rectangles + garage,
  rotated to face its street), labelled with style + seed.
  Neighborhood JSON: streets, lots, houses (lot, rotation, placed
  rect, style, seed, plan), fixed field order, two-decimal floats.

## Consequences

- The mesh is the single source of truth for 3D; adding a third
  writer (e.g. a binary glTF) means writing one more adapter, not
  another extrusion.
- The caps add rooms/splits deterministically and do not change
  the variety-count doc's numbers (program assignment is not
  counted there); a note to that effect is in the variety doc.
- The neighborhood's house seeds come from the neighborhood RNG
  (`rng.next_u32()`), so they are large u32s; the SVG labels show
  them in full, which is noisy but honest.
- No real 3D viewer exists on this machine (no blender, meshlab,
  assimp, or three.js); the mesh was verified by parsing the OBJ
  back and by top-down / isometric projection renders, not by a
  live 3D view.
