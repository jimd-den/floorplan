# 0013 — Framing skeleton, basements, realistic doors, trim that stops, real concrete (protocol v5)
Status: accepted · Date: 2026-11-16
Context: ADR 0012's "skeleton" was a logical circulation graph
  (rooms, adjacencies, hall spine) that proved connectivity. Jim
  corrected it after seeing the renders: he meant a framing
  skeleton — structural framing, not a circulation proof. The
  same look at the renders found three more defects: doors were
  sized by opening up to a 1.6 m double, so they read as giant
  brown slabs; wall trim lines ran continuously across door
  openings; and the foundation concrete rendered as a flat grey
  rectangle sitting on the grass, a ground patch rather than
  construction. Houses had no below-grade level at all.
Decision: the skeleton stage is now a structural framing
  skeleton (`framing`: wall framing — bottom plate, studs, top
  plate; floor framing — joists + rim; roof framing —
  rafters/joists + ridge beam), an explicit stage in the House
  model, the JSON, and the mesh. Walls, doors, and windows attach
  to the frame. The circulation proof (`traverse`) stays as a
  check, not the skeleton. Every house gets a basement: a
  below-grade level with foundation walls, a basement floor slab,
  and stairs from the ground floor down, in the House model, the
  JSON, and the mesh, with a permanent `--view basement` camera
  as its proof view. Doors are realistic residential widths
  (`door_width`): entry 0.9 m single, interior 0.8 m standard,
  0.7 m for small rooms (< 6.0 m²), clamp [0.6, opening − 0.6];
  at most one double per floor, capped at 1.2 m (two 0.6 m
  leaves), only on the living/dining opening. Wall trim stops at
  doors (`trim`): baseboard and chair rail split at door openings
  on that room edge and resume past them; the crown runs
  continuously over the 2.1 m door head. Concrete is foundation
  only (`foundation`): a concrete stem wall around the ground
  floor's true outline, footing (y −0.3) to 0.2 m above grade,
  0.2 m thick, gaps at exterior doors, inner face flush with the
  wall so it never shows indoors, proud 0.05 m outside; the porch
  is a wood deck, not concrete, so concrete can never be confused
  with ground cover and the lot stays grass.
Alternatives: keep the circulation graph as the skeleton and add
  framing as decoration (rejected: Jim's correction is a
  functional definition — the frame is the structure the house is
  built on, not a label); make basements opt-in per seed or style
  (rejected: a house without a below-grade level is the exception
  in this engine's realism, and every house gets one, with
  size/layout varying by seed/style as part of permutations);
  size doors by opening with a lower cap (rejected: the opening
  is not the input — residential widths are, and a wide opening
  gets a single 0.8 m door with wall beside it, not a wider
  door); run trim over doors and hide it with the door frame
  (rejected: the line is geometry, and geometry that crosses a
  doorway is wrong even if a frame covers part of it); keep the
  porch concrete and tint it (rejected: the conflict is material
  identity, not colour — concrete on the ground reads as ground
  no matter the shade).
Consequences: `PROTOCOL_VERSION` stays 5 — the JSON and mesh
  shapes did not change, only values and box counts, so the shell
  needs only the new WASM, not a version change. The house JSON
  gains `framing` and `basement`; the mesh gains framing members,
  a stem wall, and a wood porch deck. The stem wall outline uses
  `exposed_ranges` (overlap subtraction), not the count==1 flag:
  a long wall beside two short rooms is shared along its whole
  length, and the first build wrongly put 51 stem-wall boxes on
  seed 42, 34 of them on interior shared walls, showing as a grey
  strip indoors; seed 42 now gets 17 boxes, 0 intruding. Mesh
  budget is 9500 vertices / 14200 faces (Colonial 9196 / 13724).
  Gate 493 passed, 0 failed; WASM da77d461 (410,654 bytes);
  harness 726/726.
Revisit if: a style needs a basement that is not full-footprint
  (walk-out, crawlspace), or a door wider than 1.2 m is genuinely
  required somewhere a real house would put one.
