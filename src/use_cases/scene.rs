//! Viewing scenes: a mesh plus the ground it stands on.
//!
//! The exported meshes stay ground-free, so the OBJ, glTF, and WASM
//! payloads are the building or the neighborhood alone. A screenshot
//! needs a horizon, so this module composes the mesh with one ground
//! box underneath it. The ground top sits below every slab and lot, so
//! nothing z-fights and no building floats.

use crate::domain::geometry::Rect;
use crate::domain::mesh::{Box3, Material, Mesh};
use crate::domain::model::House;
use crate::domain::neighborhood::{Neighborhood, house_extent};
use crate::use_cases::extrude::extrude;
use crate::use_cases::neighborhood_mesh::neighborhood_mesh;

/// Below the floor slab (-0.1) and the lots (-0.06).
const GROUND_TOP: f32 = -0.12;
const GROUND_BOTTOM: f32 = -0.25;

/// A house with a lawn under it. The pad is large enough that the
/// exterior eye, which stands well south of the front, is still on it.
pub fn house_scene(house: &House) -> Mesh {
    let extent = house_extent(house);
    let pad = 80.0;
    let mut mesh = extrude(house);
    let lawn = Rect::new(
        extent.x - pad,
        extent.y - pad,
        extent.w + 2.0 * pad,
        extent.d + 2.0 * pad,
    );
    add_ground(&mut mesh, lawn);
    mesh
}

/// A neighborhood with a plain under it.
pub fn neighborhood_scene(neighborhood: &Neighborhood) -> Mesh {
    let mut mesh = neighborhood_mesh(neighborhood);
    add_ground(&mut mesh, bounds(neighborhood));
    mesh
}

/// One lawn-coloured box under everything.
fn add_ground(mesh: &mut Mesh, r: Rect) {
    let id = mesh.material_table.len() as u32;
    mesh.material_table.push(Material { name: "lawn", color: [0.38, 0.52, 0.30] });
    mesh.add_box(
        Box3::new(
            [r.x as f32, GROUND_BOTTOM, r.y as f32],
            [r.right() as f32, GROUND_TOP, r.bottom() as f32],
        ),
        id,
    );
}

/// Every street, lot, and house rect, padded so the plain reaches past
/// the last building.
fn bounds(n: &Neighborhood) -> Rect {
    let mut rects: Vec<Rect> = n.streets.iter().map(|s| s.rect).collect();
    rects.extend(n.lots.iter().map(|l| l.rect));
    rects.extend(n.houses.iter().map(|h| h.rect));
    let (mut min_x, mut min_y) = (f64::MAX, f64::MAX);
    let (mut max_x, mut max_y) = (f64::MIN, f64::MIN);
    for r in rects {
        min_x = min_x.min(r.x);
        min_y = min_y.min(r.y);
        max_x = max_x.max(r.right());
        max_y = max_y.max(r.bottom());
    }
    let pad = 1500.0;
    Rect::new(min_x - pad, min_y - pad, max_x - min_x + 2.0 * pad, max_y - min_y + 2.0 * pad)
}
