# 0014 — Room-size rules: IRC floor, aspect cap, hall/entry classification (protocol v5)
Status: accepted · Date: 2026-11-16
Context: the five test floorplans (Ranch 42, Colonial 7,
  Craftsman 101, Modern 202, Cottage 303) contain rooms no real
  house has. Measured against the room-size study (see
  `research_notes/residential-room-sizes-standards-20261117-0244/report.md`):
  Colonial floor 1 has a 2.9 x 12.8 m bedroom (4.41:1) and a
  2.4 x 10.30 m bedroom (4.29:1), a 7.2 x 9.5 m hall (68.4 sq m),
  and a 2.4 x 7.9 m dining room (3.29:1); Colonial floor 0 has a
  10.0 x 2.7 m entry (3.70:1, 27.0 sq m) and a 4.7 x 4.6 m hall
  (21.6 sq m); Modern has an 8.0 x 3.9 m entry (31.2 sq m);
  Ranch living (2.39:1) and dining (2.68:1) and Cottage living
  (3.29:1) are long thin strips; three Colonial studies sit at
  5.76-6.00 sq m, below the IRC 70 sq ft (6.5 sq m) habitable
  floor. The BSP splitter only guaranteed a 2.4 m minimum side,
  so it emitted full-depth strips; assignment defaulted leftover
  leaves to Study whatever their size; and Hall/Entry took
  whatever leaf the position rules picked, however big.
Decision: the study's numbers are encoded once, in a domain
  rules module (`domain::room_size`), and enforced at three
  points. (1) Habitable floor (IRC 2021 R304.1/R304.2): a
  habitable room needs area >= 6.5 sq m and a short side >=
  2.13 m; kitchens are exempt from the area/dimension rule.
  (2) Aspect: every room's length:width <= 2.0 (hard cap), with
  1.6 as the target band (bedrooms/dining 1.0-1.4, living
  1.3-1.6, galley kitchen up to 1.7); a furnished room's short
  side is >= 2.59 m where furniture sits on both long walls.
  (3) Hall/entry classification (study section 4, an inference,
  not code): circulation when the short side is <= 1.22 m or
  the area is < 3.7 sq m; foyer when the short side is >= 1.52 m
  and the area is >= 4.5 sq m; grand foyer at >= 9.3 sq m; and
  room-sized when the area is >= 6.5 sq m AND the short side is
  >= 2.13 m — a room-sized hall/entry must not be emitted, it
  is shrunk or re-split. The BSP splitter (`bsp`) only cuts
  where both children keep aspect <= 2.0 and sides >= 2.4 m, so
  a long thin leaf can never be produced; a leaf that cannot be
  cut that way is not split, and the subdivision stops early as
  it already does. Assignment (`programs`) never leaves a
  habitable program on a sub-floor leaf: leftovers under the
  floor become Laundry. A cap-split remainder under the floor
  becomes Laundry, never Dining (`caps`). A room-sized
  Hall/Entry is a re-tiling target (`retile`): the loop pins an
  alcove leaf (2.4 x 2.5 m, 6.0 sq m — under the room-sized
  threshold, aspect 1.04) as the Hall/Entry, and the big leaf
  takes another program from the normal rules. The entry door
  keeps its southern-wall placement for front entries; an
  alcove entry takes the longest exterior edge when its
  southern wall is interior.
Alternatives: enforce the rules only in a post-pass that
  re-splits offending rooms (rejected: a single straight cut
  cannot fix a 7.2 x 9.5 m hall or a 3.2 x 3.6 m entry wing —
  the remainder is a sliver — so the fix must happen where the
  leaves are made, not after); cap hall/entry area in `caps`
  (rejected: the strip cut keeps a 2.4 m strip, which is still
  room-sized for a hall, and cannot shrink a square-ish leaf);
  raise the BSP minimum side to 2.59 m (rejected: baths need
  2.4 m leaves, and the aspect cap already stops the strips);
  make the hall a carved corridor rather than a leaf (rejected:
  the hall is a BSP leaf in the skeleton, the doors, and the
  stairs, and a corridor carve is a redesign, not a rule).
Consequences: `PROTOCOL_VERSION` stays 5 — the JSON and mesh
  shapes do not change, only room rects and programs, so the
  shell needs no change and no new WASM. Every generated house
  changes (the BSP cut ranges move), so the five test plans are
  regenerated with the same seeds and styles and compared
  before/after. The subdivision may stop early more often on
  long thin rects, so a floor can hold fewer rooms than its
  target count — the existing early-stop behaviour, not a new
  one. Typical sizes per program (master bedroom 12x14 ft,
  secondary 10x12 ft, living 13x17 ft, kitchen 10x12 ft,
  dining 11x14 ft, bath 6x9 ft typical, office 10x11 ft, entry
  6x8 ft, laundry 6x8 ft, garage 12x20 ft single / 20x20 ft
  double minimum) live in the rules module as the reference
  table; only the hard floor, the aspect cap, and the
  classification are enforced. Numbers come from the study
  report; anything the report could not verify (Neufert/AGS
  room tables, a code threshold for hall-becomes-room, HUD
  room-size tables) stays out.
Revisit if: a style genuinely needs a room above 2.0:1 (a
  gallery, a bowling lane), or the hall needs to be a carved
  corridor rather than a leaf.
