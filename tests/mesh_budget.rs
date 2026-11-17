//! NFR-2: the meshes stay small enough for a first-person WASM frame.
//!
//! These are regression guards, not drivers: they name the budget the
//! screenshot adapter and the WASM shell rely on, so a later realism
//! pass that doubles the geometry fails here first.

use floorplan::domain::style::Style;
use floorplan::use_cases::extrude::extrude;
use floorplan::use_cases::generate::generate_house;
use floorplan::use_cases::neighborhood::generate_neighborhood;
use floorplan::use_cases::neighborhood_mesh::neighborhood_mesh;

/// One house, every style. A house is drawn alone, so it gets the
/// tighter budget: ~9.5k vertices and ~14.2k triangles. The M6
/// realism pass (pitched roof, siding, frames, baseboards, mouldings,
/// relief, painted linings) spends geometry on reads-as-a-house; the
/// guard moved with it. It moved again twice for correctness: the
/// trim fix (bands stop at doors, +24 boxes on Colonial) and the
/// stem wall (a concrete foundation wall around the perimeter, +63
/// boxes on Colonial) — Colonial, the largest house, is now
/// 9196 vertices / 13724 faces. It moved again for the room-size
/// rules (ADR 0014): bath-sized strip pieces and hall/entry
/// re-tiling add leaves, so Colonial grows to 9556 vertices /
/// 14264 faces. It moved again for the BSP post-pass (ADR 0014):
/// over-cap leaves are re-split into aspect-valid pieces, so
/// Colonial grows to 10660 vertices / 15920 faces. The guard is
/// a performance budget, not a shape rule; the increase buys
/// rooms that pass the study.
#[test]
fn a_house_mesh_fits_the_first_person_budget() {
    for style in Style::all() {
        let mesh = extrude(&generate_house(42, style));
        assert!(
            mesh.vertices.len() <= 10_700,
            "{style:?}: {} vertices",
            mesh.vertices.len()
        );
        assert!(mesh.faces.len() <= 16_000, "{style:?}: {} faces", mesh.faces.len());
    }
}

/// A whole 2x2 neighborhood. This is the payload the WASM shell hands
/// to three.js, so it is the one that decides whether a frame is fast.
#[test]
fn a_neighborhood_mesh_fits_the_streaming_budget() {
    let mesh = neighborhood_mesh(&generate_neighborhood(42, (2, 2)));

    assert!(mesh.vertices.len() <= 115_000, "{} vertices", mesh.vertices.len());
    assert!(mesh.faces.len() <= 175_000, "{} faces", mesh.faces.len());
    assert_eq!(mesh.validate(), Ok(()));
}
