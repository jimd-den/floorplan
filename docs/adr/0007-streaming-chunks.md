# ADR 0007: Streaming chunks and protocol v2

## Status

Accepted. Implements SRD-001 §4.1 (FR-1 – FR-9) and the M4 exit
criteria. Extends ADR 0004's WASM boundary without changing any v1
call.

## Context

The neighborhood generator built one finite street network from a
world seed. The engine needs an unbounded neighborhood that is
generated in pieces around the player, installed without a hitch,
and released behind them — while a revisit is byte-identical and the
fixed 2x2 neighborhood stays exactly what the CLI already emits.

The core is std-only and compiles to `wasm32-unknown-unknown` with
no threads. "Asynchronous" therefore cannot mean spawning workers in
the core; it means the core exposes a lifecycle the shell advances
one frame at a time, with generation and installation metered.

## Decision

### Granularity

One chunk is one **2x2-block neighborhood** (`DEFAULT_CHUNK_BLOCKS`).
This is the unit the existing invariant tests already cover, and it
makes FR-8 exact: chunk (0,0) is `generate_neighborhood(world,
(2, 2))` with no translation.

### Derivation

`chunk_seed(world, cx, cz)` returns the world seed for (0,0) and an
FNV-1a mix of the seed and the two signed coordinates otherwise.
Every chunk is generated from that seed alone, so visit order,
wall-clock time, and player state cannot affect content (FR-1).

Chunk origins are spaced `CHUNK_SPAN = 400 m` apart. The largest
2x2 neighborhood is well under that (proven over 100 seeds in
`tests/chunk.rs`), so chunks never touch. A non-origin chunk
translates its street, lot, and placed-house rectangles by the
origin; the house plans themselves stay local and are placed by the
existing mesh builder.

### Lifecycle and metering

`use_cases::stream::Streamer` is a pure state machine:

- `request` inserts a `Requested` chunk with no content.
- `generate_step` builds at most one chunk per tick
  (`GENERATION_PER_TICK = 1`) — the 8 ms/frame budget, not the
  50 ms chunk budget, is the binding constraint.
- `install_step` flips at most two `Ready` chunks to `Installed`
  per tick (`MAX_INSTALLS_PER_TICK = 2`, FR-3).
- `tick(player)` requests the 3x3 ring (`RING_RADIUS = 1`),
  generates, installs, then evicts chunks beyond `KEEP_RADIUS = 2`.

Eviction removes the chunk and its content from the map. Because
generation is a pure function of the seed and coordinates, a revisit
rebuilds the same bytes (FR-5, NFR-2). The working set is bounded at
25 chunks, so memory plateaus instead of growing with distance
(NFR-3).

### WASM protocol v2

`fp_version()` returns 2. The two new exports are
`fp_chunk_json(seed, cx, cz)` and `fp_chunk_mesh(seed, cx, cz)`;
signed chunk coordinates are `i32`. Every v1 export keeps its
signature and semantics (FR-9).

Chunk JSON is
`{"chunk":[cx,cz],"state":"ready","neighborhood":<v1 neighborhood JSON>}`.
The neighborhood document is reused verbatim, so chunk (0,0) embeds
the fixed neighborhood byte-for-byte. The chunk mesh is the existing
`FPM1` neighborhood mesh, so the payload format does not change.

## Alternatives

- **One street segment per chunk.** Rejected: (0,0) would not be the
  fixed 2x2 neighborhood without a special case, and the invariant
  tests are written per neighborhood.
- **Grow one global neighborhood by blocks.** Rejected: content
  would depend on visit order and memory would grow with distance.
- **Worker threads in the core.** Rejected: wasm32 without threads,
  std only, and the shell already owns the frame loop.
- **Mutate protocol v1 in place.** Rejected: v1 takes `(seed, cols,
  rows)`; adding coordinates changes the meaning of an existing
  call.

## Consequences

- Chunk (0,0) is byte-identical to the fixed neighborhood's JSON and
  mesh; the Node harness checks both.
- The shell drives streaming with one `tick` per frame; it never
  waits for a chunk. The next chunk in the walking direction is
  installed before arrival (FR-4, tested).
- A 10-minute soak is modelled, not wall-clock measured: the tests
  bound the working set, the per-tick installs, and the generated
  chunk count. Release-built timing remains a separate gate (AC-9).
- The harness grew from 364 to 529 checks, all passing.

## Revisit if

Chunk generation exceeds 8 ms/frame in a release build, or the shell
needs look-ahead beyond one chunk (a larger ring, or a directional
ring rather than a square one).
