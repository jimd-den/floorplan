# 0015 — Footprint/BSP redesign: no dead leaves, core rooms first (protocol v5)
Status: accepted · Date: 2026-11-16
Context: ADR 0014 made the room-size rules hold on the five
  test plans (4 of 5 pass), but the 200-seed sweep still fails
  on 20 seeds — the two red tests stop at the first failure, so
  the sweep hid the other 19. Probed and classified (see
  `docs/reports/footprint-bsp-redesign.md`):
  - Room-sized entries on seeds 45, 60, 80, 101, 186: the
    entry leaf is a strip remainder (2.4 x ~4.3 m) or a whole
    wing (3.6 x 6.32 m), and the re-tiling loop is exhausted —
    no rect admits another alcove strip.
  - Missing core rooms on seeds 28, 58, 76, 77, 80, 100, 104,
    105, 106, 142, 143, 149, 152, 167, 178, 191: alcove strips
    consume the floor (each strip yields 3 leaves, none
    habitable-capable) while the rest is starved or sliced into
    dead pieces; and Dining/Study/Laundry take the last
    habitable leaf before the bed/bath rule runs.
  - Seed 109 Colonial: a Study at 4.9 x 2.4 m (2.04:1) — the
    cap strip cut keeps a 2.4 m strip whatever the leaf width.
  Root cause, one sentence: the subdivision emits leaves that
  can hold no real room, and the assignment spends the good
  leaves on optional programs before securing the core rooms.
  A leaf in (6.0, 6.5) sq m is dead: too big for a Bath (cap
  6.0), too small for a habitable room (floor 6.5).
Decision: seven changes, all inside the generator, no
  protocol/WASM/shell change.
  (1) `cap_cut`: the kept strip's depth is
  `max(MIN_SIDE, short_side / 2.0)` so the kept piece keeps
  aspect <= 2.0; `is_cuttable` requires the kept piece to fit
  the cap and the remainder to keep MIN_SIDE. A leaf no strip
  cut can fix legally is uncuttable, so the assignment's
  fitting-leaf rule sends the Study elsewhere.
  (2) `bsp::cut_range`: a cut is invalid when either child
  lands in the dead zone (6.0, 6.5) sq m. The range minus the
  two dead-zone holes returns its longest surviving interval,
  so every BSP leaf is bath-takable (<= 6.0) or
  habitable-capable (>= 6.5).
  (3) `alcove`: the rest target is clamped to
  `[max(1, min(3, cap)), max(1, cap)]` where
  `cap = floor(rest.area / 6.5)` — at least the three core
  rooms, at most one habitable leaf per 6.5 sq m, so the rest
  is neither starved nor sliced into dead pieces.
  (4) `retile`: when the floor already has a fitting Bath, a
  room-sized Hall/Entry outranks an over-cap Bath for the next
  strip (the over-cap Bath falls back to demotion to Laundry,
  and the floor keeps its fitting Bath). Without a fitting
  Bath, Baths keep priority — a house must keep a bath.
  (5) `footprint`: wing widths (L, U, T) are drawn from 4.8 m,
  not 3.0 m — a wing admits an alcove strip, so an Entry pin
  always has a rect. A 3.6 m wing can neither hold a strip nor
  a foyer entry (3.6 x 2.4 = 8.64 sq m is room-sized).
  (6) Assignment reserves: Dining, Study and Laundry (ground)
  and Study (upper) only take a habitable-windowed leaf when
  another one remains for the Bedroom; the Kitchen prefers a
  habitable-capable leaf, falling back to the nearest
  habitable leaf when no adjacent leaf is habitable (a
  sub-floor neighbour would choke it to Laundry).
  (7) `entry_shrink`: a room-sized Entry sharing its full
  north edge with a neighbour that can absorb the depth
  (stays within aspect and its program cap) is shrunk to a
  2.4 m foyer from the front, the shared split moving with
  the edge so the skeleton stays coherent. Runs in
  `build_floor` after the habitable-floor choke, before the
  skeleton. No-op when no neighbour can absorb.
  (8) `bsp::cut`: when more leaves are still needed, the cut
  position is chosen among positions that leave at least one
  child able to split again. A cut that leaves two dead-end
  children wastes the split — seed 28's 3.9 x 7.5 m rest cut
  at 3.75 leaves two 3.9 x 3.75 leaves, neither splittable,
  so (3) alone asks for three leaves and the rest stalls at
  two. When no such position exists the cut stays uniform-
  random over the valid range, so the subdivision keeps its
  variety.
  (9) Study assignment: when no leaf fits the Study (every
  habitable leaf is over its 12 sq m cap and uncuttable), the
  Study is skipped — not assigned to the farthest leaf. The
  farthest-leaf fallback bypassed the bed/bath rule's
  unstudyable protection, which promotes an over-cap,
  uncuttable leaf to a Bedroom (or Laundry when the bath
  guard holds it). Found by the generation cap test: (1)
  made a 4.9 x 6.9 m leaf uncuttable (kept piece 12.005 sq
  m), emptying the fitting set on seed 97 Colonial upper,
  and the fallback put the Study on a 5.9 x 6.9 m (40.7 sq
  m) leaf no strip cut can fix. A house needs no Study; an
  over-cap one is a failure. Applies to ground and upper.
  (10) `entry_shrink`: an over-cap Bath neighbour absorbs
  when a fitting Bath exists elsewhere — the re-tiling
  loop's exit step demotes it to Laundry, so it is
  effectively uncapped. Without a fitting Bath it is kept
  and cannot absorb. Found by the room-size sweep: (9)
  changed seed 45 Modern's re-tiling so the entry's
  northern neighbour is a 10.46 sq m Bath in the final
  round, and (7) runs before the demotion, so it saw a
  Bath where a Laundry will be.
  (11) `front_leaf`: among the southernmost leaves, a leaf
  that is not room-sized wins over a room-sized one — the
  centre-x rule only breaks ties within each class. An
  entry must never be a room when a foyer leaf sits at the
  front. Found by the room-size sweep: (10) left seed 77
  Townhouse with no Bedroom — the re-tiling loop pinned a
  second strip for a 4.8 m room-sized Entry and the 2.4 m
  rest it left was sliced into 6.0 sq m pieces by the BSP
  post-pass (a 2.4 m rest cannot be cut into habitable
  leaves: any horizontal cut violates aspect). With (11)
  the entry takes the bath strip's southern leaf and the
  loop exits after one strip, the rest intact.
  (12) `footprint`: wing/stem depths avoid (5.208, 7.3) m.
  A strip rest is the depth minus the 2.5 m bath leaf, and
  it is room-sized above 6.5 / 2.4 = 2.708 m and
  unsplittable below 2 x 2.4 = 4.8 m, so a depth in that
  range leaves a room-sized strip rest no repair can fix
  (seed 116 Craftsman T, stem 5.58 m, rest 3.08 m). The
  depth is moved to the foyer-sized side (5.2 m) when that
  stays in the draw range, else to the splittable side
  (7.3 m). Applies to L wing, U wings and T stem depths.
Alternatives: widen the dead zone to (6.0, 6.5] and forbid
  6.0 leaves from BSP cuts (rejected: the alcove bath leaf is
  exactly 6.0 and the Entry needs foyer leaves at 6.0);
  move the bed/bath rule before Dining/Study/Laundry
  (rejected: it would take two-thirds of the remaining leaves
  as bedrooms and starve Dining/Study everywhere — the
  reserve is surgical, it only fires when habitable leaves
  are scarce); make the entry a carved corridor instead of a
  leaf (rejected: same as ADR 0014 — a redesign, not a rule);
  let the entry take any foyer leaf anywhere, not just the
  front (rejected: the entry must be at the front; a side
  entry is a different house).
Consequences: `PROTOCOL_VERSION` stays 5 — only room rects
  and programs change. Every generated house changes (the BSP
  cut ranges move, the wing floor moves, the assignment
  reserves fire), so the five test plans are regenerated as
  v4 and compared before/after. The mesh budget may move
  (more/fewer leaves); it is adjusted only with a documented
  cause. Seeds 60 and 80 are the stress case for (4)+(5):
  if either still fails after these changes, the evidence is
  in the report and the footprint rule, not the entry rule,
  is where the next fix belongs.
Revisit if: a style genuinely needs a room above 2.0:1, or
  the wing floor of 4.8 m proves too wide for the variety
  target (the thousand-seed distinctness test is the tripwire).
