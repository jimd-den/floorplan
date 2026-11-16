# 0012 — Functional houses: base first, skeleton, walls, doors by opening (protocol v5)
Status: accepted · Date: 2026-11-16
Context: houses were built footprint first, with rooms from BSP
  splits, doors sized by a 0.6–0.9 m clamp, a fixed 1.0 m entry
  door, and furniture placed after the plan. The clamp meant a
  wide opening still got at most a 0.9 m door, so a grand opening
  read as a narrow one; the garage could be slid to a spot where
  it shared no wall with any room, so it had no door and no room
  could walk to it; and the mesh carried furniture the shell
  renders as clutter in rooms that should read empty and
  traversable.
Decision: generation order is now base -> skeleton -> walls ->
  doors. The base is a foundation slab (`Base`, one rect per
  footprint rect plus the garage, 0.3 m thick) laid before any
  room and carried in the House model, the JSON, and the mesh as
  concrete slabs. The skeleton is the room layout plus the
  circulation structure (`Skeleton`: rooms, adjacencies, and the
  hall spine as backbone links); it proves every room is
  connected, and walls and doors are built from it. Doors are
  sized by their opening: >= 3.0 m gets a 1.6 m double door (two
  0.8 m leaves), >= 2.0 m a 1.0 m wide single, >= 1.5 m a 0.9 m
  standard, else a 0.8 m narrow single, clamped to
  [0.6, opening - 0.6] and quantized to 0.1 m (`door_width`).
  A door at or above 1.4 m is two panel leaves in the mesh, each
  hinged at its outer end. The garage is only placed where it
  shares a wall with a room by at least a door's width
  (`attachable`), so it always gets a door. Furniture is removed
  from generation, JSON, and mesh: rooms are empty and walkable.
  Traversability is proved geometrically (`traverse`: a 0.1 m
  grid walk through door gaps, windows blocked), not by counting
  adjacencies. The furniture kinds stay in the domain for
  interactions and tasks, which operate on empty lists.
Alternatives: keep the clamp and size doors by room program
  (rejected: the opening is what a person walks through, so the
  opening is the honest input); keep furniture and make rooms
  walkable around it (rejected: Jim ordered it removed, and empty
  rooms are the traversability proof); exclude the garage from
  traversability (rejected: a garage is a room, and an unreachable
  room is a defect in the plan, not in the test).
Consequences: `PROTOCOL_VERSION` goes 4 -> 5. The house JSON
  gains `base` and loses the `furniture` array; the mesh gains
  concrete foundation slabs and loses furniture boxes, so the
  shell must update its version check and stop expecting
  furniture. The home-fixing tasks degrade to restore_power and
  close_doors (check_lights needs a lamp, catalog needs
  furniture). The neighborhood material constants are now derived
  from `CONCRETE` instead of literals: appending the concrete
  material had shifted street, lot, sidewalk, and the street
  furniture ids by one, which mislabelled lamp posts as sidewalk
  and failed the ground-faces-up test. Door panels for a double
  door are two leaves; a single leaf wider than 1.4 m no longer
  exists.
Revisit if: a room needs a door wider than 1.6 m (the double-door
  cap), or furniture returns as an opt-in layer on top of the
  empty plan.
