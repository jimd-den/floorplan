# ADR 0004: Random neighborhood layouts and the WASM adapter

## Status

Accepted. Implements the spec's streaming foundation (a finite
neighborhood, fully realized) and the thin WASM adapter the engine
architecture calls for. This supersedes the earlier plan for a 3D
neighborhood mesh export (OBJ/glTF writers, `--obj`/`--gltf` on
the neighborhood CLI): Jim corrected the scope on 2026-11-12 —
"we just want it to render good first person on wasm" — so there
is no neighborhood mesh export, and the mesh exists only as the
in-memory data the WASM adapter hands to the web shell.

## Context

The neighborhood generator built one street grid (8 m streets,
blocks 76–96 m from the seed) with lots and furnished, re-tiled
houses, and adapters for SVG and JSON only. The engine needs two
things before the web shell can render it first-person:

1. Random layouts: the street/block pattern must vary per seed,
   not just the block sizes, while keeping the invariants (every
   lot street-fronted, exactly one house per lot, no overlaps,
   every house inside its lot with setbacks, every house
   enterable).
2. A WASM adapter: the std-only core compiled to
   `wasm32-unknown-unknown`, exposing house and neighborhood
   generation through raw `extern "C"` exports, with a Node
   harness proving the WASM output matches the native CLI output
   for the same seeds.

## Decision

### Layout families (`domain/layout.rs`, `use_cases/layout.rs`, `use_cases/lots.rs`)

- `Layout`: Grid, Offset, CulDeSac. `Layout::from_seed(seed)` is
  `seed % 3`, so each family covers a third of the seed space and
  the family is visible in the seed.
- The planner (`plan_layout`) returns the street network and, per
  block, the adjacent street indices, so the lot layer lays houses
  without knowing which family built the network. The generator
  (`generate_neighborhood_layout`) is the test entry point for
  verifying invariants per family; `generate_neighborhood` picks
  the family from the seed.
- Grid is the original builder, unchanged: the planner consumes
  no RNG draws for Grid, and the lot layer draws in the same
  order, so a Grid neighborhood is byte-identical to before
  (streets and lots for seed 42 match the old example exactly;
  the example JSON/SVG were regenerated only because the M0
  re-tiling had already changed the house plans, and because the
  JSON now names the layout).
- Offset staggers alternate block rows east by half a block:
  the horizontal streets stay full-width, the vertical streets
  become per-row segments, and the odd rows extend half a block
  further east. Blocks keep their four adjacent street indices.
- CulDeSac is a regular grid in which one or two blocks become
  courts: a stub street (8 m wide) enters the block from its
  south street and ends in a 28 m square bulb. The stub and bulb
  are streets in their own right, appended after the grid
  streets. `lay_court` derives its five lots from the stub and
  bulb rects — a north strip facing south onto the bulb, a lot on
  each side of the bulb, and a lot on each side of the stub — so
  the lots always fit around the streets. The flank lots get the
  small styles (Cottage/Townhouse), as the side lots do.
- `Neighborhood` carries `layout: Layout`, and the JSON writer
  names it (`"layout":"grid" | "offset" | "cul-de-sac"`) in both
  the seeded and the seed-free form, so the variety hash
  distinguishes families and the shell can read the family.
- Verified: the invariants (inside lot with setbacks, no
  overlaps, one house per lot, every lot street-fronted, every
  house facing its street) hold over 100 seeds **per family**,
  and each family gives 100 distinct layout hashes over 100
  seeds (`tests/neighborhood_layouts.rs`, `tests/layout.rs`).

### The neighborhood mesh (`use_cases/neighborhood_mesh.rs`)

- `neighborhood_mesh(n)` builds the whole neighborhood as one
  domain `Mesh`: every house extruded with furniture by the
  existing extruder, placed and rotated on its lot, plus one
  thin ground box per street and per lot. It reuses the house
  mesh builder — no duplicated extrusion logic.
- Placement follows `place_rect` exactly. Rotation 90 is a
  reflection across the diagonal (determinant −1) while 0/180/270
  are proper rotations, so only rotation 90 reverses triangle
  winding (`flips_winding`); the other rotations keep it. The
  placed bounds match the placed rect for every rotation.
- Ground: streets are boxes from y −0.05 to 0.0, lots from y
  −0.06 to −0.02, so no ground plane is coplanar with a house
  floor slab (which tops out at y 0). Materials: the house table
  (29) plus `street` (id 29, dark asphalt) and `lot` (id 30,
  grass) — 31 materials in all.
- Verified: the mesh counts equal the sum of the per-house
  extrude counts plus one ground box per street and lot, and
  `Mesh::validate()` passes for all three families.

### The WASM adapter (`adapters/wasm.rs`, `[lib] crate-type`)

- The crate builds as `cdylib` + `rlib`: the cdylib produces
  `target/wasm32-unknown-unknown/debug/floorplan.wasm` with the
  exports; the rlib keeps native tests linking as before.
- No wasm-bindgen, no crates beyond std. The payload builders
  are plain Rust and tested natively (`tests/wasm.rs`); only the
  `extern "C"` exports are wasm-only (`#[cfg(target_arch =
  "wasm32")]`). The module imports nothing.
- Built with `-C panic=abort`, so a bug traps instead of
  unwinding. Exports validate their arguments and return null
  rather than panicking (an invalid style index returns null
  with length 0).
- `#[unsafe(no_mangle)]` (the 2024-edition form) marks the
  exports; the wasm build is `RUSTFLAGS="-C panic=abort" cargo
  build --lib --target wasm32-unknown-unknown` (see `make wasm`).

### The export protocol (the web shell builds against this)

Protocol version 1 (`fp_version()` returns 1).

Memory: the caller owns every pointer an export returns and
frees it with `fp_free(ptr, size)` when done. `fp_alloc(size)`
allocates caller-owned memory. Every payload export returns a
pointer to its bytes and records the byte length in
`fp_last_len()` — read it immediately after the call, before
any other export call. A null pointer means the call was
refused (invalid argument); the length is then 0.

```
fp_version() -> u32                                    // 1
fp_alloc(size: usize) -> *mut u8
fp_free(ptr: *mut u8, size: usize)
fp_last_len() -> u32
fp_house_json(seed: u32, style: u32) -> *mut u8        // null if style > 5
fp_neighborhood_json(seed: u32, cols: u32, rows: u32) -> *mut u8
fp_house_mesh(seed: u32, style: u32) -> *mut u8        // null if style > 5
fp_neighborhood_mesh(seed: u32, cols: u32, rows: u32) -> *mut u8
```

Style indices are `Style::all()` order: 0 Ranch, 1 Colonial,
2 Craftsman, 3 Modern, 4 Cottage, 5 Townhouse.

JSON payloads are the same text the CLI writers emit
(`generate --json`, `neighborhood --json`): rooms, furniture,
openings, streets, lots, placed houses with rotation and seed —
everything the shell needs to build geometry and collision.

Mesh payloads are one binary blob, all little-endian:

```
magic "FPM1" (4 bytes)
u32 vert_count, u32 face_count, u32 material_count
f32[vert_count * 3] positions (x, y, z) — metres, x east, y up, z south
u32[face_count * 3] indices
u32[face_count] material ids
material table: per material, u8 name_len, name bytes, f32[3] colour
```

House meshes use the 29-material house table; neighborhood
meshes use the 31-material table (house table plus `street`
id 29 and `lot` id 30).

### The Node harness (`tools/wasm_check.mjs`)

`node tools/wasm_check.mjs [--wasm <path>] [--bin <path>]`
loads the built .wasm, calls the exports for seeds 0, 7, 42,
106, 199, and checks the WASM output against the native CLI
for the same seeds: house JSON byte-identical for all six
styles; house mesh counts and material names matching the
native OBJ/MTL; neighborhood JSON byte-identical for 2×2 and
3×3 blocks; neighborhood mesh internally valid (indices and
material ids in range) with counts equal to the native
per-house OBJ counts plus one ground box per street and lot
(2×2); every payload byte-identical across two calls; an
invalid style index refused with a null pointer. **364 checks,
0 failures.** `make wasm` builds both binaries and runs it.

## Consequences

- The web shell can now render a whole neighborhood first-person
  from WASM: one `fp_neighborhood_json` call for plans,
  furniture, and collision, and one `fp_neighborhood_mesh` call
  for the renderable mesh. Seed 42 (2×2, Grid): 22 houses, 6
  streets, 22 lots; neighborhood mesh 39,888 verts / 59,832
  faces / 31 materials (1,436,551 bytes). The Modern seed 42
  house mesh is 1,400 verts / 2,100 faces / 29 materials, as
  in M0+M1. The .wasm is 5,879,409 bytes (debug build).
- The layout variety is now structural: three families over the
  seed space, each with 100 distinct layouts over 100 seeds.
  Grid output is unchanged, so existing Grid examples and tests
  still hold; the neighborhood example JSON/SVG were
  regenerated for seed 42 (the JSON gained the `layout` field).
- `make check` is green (220 tests, clippy clean) and the
  WASM build is clean on wasm32-unknown-unknown.
- The mesh export stays dropped: there are no neighborhood
  OBJ/glTF writers, no `--obj`/`--gltf` on the neighborhood CLI,
  and no mesh examples. The next step is the three.js web shell
  (not built here); streaming (spec M4) is still a later phase.

## Alternatives considered

- wasm-bindgen: rejected — the crate is std-only with zero
  crates, and the protocol is four functions and a pointer/length
  pair; a binding generator would add a dependency and a build
  step for nothing.
- JSON for the mesh: rejected — the binary form is ~10× smaller
  than the same mesh as JSON text and parses with a DataView in
  a dozen lines; the shell needs numbers, not text.
- Packing (pointer, length) into one u64 return: rejected — it
  needs BigInt on the JS side; `fp_last_len()` is one more call
  and keeps every return a plain i32.
- A static output buffer inside the module: rejected — the
  caller-owned allocation (`fp_alloc`/`fp_free`) lets the shell
  hold several payloads at once without the next call clobbering
  the last.
- Keeping the neighborhood mesh export (OBJ/glTF): dropped by
  Jim's correction — the shell renders from WASM, so the mesh
  never needs to leave the module as a file.
