# 0010 — Spline streets replace the street grid (protocol v3)
Status: accepted · Date: 2026-11-15
Context: neighborhoods were a square street grid in one of three
  layout families (Grid, Offset, CulDeSac, `seed % 3`): streets and
  lots were axis-aligned rectangles, houses rotated only 0/90/180/
  270, and the lot merely fronted its street with no path to the
  door. Jim ordered the next milestone: organic fluid streets and
  sidewalks, curves replacing the grid, with sidewalks that lead to
  the entry door instead of just the house.
Decision: streets are Catmull-Rom spline centerlines through seeded
  control points (the curve passes through its control points, so
  the road goes where the generator places it, and it is
  C1-continuous, so there are no kinks). The network is a spine
  running across the area with branches curving off it; every branch
  ends in a turnaround bulb (a 300° arc, radius 14 m), which is a
  spline loop, not a layout family. Lots are polygons: the front
  edge is an arc of the street's offset curve, the sides run along
  the street normal, the back closes the lot. Sidewalks are 1.5 m
  ribbons offset from the centerline on both sides. Each house gets
  a 1.2 m walkway ribbon from the sidewalk edge to its entry door
  (the door's placed position drives it). House rotation is a
  continuous angle in degrees, clockwise from south on a map (0
  faces south, 90 faces west, 180 faces north, -90 faces east),
  taken from the street tangent at the lot center. The `Layout`
  enum, `plan_layout`, `lay_block`/`lay_court`, and `place_rect`
  are retired; `Street`/`Lot`/`PlacedHouse` carry splines,
  polygons, and a rotation + center instead of rectangles.
  `PROTOCOL_VERSION` bumps 2 → 3: the neighborhood JSON now
  carries street control points, lot polygons, and walkway paths.
Alternatives: Bezier centerlines (rejected: they do not pass
  through their control points, so placing lots on the road the
  generator asked for is harder); keeping the grid as an option
  (rejected: Jim chose replacement); walkways as lines (rejected:
  the spec wants real geometry); a full street graph with loops
  (rejected for now: a spine-plus-branches tree is connected by
  construction and needs no crossing logic — revisit if cross
  streets are wanted).
Consequences: the same seed produces a different neighborhood than
  the grid did — intended, the grid is gone. The mesh gains ribbon
  strips (street asphalt, sidewalks, walkways, lot ground) built
  from the sampled centerlines; the three.js shell renders from the
  mesh, so it picks them up with no shell change. The neighborhood
  variety bound is re-derived without the three layout families.
  The 200-line ceiling splits the new geometry into `domain/curve`
  (points, polylines, Catmull-Rom) and `domain/polygon` (containment
  and convex-quad overlap).
Revisit if: the network needs cross streets or loops rather than a
  tree; a bulb needs a different radius (14 m, kept from the old
  28 m court bulb).
