# ADR 0001 — The generator model

Status: accepted.

## Context

The generator must produce millions of distinct houses from a single
u32 seed, deterministically (same seed = same house, byte-identical
SVG and JSON), with every room reachable from the entry and every
habitable room daylit. The code is Clean Architecture: the plan model
is pure data, the generator is pure functions, and all I/O lives in
adapters.

## Decision

The generator is a pipeline of small pure functions, each independently
testable, composed in `generate_house`:

```
footprint -> subdivide (BSP) -> assign_programs -> place_doors
          -> place_windows -> House
```

- **Domain** (`src/domain/`): `House`, `Floor`, `Room`, `Opening`,
  `Footprint`, `Style`. Pure data, no I/O. A `Room` is a program on a
  rectangle; an `Opening` is a door or window centred on a wall.
- **Use cases** (`src/use_cases/`): the pipeline stages. `footprint`
  picks a shape and dimensions from the style's ranges; `bsp`
  subdivides each footprint rectangle into rooms with a 2.4 m minimum
  side so there are no sliver rooms; `programs` assigns programs by
  rule (largest leaf near the entry is living, kitchen next to living,
  bedrooms farthest from living, baths small and next to bedrooms);
  `doors` puts one door on every BSP split and every footprint join,
  which makes connectivity structural rather than hoped for; `windows`
  puts one window on an exterior stretch of every habitable room.
- **Adapters** (`src/adapters/`): the SVG renderer, the JSON writer
  and the CLI. The JSON writer is hand-rolled with fixed field order
  and two-decimal floats so the output is byte-deterministic.
- **Drivers** (`src/frameworks_drivers/`): a splitmix64 RNG on `std`
  only. The seed is the only input; there is no other source of
  variation.

## The two robustness decisions

Two failures forced their way into the design during verification:

1. **Windowless rooms.** A habitable room can end up with no exterior
   wall — a middle column of a three-column split is enclosed on all
   four sides, and the garage can cover a room's only exterior edge.
   The assignment is therefore daylight-aware: dining, study and
   bedrooms prefer leaves that can hold a window, while the hall and
   laundry prefer leaves that cannot. What the assignment cannot
   place, the repair fixes: a habitable program on a windowless leaf
   swaps with a bath, hall or laundry on a leaf that can hold a
   window — once at assignment time, once after the garage is placed.
   The window rule is the hard rule; kitchen adjacency can lose to
   it in the rare case where both cannot hold.

2. **The garage slide.** The garage attaches to the east wall and its
   6 m span can orphan a room's only window. After the rooms are
   known, the garage slides along the east wall on the 0.1 m grid to
   the spot that leaves the fewest habitable rooms windowless; the
   footprint's own position wins ties, so the garage only moves when
   it helps. Its join (and therefore its door) moves with it.

## Consequences

- Program assignment is a deterministic function of the leaves. It
  contributes no variety of its own; the variety lives in the leaves
  (see `docs/reports/variety-count.md`).
- The seed space (2^32) is the cap on distinct outputs, not the
  decision space — the decision space is larger (see the variety
  report). The empirical check is 1000 seeds giving 1000 distinct
  plan hashes, where the hash is taken over the seed-free plan JSON
  so that distinct hashes mean distinct plans.
- Room ids are global across the house so the stair pair
  (ground hall, upper hall) is unambiguous; opening ids are
  per-floor.
