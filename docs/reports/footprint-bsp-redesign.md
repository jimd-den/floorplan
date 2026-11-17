# Footprint / BSP redesign — build report

Branch context: `feat/functional-houses`. ADR 0015
(`docs/adr/0015-footprint-bsp-redesign.md`). No GitHub push, no
protocol/WASM change, no shell change. Tree:
`~/tmp/floorplan-functional/tree`.

## Gate — GREEN

- `make check`: **exit 0, 549 passed, 0 failed** (49 test
  binaries, all ok). Log: `~/tmp/make_check_v4f.log`.
- `cargo clippy --all-targets -- -D warnings`: **exit 0**.
- `cargo check --all-targets`: **exit 0**.
- Line check (`find src tests -name '*.rs' ... | awk '$1 > 200'`):
  **clean** — no .rs file over 200 lines (bsp.rs and ground.rs
  are exactly 200; entry_shrink.rs is 198).
- No `tests/zz_*.rs` probe files remain. (One was found after
  the first green run — `tests/zz_seed116.rs` — deleted, and
  the gate re-run clean: 550 → 549 tests.)

The 2 formerly-red tests in `tests/room_size.rs` both pass:
`a_seed_sweep_passes_the_room_size_rules_and_keeps_the_core_rooms`
and `the_five_test_plans_pass_the_room_size_rules`.
`tests/generation.rs::no_room_exceeds_its_program_cap_for_200_seeds`
also passes. No test was weakened.

## What fixed each deadlock

Root cause (one sentence): the subdivision emitted leaves that
could hold no real room, and the assignment spent the good
leaves on optional programs before securing the core rooms.

- **F1 — cap_cut aspect-aware strip depth.** A 4.9 x 5.9 m Study
  leaf was cuttable, but any MIN_SIDE strip kept > 12 m², so
  the strip cut could never fix it. `strip_depth` is now
  `max(MIN_SIDE, short_side / 2.0)`; `is_cuttable` requires the
  kept piece to fit the cap AND the remainder to keep MIN_SIDE.
- **F2 — BSP cut_range dead-zone exclusion.** A cut is invalid
  when either child lands in (6.0, 6.5) m² — too big for a Bath
  (cap 6.0), too small for a habitable room (floor 6.5).
  `cut_range` returns the longest surviving interval. The
  `split_long_rect` score is now `(bath, habitable, foyer)`.
- **F3 + F8 — alcove rest_target clamp, and BSP cuts that keep a
  splittable child.** `rest_target` is clamped to at most one
  habitable leaf per 6.5 m² and at least 3 leaves when strips
  are present. F3 alone stayed red on the seed 28 probe (the
  BSP stalled at 2 leaves); F8 makes `cut` prefer positions
  that leave at least one child able to split again.
- **F4 — retile priority: room-sized Hall/Entry before a second
  Bath when a fitting Bath already exists.** The over-cap Bath
  falls back to `demote_over_cap_baths` → Laundry.
- **F5 — footprint wing width floor 3.0 → 4.8 m.** Every wing
  admits at least one alcove strip (2.4 + 2.4 m).
- **F6 — assignment reserves (core before optional).**
  Dining/Study/Laundry only take a habitable-capable leaf when
  another remains for the Bedroom; Kitchen prefers a
  habitable-capable leaf. New module `core_rooms.rs`.
- **F7 + F10 — entry-shrink repair.** A room-sized Entry sharing
  its full north edge with a neighbour that can absorb the
  depth is shrunk to a 2.4 m foyer by moving the shared split.
  F10: an over-cap Bath counts as absorbent when a fitting Bath
  exists elsewhere (it will be demoted to Laundry).
- **F9 — skip the Study when no leaf fits it.** The farthest-leaf
  fallback assigned a 40.7 m² Study on seed 97 Colonial upper;
  `assign_bed_bath` then promotes the big uncuttable leaves to
  Bedroom.
- **F11 — front_leaf prefers a non-room-sized southernmost leaf.**
  Fixed seed 77 Townhouse (no Bedroom): the Entry took the bath
  strip's southern leaf instead of the wide rest leaf, so no
  second strip was pinned.
- **F12 — footprint wing/stem depths avoid (5.208, 7.3) m.**
  Fixed seed 116 Craftsman T (a regression from the redesign:
  a 5.58 m stem leaves a 2.4 x 3.08 m strip rest — room-sized
  and unsplittable). Depths are moved to the foyer-sized side
  (5.2 m) or the splittable side (7.3 m). The test uses a 1e-9
  epsilon on the boundary (floating-point makes exact 7.3
  fragile: 7.299999… is still in the bad range).

Whack-a-mole order during the sweep: 97 → 45 → 77 → 116.
Each fix exposed the next seed; the sweep stops at the first
failure, so seeds after the current blocker were untested
until the gate went green.

## Known failures — before / after

| Failure | Before (v3) | After (v4) |
|---|---|---|
| Colonial seed 7 hall | 68.4 m² (v2) → 6.0 m² foyer (v3) | 6.0 m² foyer (unchanged) |
| Colonial seed 7 entry | 27.0 m² (v2) → 6.0 m² foyer (v3) | 6.0 m² foyer (unchanged) |
| Modern seed 202 entry | 31.2 m² (v2) → 6.0 m² foyer (v3) | 6.0 m² foyer (unchanged) |
| Craftsman seed 101 entry | 2.6 x 3.4 = 8.84 m² room-sized (v3, FAIL) | 2.4 x 2.5 = 6.0 m² foyer (PASS) |
| Seed 28 Cottage bedroom | No Bedroom (v3, FAIL) | 2 bedrooms, 12.0 and 9.5 m² (PASS) |

## Fragmentation — v3 → v4 (rooms excl. garage; ~6.0 m² = within 0.05)

| Plan | Rooms | ~6.0 m² | Worst aspect | Largest bedroom | Largest living |
|---|---|---|---|---|---|
| 01 Ranch 42 | 10 → **9** | 2 → 2 | 1.97 → **1.89** | 12.5 → **14.4** | 17.7 → **24.5** |
| 02 Colonial 7 | 35 → 35 | 20 → 20 | 1.97 → 1.97 | 44.8 → 44.8 | 25.6 → 25.6 |
| 03 Craftsman 101 | 9 → 11 | 1 → 2 | 1.95 → **1.62** | 15.0 → 11.8 | 16.8 → **22.2** |
| 04 Modern 202 | 12 → **8** | 3 → 2 | 1.96 → 2.00 | 11.3 → **21.1** | 30.2 → **36.0** |
| 05 Cottage 303 | 7 → 8 | 2 → 2 | 1.96 → 1.96 | 11.2 → 8.1 | 16.5 → 17.2 |

All five v4 plans were verified directly from their JSONs
against the room-size rules (aspect ≤ 2.0, habitable floor
≥ 6.5 m², Bath ≤ 6.0, Study ≤ 12.0, no room-sized Entry/Hall,
Living + Kitchen on the ground floor, a Bedroom somewhere):
**5/5 pass.**

Reading the fragmentation: Ranch and Modern rebalanced
cleanly (fewer rooms, much larger bedrooms/living). Craftsman
grew one room to fix the Entry, but its worst aspect improved
and its living grew. Colonial is **unchanged** — the fixes did
not move seed 7 at all, so its 20 tiny rooms remain (see
Residuals). Cottage is mixed (one more room, bedroom smaller).

## v4 file list (~/workspace/floorplan-test-plans/)

- `01-v4.svg`, `01-v4.json`, `01-v4-white.svg` — Ranch seed 42
- `02-v4.svg`, `02-v4.json`, `02-v4-white.svg` — Colonial seed 7
- `03-v4.svg`, `03-v4.json`, `03-v4-white.svg` — Craftsman seed 101
- `04-v4.svg`, `04-v4.json`, `04-v4-white.svg` — Modern seed 202
- `05-v4.svg`, `05-v4.json`, `05-v4-white.svg` — Cottage seed 303

Originals (`*-v3`, `*-v2`, unversioned) kept. White copies have
a white rect covering the viewBox as the first element.

## Residuals

- **Colonial seed 7 fragmentation unchanged.** 35 rooms, 20 of
  them exactly 2.4 x 2.5 = 6.0 m². The redesign's fixes never
  trigger on this house's geometry, so the tiny-room count did
  not move. Its large rooms are already large (bedroom 44.8,
  living 25.6 m²), so the house reads fine — but the rebalance
  did not reduce its service-room count. A further rebalance
  (e.g. merging adjacent tiny service leaves) would be a new
  change, not a fix for a red test.
- **Modern seed 202 worst aspect is exactly 2.00** — at the hard
  cap, not over. Within the rule, but with no margin.
- **Cottage seed 303 largest bedroom shrank** 11.2 → 8.1 m².
  Still above the 6.5 m² floor, but the smallest largest-bedroom
  of the five.
- Seeds 60 and 80 (room-sized Entry, assessed pin-only via
  F4+F5) pass the sweep — no genuine deadlocks remain in the
  200-seed sweep.

## Test-change justifications carried forward (§2.7)

- `tests/alcove.rs` pin-index change: asserts leaf geometry
  instead of hard-coded indices.
- `retile.rs` capped-room change: Study → Bath (the retile
  target is a capped room; Bath is the capped program that
  can be pinned).
- `tests/bsp_aspect.rs` residual test: superseded by ADR 0014.
- Deleted duplicate unit test in `bsp.rs`.

## New splits made this session

- `src/use_cases/split_pieces.rs` (54 lines): `record_piece_splits`
  + `cut_into`, moved out of `bsp.rs` (now exactly 200 lines).
- `tests/retile.rs`: the whole `#[cfg(test)]` module moved out of
  `retile.rs` (now 100 lines).
- `tests/ground.rs`: red-first integration tests for F6/F11.
- `tests/floor.rs`: the `#[cfg(test)]` module moved out of
  `floor.rs` (was exactly 200 lines, now 151) before adding
  the F7 call.
- `src/use_cases/core_rooms.rs`: F6 helpers
  (`bedroom_reserve_holds`, `pick_kitchen`).
- `src/use_cases/entry_shrink.rs`: F7/F10 (198 lines).

## TDD note

One minor slip to record: the F6 `core_rooms` helpers and their
unit tests were written together with the already-red
integration tests as the red evidence, rather than the unit
tests being red first on their own. The behaviour itself was
red-first (both integration tests failed before wiring); the
unit tests were not independently watched red.
