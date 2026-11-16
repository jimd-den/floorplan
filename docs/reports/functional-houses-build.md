# Functional houses build — base first, skeleton, walls, doors by opening

Working tree: `/home/hatch/tmp/floorplan-functional/tree` (copy of
branch `feat/infinite-street-neighborhood`, head `914b0449`).
Ready for the parent to push to branch `feat/functional-houses`.
Do not push from here; the shell artifact is untouched.

## What changed

**Base first.** New `src/domain/base.rs`: `Base { rects, thickness }`,
`BASE_THICKNESS = 0.3`. `Base::from_footprint` takes the footprint
rects plus the garage (the porch keeps its own slab). `House.base`
is built before any room, and the base is in the JSON
(`"base":{"rects":[...],"thickness":0.30}`, with-seed JSON only —
it is a deterministic function of the footprint, so the plan JSON
and `plan_hash` are unchanged) and in the mesh (one concrete slab
box per base rect, y −0.3 to 0, every LOD).

**Logical skeleton.** New `src/domain/skeleton.rs` (`Skeleton`,
`SkeletonLink`, `is_connected`, `spine`, `backbone_links`,
`adjacency`) and `src/use_cases/skeleton.rs`
(`build_skeleton`: a link is backbone when its segment overlaps a
split line or a join). `Floor.skeleton` is built in `build_floor`
and passed to `place_doors`. The skeleton proves every room is
connected (BFS over all links; the spine is a tree from the entry).

**Walls then doors.** `place_doors` now takes the skeleton and
places one door per backbone link whose segment is ≥ 0.6 m, plus
the entry door and `circulation::extras`. `Adjacency` is deleted,
consolidated into `SkeletonLink`.

**Doors by opening size.** New `src/use_cases/door_width.rs`.
The old 0.6–0.9 clamp and the fixed 1.0 m entry door are gone.
Rule (documented in code and ADR 0012):

| opening | door |
|---|---|
| ≥ 3.0 m | 1.6 m double (two 0.8 m leaves) |
| ≥ 2.0 m | 1.0 m wide single |
| ≥ 1.5 m | 0.9 m standard |
| else | 0.8 m narrow single |

Clamped to `[0.6, opening − 0.6]`, quantized to 0.1 m. The
double-door threshold is a 3.0 m opening (1.6 m door).
`door_panel.rs` splits a door at or above 1.4 m into two panel
leaves, each hinged at its outer end, closed and open; a 0.9 m
door is still one panel (existing tests unchanged, geometry
bit-identical for a single leaf).

**Rooms traversable.** New `src/use_cases/traverse.rs`
(`reachable_rooms`, `is_traversable`) and
`src/use_cases/wall_block.rs` (`Wall`, `walls`, `gaps_on`,
`blocked`): a 0.1 m grid BFS over floor space; shared walls
block within half the wall thickness except inside door-gap
spans; windows are not gaps (their sills block). Proof:
`tests/traversable.rs`
(`every_room_is_walkable_from_the_entry_for_100_seeds`) — 100
seeds × all 6 styles, every room walkable from the entry
(ground) or the stair landing (upper).

**Garage fix (found by the traversability proof).** Seed 6
Craftsman put the garage at gy 5.7 where its west edge touched
no room, so it had no door and was unreachable — a pre-existing
defect (the old code has the same garage position and no door).
`choose_garage_y` now only accepts spots where the garage shares
a wall with a room by at least a door's width (`attachable`,
≥ 0.6 m); the footprint gy is kept only when it is attachable
and orphans nothing, so the garage only moves when it helps.

**Furniture removed.** Deleted `src/use_cases/furnish.rs`,
`furniture_mesh.rs`, `furniture_parts.rs`, `clutter.rs` and
`tests/furnish.rs`, `tests/furnish_density.rs`,
`tests/clutter.rs`. The furnish loop is out of `generate.rs`.
`extrude` and the JSON carry no furniture. `furnish_clear.rs`
keeps `fits`/`blocks_door`/`door_zone`/`blocks_window` (used by
`interact::move_furniture`); `walkable`/`door_inner_point`/
`reachable` are deleted (superseded by `traverse`). The domain
furniture kinds and the `interact`/`haunt`/`light` machinery
stay and operate on empty lists; their tests use manual fixtures
(a lamp, a bed, a chair and a table placed by hand, never
generated). Home-fixing tasks degrade to restore_power and
close_doors (check_lights needs a lamp, catalog needs
furniture). The lamp-toggle mesh assertion in
`tests/wasm_sandbox.rs` is removed (no lamp in the mesh to
toggle).

**Material-id fix (found by the gate).** Appending `CONCRETE =
39` to the house table had shifted the neighborhood's appended
materials by one, but the constants were still literals:
street/lot/sidewalk pointed at concrete/street/lot, and the
lamp/sign ids pointed at sidewalk/lamp_post/lamp_glass/
sign_post — so lamp posts were rendered as sidewalk and the
ground-faces-up test failed. All of them are now derived from
`CONCRETE` (`STREET = CONCRETE+1`, `LOT = +2`, `SIDEWALK = +3`,
`LAMP = +4`, `LAMP_GLASS = +5`, `SIGN_POST = +6`,
`SIGN_FACE = +7`). Actual table: concrete 39, street 40,
lot 41, sidewalk 42, lamp_post 43, lamp_glass 44, sign_post 45,
sign_face 46.

**Protocol v5.** `PROTOCOL_VERSION` 4 → 5
(`src/adapters/wasm.rs`). What broke: the house JSON gains
`base` and loses the `furniture` array; the mesh gains concrete
foundation slabs and loses furniture boxes. The shell must
update its version check and stop expecting furniture.
`tools/wasm_check.mjs` updated (`fp_version is 5`; the
neighborhood mesh check renamed to "ground and street furniture
names"). `tests/wasm.rs` pins the version to 5.

**ADR.** `docs/adr/0012-functional-houses.md` — the new order
(base → skeleton → walls → doors) and the door-sizing rule.

**Line-ceiling splits.** `traverse.rs` (156) split off
`wall_block.rs` (140) by responsibility (wall blocking vs the
walk). `interact.rs` (112) moved its public-API unit tests to
`tests/interact.rs` (186). `tests/walls.rs` (181) gained a
`make_floor` helper for its six Floor literals.
`door_panel.rs` is 193 lines; `garage.rs` 145;
`furnish_clear.rs` 88.

## Red witnesses (pasted before each green run)

- `traverse` stub: `two_rooms_joined_by_a_door_are_both_reachable`
  and `a_chain_of_three_rooms_is_walkable_end_to_end` FAILED.
- `garage_is_placed_where_it_shares_a_wall` FAILED (garage at
  gy 5.7 shares no wall).
- `the_flat_ground_faces_up` FAILED ("ground face 0 faces down")
  and two `tests/grounds.rs` tests FAILED (material constants
  off by one).
- `a_double_door_has_two_panels` and
  `an_open_double_door_swings_both_leaves` FAILED.
- `house_interact_mesh_bytes_...` FAILED at the lamp-toggle
  assertion (no lamp in the mesh).

## Gate and harness

- `make check` exit 0: **461 passed, 0 failed**, clippy clean.
- `cargo check --all-targets` exit 0.
- Line check (`find src tests -name '*.rs' -exec wc -l {} + |
  awk '$1 > 200'`): no file over 200 lines.
- WASM release: `RUSTFLAGS="-C panic=abort" cargo build
  --release --lib --target wasm32-unknown-unknown`,
  366,546 bytes,
  sha256 `ed36f44e4b4ca73d2d411b42789662c2d8a6b262d160a0dc58443bfc9b2a464b`.
  `wasm/floorplan.wasm.b64` regenerated (495,159 bytes) and
  verified to decode back to the same sha.
- `node tools/wasm_check.mjs --wasm
  target/wasm32-unknown-unknown/release/floorplan.wasm --bin
  target/release/floorplan`: **726 passed, 0 failed**.

## Deferred

- Shell protocol v5 update (parent's job, not this build's):
  version check 4 → 5, stop expecting furniture in JSON/mesh.
- Seed widening to u64 (still Jim's call).
- M0+M1 residuals: the "~8% of small rooms skip a bed/tub"
  residual is now moot (no furniture); the 5 strict-reading
  studies (12.0–13.9 m²) are unchanged and still Jim's call.
- Pale-blue ground-gap rectangles, glass tint, roof
  single-sidedness: unchanged, not ordered.
- The `rooms_cover_the_footprint_exactly` test still excludes
  the garage from the room area; unchanged.
