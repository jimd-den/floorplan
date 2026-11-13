# ADR 0003: Furniture model, mesh, and interaction

## Status

Accepted. Implements spec §4.2 and §5, phases M0 (furniture model)
and M1 (furniture mesh, interaction).

## Context

The generator builds a procedural sandbox horror mechanic engine
(spec §1): random home layouts with doors and procedural interactive
furniture, Rust to WASM, first-person, three.js renderer. M0 had
re-tiling (bath ≤6 m², study ≤12 m², zero over-cap under the repo's
one-style-per-seed test; a 6-study strict residual remains — see
below). The engine now needs furniture as data, as mesh, and as
commands, so the viewer can render a furnished house and the player
can open doors, switch lamps, and move chairs and tables.

## Decision

### Furniture as data (`domain/furniture.rs`)

- `FurnitureKind`: Bed, Nightstand, Table, Chair, Sofa, Lamp, Desk,
  Tub, Sink, Shelf, Cabinet, Counter. Each kind has a fixed
  footprint (`size()`, facing south) and a `default_state()`.
- `Facing`: North / South / East / West — the direction a piece's
  front faces, into the room.
- `FurnitureState`: `Fixed` (bed, tub, sink, sofa, desk, shelf,
  counter), `On` / `Off` (lamp), `Open` / `Closed` (cabinet),
  `Moved { x, y }` (chair, table — stores the current position).
- `Furniture { id, kind, room_id, rect, facing, state }`.
  `Furniture::new` sets `Moved` at the placed position for movable
  kinds, else the kind's default state.
- Piece ids are `room.id * 100 + index in the room`, unique across
  the house, so commands address a piece by id.
- `is_movable()` — Chair and Table only. `is_tall()` — Shelf,
  Cabinet, Counter only; these are the pieces that can block a
  window (low pieces sit below the sill).

### Placement (`use_cases/furnish.rs`, `furnish_clear.rs`)

- `furniture_set(program)`: Bedroom [Bed, Nightstand, Cabinet],
  Dining [Table, Chair×4], Living [Sofa, Lamp, Table, Shelf],
  Bath [Tub, Sink], Study [Desk, Chair, Shelf], Kitchen [Counter,
  Table, Chair×2], Entry [Shelf], Hall [Shelf], Laundry [Shelf],
  Garage [].
- `furnish(rng, floor)`: per room, places each kind in set order.
  `place_against_wall` tries the four walls in seeded shuffled
  order, sliding along each wall in 0.1 m steps, first fit wins;
  east/west walls swap the footprint dimensions. `place_chair`
  sits a chair next to an already-placed Table or Desk (one
  candidate spot per side, centred, gap 0.15, facing the anchor),
  falling back to wall placement. A piece that fits nowhere is
  skipped. Deterministic: same floor + same RNG seed → same
  furniture.
- `fits` (`furnish_clear.rs`): inside the room, no overlap with
  placed pieces, no door block, no window block (window check only
  for tall kinds). The door swing zone is a rectangle the door's
  width along the wall (plus 0.1 m margin each side) and the door's
  width into the room, **on the room's side of the wall only** — a
  door swings into one room, not both. (The first version used a
  both-sides zone and was far too conservative.) A piece blocks a
  window only when it stands against the window's wall and its
  span covers the window's span.
- Placement rules, as tested over 200 seeds: inside its room;
  against a wall (chairs may instead sit next to a table/desk);
  no overlaps; no door-swing block; no window block (tall pieces
  only); every piece reachable (room reachable via the
  generation's connectivity guarantee, piece against a wall so
  the centre stays walkable).
- Placement rates measured over 200 seeds × 6 styles (1,200
  houses): 92% of baths get a tub (1495/1626) and 92% of bedrooms
  get a bed (2630/2853). The remaining ~8% are genuinely
  constrained small rooms (e.g. seed 0 Ranch bath 1: 2.4×2.4 with
  doors on opposite walls — the two 0.9 m swing zones plus the
  1.7 m tub exceed the 2.4 m depth). The integration test gates
  the **rules** for all placed pieces plus a **≥90% placement
  rate** for beds and tubs, rather than requiring every room to
  get its full set.
- `Floor` carries `pub furniture: Vec<Furniture>`; `build_floor`
  calls `furnish` after `place_doors` / `place_windows`.

### Furniture mesh (`use_cases/furniture_mesh.rs`, `furniture_parts.rs`)

- `FURNITURE_BASE: u32 = 15` (after the 5 structural + 10 program
  materials), `LAMP_ON: u32 = 27` (bright shade for a lamp that
  is on). `furniture_materials()` — 13 materials, one per kind
  in `FurnitureKind` order plus `lamp_on`. `furniture_material(kind)`.
- `furniture_boxes(floor, base)`: the composed boxes for every
  piece on a floor. A table is a top (0.05 thick at 0.75) plus
  four legs; a chair is a seat plus four legs plus a backrest on
  the side opposite its facing; a sofa is a base plus a backrest
  opposite facing plus two arms; a bed is a frame (0.35) plus a
  mattress (0.2, inset 0.05); a lamp is a base plus a pole (to
  1.5) plus a shade (1.5–1.75) in `LAMP_ON` when On; a desk is
  table boxes plus a drawer unit; a tub is a single 0.55 box; a
  sink is a pedestal plus a basin; a shelf is a single 1.8 box;
  a cabinet is a carcass (0.9) plus two door panels on its facing
  side — closed doors sit flush (0.02 out), open doors stick out
  0.33–0.35 m; a counter is a single 0.9 box.
- `piece_position(piece)`: a moved piece is drawn at its stored
  `Moved { x, y }` position, everything else at its placed rect —
  the mesh follows the interaction state.
- `extrude` adds `furniture_boxes` for each floor after the wall
  boxes and appends `furniture_materials()` to the material table
  (28 materials total). The OBJ/glTF writers need no change —
  they read the one domain `Mesh`, so furniture flows through
  them automatically.

### Door panels (`use_cases/door_panel.rs`)

- `DOOR: u32 = 28` (after the furniture materials). `door_panels`
  returns one panel box per door on a floor. Closed, the panel
  fills the gap in the wall (2.0 m tall, 0.05 thick, spanning the
  door width). Open, it stands perpendicular to the wall, swung
  into the `from` room from the hinge at the door's min end.
- The mesh follows the door state, so an `open` command is
  visible in the render.

### Interaction (`domain/interaction.rs`, `use_cases/interact.rs`)

- `Interaction`: `OpenDoor { door_id }`, `CloseDoor { door_id }`,
  `ToggleLamp { furniture_id }`, `MoveFurniture { furniture_id, to }`.
  (`take` is M5; not in this ADR.)
- `apply(house, command) -> bool`. A command that names nothing,
  or a move that would break a placement rule, is refused and
  returns false.
- Door state lives on the floor: `Floor.open_doors: Vec<u32>`
  (ids of open doors; all closed by default). `set_door` opens or
  closes a door by id; a window is not a door and is refused.
- `toggle_lamp` flips a lamp On/Off; anything that is not a lamp
  is refused.
- `move_furniture` moves a chair or table to a new rect. The
  destination must be a valid placement — inside the room,
  overlapping nothing, blocking no door swing and no (tall-piece)
  window — validated by reusing `fits` from `furnish_clear.rs`
  against the other furniture in the room. The piece's `rect` and
  `state` (`Moved { x, y }`) follow the move, and the mesh draws
  it at the stored position.

### JSON writer (`adapters/json.rs`)

- The full house JSON (`render_json`, CLI `--json`) now lists
  `furniture` and `open_doors` per floor, so the writers reflect
  the interaction state. A furniture entry carries id, kind,
  room, rect, facing, and state.
- The plan JSON (`render_plan_json`) — the canonical form the
  variety hash is taken over — stays free of furniture: furniture
  is a deterministic function of the plan, so it adds no
  independent decision and the variety count is unchanged.

## Consequences

- A furnished Modern seed 42 house (22 pieces, all 7 rooms
  complete) renders through the existing OBJ/glTF writers and
  the viewer; `open` / `toggle` / `move` commands change the
  state and the mesh follows.
- The 200-line file ceiling forced three splits: `furnish.rs`
  → `furnish_clear.rs`, `furniture_mesh.rs` → `furniture_parts.rs`,
  and the JSON / interact test modules moved to `tests/`.
- Residuals (open, for Jim's call): the 6-study strict residual
  (all 6 styles × 200 seeds finds 6 over-cap studies in thin
  rects; the repo's own M0 test passes); the ~8% of small rooms
  that skip a bed or tub.

## Alternatives considered

- Door state on `Opening` itself: rejected — it would change
  every `Opening` constructor in the crate. A `Vec<u32>` of open
  door ids on `Floor` is additive and keeps the door's geometry
  (the `Opening`) immutable.
- A both-sides door swing zone: rejected — far too conservative
  (baths 86%, bedrooms 83% placement). A door swings into one
  room only.
- Furniture in the plan JSON: rejected — the variety hash is over
  the plan JSON; furniture is deterministic from the plan, so it
  would change hash values without changing the variety count.
