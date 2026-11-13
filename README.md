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
  tested geometrically over 200 seeds.
- **Per floor**: a floor slab and a ceiling slab per room.
  **Stairs** for two-storey houses are solid stepped boxes in the
  ground hall. **Roof**: a flat slab cap — the top floor's ceiling
  and any exposed ground ceiling (e.g. over the garage). **A
  pitched roof is explicitly out of scope**; it is not built.
- **Materials**: one per room program (living, kitchen, dining,
  bedroom, bath, hall, entry, study, laundry, garage) plus
  wall/ceiling/roof/stairs/porch.

```sh
floorplan generate --seed 42 --style modern --obj house.obj --gltf house.gltf
# house.obj + house.mtl + house.gltf; both flags optional.
# --obj/--gltf cannot be combined with --count.
```

## Neighborhoods

A street grid on top of the same house generator — no second house
generator. Streets (8 m, at least one through-street plus cross
streets), lots along the streets with setbacks (front 5 m, back
3 m, sides 2 m), one house per lot, each facing its street. Lot
size biases the style: small lots get Cottage/Townhouse, wide lots
get Ranch/Colonial/Modern/Craftsman. Every house fits its lot (the
maximum house extent is 26 × 16 m); every lot has street frontage;
no two houses overlap. The default is 2 × 2 blocks, ~16–24 houses.

```sh
floorplan neighborhood --seed 42
# neighborhood-42.svg (streets, lots, footprints in place, style + seed labels)

floorplan neighborhood --seed 42 --blocks 3x3 --out nb.svg --json nb.json
```

## Room-size caps

Bath is capped at 6 sq m and study at 12 sq m. Over-cap leaves are
split (a 2.4 m strip, remainder Dining or Laundry) or reassigned;
bedrooms never take a sub-cap leaf while a larger one can hold one.
The honest limit: a strict ≤ 6 sq m bath cap is geometrically
impossible for most generated houses without re-tiling the leaves
— over 200 seeds, 30 of 310 over-cap baths and 32 of 37 over-cap
studies are fixed, the rest have no ≤ 6 sq m leaf to move to and
no strip-cut that fits. See ADR 0002.

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
