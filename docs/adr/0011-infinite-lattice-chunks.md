# 0011 — The unbounded lattice and chunk-as-window (protocol v4)
Status: accepted · Date: 2026-11-15
Context: chunks were a fixed grid of blocks keyed by `(seed, cx, cz)`
  with `DEFAULT_CHUNK_BLOCKS`, so the map had a finite size and chunk
  (0,0) was defined as the fixed 2x2 neighborhood. The next milestone
  is an infinite street-and-layout generator: walk in any direction
  and new streets keep coming, with more than a million distinct
  layouts. That needs a world model that is addressed, never
  allocated, and two neighbouring chunks must agree on the street
  they share without any global state or visit order.
Decision: the world is a lattice of nodes at every `(i32, i32)`
  multiple of `LATTICE = 140 m`, each jittered by a seeded offset of
  `-30..=30 m` on a 1 m grid. East and south edges exist unless the
  seed drops them (`EDGE_PERCENT = 92`), so the network is a planar
  grid with holes: a hole turns a crossroads into a T-junction and a
  curve into a cul-de-sac. Every value is a pure function of
  `(seed, node key)`, so a chunk rebuilt alone computes exactly the
  same streets as the chunk next to it. A chunk is one lattice cell
  (`CHUNK_SPAN = LATTICE`); it keeps the lots whose front centre
  falls in its half-open rectangle, which partitions the world's
  lots exactly once. A lot is laid from its own street's geometry
  only, and `lay_streets` resolves cross-street corner conflicts by
  street key, so the decision is chunk-independent too. Chunk (0,0)
  keeps the world seed for continuity of the JSON's `seed` field,
  but it is no longer equal to the finite generator's 2x2
  neighborhood — the finite generator remains for the CLI's
  `neighborhood` command.
Alternatives: a growing graph stored in a `HashMap` (rejected: it
  needs global state and a visit order, so two chunks could
  disagree); a fixed torus (rejected: no infinity); per-chunk
  procedural streets with no shared keys (rejected: streets would
  not line up at the border); reusing the finite spine-and-branches
  network per chunk (rejected: borders would not connect).
Consequences: `tests/wasm.rs` no longer asserts chunk (0,0) equals
  the fixed neighborhood; it asserts the chunk is the extrusion of
  its own content, which is the property that matters now. The
  Node harness drops the same two equality checks and instead
  checks every chunk has streets and lots. `PROTOCOL_VERSION` goes
  3 → 4 because the neighborhood document gains `lamps` and
  `signs`. The variety proof moves from layout families to the
  lattice's node jitter and edge holes.
Revisit if: streaming needs chunks smaller than one lattice cell,
  or a street must be re-routed after houses are placed.
