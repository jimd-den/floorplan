# 0006 — The screenshot render adapter

Status: accepted · Date: 2026-11-12

## Context

The mesh (`domain::mesh::Mesh`) is the single 3D representation, and
the WASM adapter hands it to a three.js shell. That shell is outside
this repo and there is no browser on the build machine, so there was no
way to *look* at the mesh — every realism change (M6) would be made
blind. SRD-001 §4.3 also asks for a first-person read of the houses.

A screenshot path has to be deterministic, testable, and std-only like
the rest of the crate. It must not become the runtime renderer: the
WASM shell keeps drawing the frames.

## Decision

A native-only render adapter, four small pieces behind one command:

- `adapters/camera.rs` — pinhole projection, the near plane, and
  Sutherland–Hodgman clipping. `Camera::project` returns `None` behind
  the near plane; `clip_near` keeps the front part of a triangle, so a
  floor that passes under the eye still renders.
- `adapters/render.rs` — a z-buffer rasterizer with flat headlight
  shading (`0.55 + 0.45·|n·f|`), a sky gradient, and **no backface
  culling**, so a room's inner walls are visible from inside.
- `adapters/png.rs` — an 8-bit RGB PNG writer: hand-rolled chunk CRCs,
  zlib wrapper, and Adler-32, with stored deflate blocks. Screenshots
  are written once; compression would buy bytes and cost a crate.
- `use_cases/viewpoint.rs` — where to stand, as plain data
  (`Pose { eye, target }`). Exterior is centred on the footprint and
  stands back in proportion to its width; interior stands in the
  largest ground-floor room, trying the centre then the four inset
  corners until one is clear of furniture, and looks at the far corner
  at chest height; street stands on the busiest street and looks along
  it.
- `use_cases/scene.rs` — a mesh plus one ground box. Exported meshes
  stay ground-free so the OBJ, glTF, and WASM payloads are the
  building or the neighborhood alone.

The CLI command is `floorplan render --seed <u32> [--style <name>]
[--view exterior|interior|street] [--blocks WxH] [--out <path.png>]`,
defaulting to `render-<seed>-<view>.png` at 1280×720. The default name
deliberately avoids the `house-*` / `neighborhood-*` probe outputs that
§10 forbids committing.

## Alternatives considered

- **A Python rasterizer outside the crate.** Rejected: it would be a
  second 3D reader, untested by `make check`, and would drift from the
  mesh.
- **Headless three.js.** Rejected: no browser on the machine, and it
  would make the screenshot path depend on Node and a display stack.
- **A PNG crate (`png`, `flate2`).** Rejected: the zero-crate rule. The
  stored-deflate writer is ~100 lines and has its own round-trip test.
- **PPM output.** Rejected: trivial to write but not viewable in the
  review tools, and 3× the size of a stored PNG.
- **Backface culling.** Rejected: with box walls, culling hides the
  inner face of every wall, so an interior render is empty.

## Consequences

- `make check` covers the camera, the rasterizer, the PNG writer, the
  poses, and the scenes. The screenshot is reproducible from a seed.
- The rasterizer never reaches WASM: it is compiled only for the native
  CLI, and the WASM gate is unchanged.
- Rendering is fast enough to run on every change: 1280×720, a whole
  2×2 neighborhood (~60k faces) in well under a second in release.
- A realism pass (M6: pitched roofs, siding, framed windows, porches,
  driveways, trees) is now visible before it ships. The screenshots in
  `docs/screenshots/` are of the current mesh, which is pre-M6.
- Flat shading and one headlight are deliberate limits: no textures, no
  shadows, no per-room lights. M3 light play is the natural place to
  give the interior a real key light.

## Revisit if

M6 introduces textured materials (the PNG writer and the rasterizer
would need a sampler), or a three.js shell becomes available and the
screenshots should come from the real renderer.
