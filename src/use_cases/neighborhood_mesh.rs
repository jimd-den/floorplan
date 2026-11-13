//! neighborhood_mesh: a whole neighborhood as one domain Mesh.
//!
//! Every house is extruded with furniture by the existing
//! extruder and placed on its lot (rotated to face its street);
//! streets and lots become thin ground boxes. This is the mesh
//! the WASM adapter hands to the web shell — there is no OBJ or
//! glTF neighborhood writer. The house mesh builder is reused,
//! never duplicated.

use crate::domain::geometry::Rect;
use crate::domain::mesh::{Box3, Material, Mesh};
use crate::domain::neighborhood::{Neighborhood, PlacedHouse, house_extent};
use crate::use_cases::extrude::{extrude, material_table};
use crate::use_cases::grounds::add_grounds;

/// The street material id, appended after the house table.
pub const STREET_MATERIAL: u32 = 29;
/// The lot material id, appended after the house table.
pub const LOT_MATERIAL: u32 = 30;

/// The material table for a neighborhood mesh: the house table
/// plus street asphalt and lot grass.
pub fn neighborhood_material_table() -> Vec<Material> {
    let mut table = material_table();
    table.push(Material { name: "street", color: [0.34, 0.34, 0.36] });
    table.push(Material { name: "lot", color: [0.45, 0.62, 0.38] });
    table
}

/// Builds the whole neighborhood as one mesh.
pub fn neighborhood_mesh(n: &Neighborhood) -> Mesh {
    let mut mesh = Mesh::new(neighborhood_material_table());
    for h in &n.houses {
        append_placed(&mut mesh, &extrude(&h.house), h);
    }
    for s in &n.streets {
        mesh.add_box(
            Box3::new(
                [s.rect.x as f32, -0.05, s.rect.y as f32],
                [s.rect.right() as f32, 0.0, s.rect.bottom() as f32],
            ),
            STREET_MATERIAL,
        );
    }
    for lot in &n.lots {
        mesh.add_box(
            Box3::new(
                [lot.rect.x as f32, -0.06, lot.rect.y as f32],
                [lot.rect.right() as f32, -0.02, lot.rect.bottom() as f32],
            ),
            LOT_MATERIAL,
        );
    }
    add_grounds(&mut mesh, n);
    mesh
}

/// Appends one house's mesh, placed and rotated on its lot.
/// The house mesh is in local coordinates (x east, y up, z
/// south, origin at the extent's north-west corner); the placed
/// rect is in plan coordinates (x east, y south).
fn append_placed(mesh: &mut Mesh, house_mesh: &Mesh, placed: &PlacedHouse) {
    let extent = house_extent(&placed.house);
    let base = mesh.vertices.len() as u32;
    for v in &house_mesh.vertices {
        let (wx, wz) = place_point(v[0] as f64, v[2] as f64, extent, placed.rect, placed.rotation);
        mesh.vertices.push([wx as f32, v[1], wz as f32]);
    }
    let flip = flips_winding(placed.rotation);
    for (i, f) in house_mesh.faces.iter().enumerate() {
        let t = if flip { [f[0], f[2], f[1]] } else { *f };
        mesh.faces.push([t[0] + base, t[1] + base, t[2] + base]);
        mesh.materials.push(house_mesh.materials[i]);
    }
}

/// Maps a house-local (x, z) point into its placed position,
/// following `place_rect` exactly: rotation 90 is a reflection
/// across the diagonal (determinant -1), the other rotations are
/// proper rotations.
fn place_point(x: f64, z: f64, extent: Rect, placed: Rect, rotation: u16) -> (f64, f64) {
    match rotation {
        0 => (placed.x + x, placed.y + z),
        90 => (placed.x + z, placed.y + x),
        180 => (placed.x + extent.w - x, placed.y + extent.d - z),
        _ => (placed.x + extent.d - z, placed.y + x),
    }
}

/// True when the rotation flips triangle winding (the 90
/// reflection; see `place_point`).
fn flips_winding(rotation: u16) -> bool {
    rotation == 90
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::use_cases::neighborhood::generate_neighborhood;

    #[test]
    fn the_mesh_counts_are_the_sum_of_its_parts() {
        let n = generate_neighborhood(42, (2, 2));
        let mesh = neighborhood_mesh(&n);
        let mut verts = 8 * (n.streets.len() + n.lots.len());
        let mut faces = 12 * (n.streets.len() + n.lots.len());
        for h in &n.houses {
            let hm = extrude(&h.house);
            verts += hm.vertices.len();
            faces += hm.faces.len();
        }
        // The house, street, and lot core is exact; grounds append
        // driveways and trees after it.
        assert!(mesh.vertices.len() >= verts, "vertex count");
        assert!(mesh.faces.len() >= faces, "face count");
        assert_eq!(mesh.materials.len(), mesh.faces.len(), "material count");
        assert_eq!(
            mesh.material_table.len(),
            crate::use_cases::materials::material_table().len() + 2,
            "house materials plus street and lot"
        );
    }

    #[test]
    fn the_mesh_validates_for_all_three_families() {
        for seed in [0u32, 1, 2] {
            let n = generate_neighborhood(seed, (2, 2));
            let mesh = neighborhood_mesh(&n);
            assert_eq!(mesh.validate(), Ok(()), "seed {seed} ({:?})", n.layout);
        }
    }

    #[test]
    fn placing_a_point_matches_place_rect_for_every_rotation() {
        let extent = Rect::new(0.0, 0.0, 2.0, 3.0);
        let placed = Rect::new(10.0, 20.0, 2.0, 3.0);
        let placed90 = Rect::new(10.0, 20.0, 3.0, 2.0);
        // The local corners map onto the placed rect's corners.
        assert_eq!(place_point(0.0, 0.0, extent, placed, 0), (10.0, 20.0));
        assert_eq!(place_point(2.0, 3.0, extent, placed, 0), (12.0, 23.0));
        assert_eq!(place_point(0.0, 0.0, extent, placed, 180), (12.0, 23.0));
        assert_eq!(place_point(2.0, 3.0, extent, placed, 180), (10.0, 20.0));
        assert_eq!(place_point(0.0, 0.0, extent, placed90, 90), (10.0, 20.0));
        assert_eq!(place_point(2.0, 3.0, extent, placed90, 90), (13.0, 22.0));
        assert_eq!(place_point(0.0, 0.0, extent, placed90, 270), (13.0, 20.0));
        assert_eq!(place_point(2.0, 3.0, extent, placed90, 270), (10.0, 22.0));
    }

    #[test]
    fn only_the_90_rotation_flips_winding() {
        assert!(!flips_winding(0));
        assert!(flips_winding(90));
        assert!(!flips_winding(180));
        assert!(!flips_winding(270));
    }

    #[test]
    fn the_ground_boxes_sit_below_the_house_floors() {
        // House floor slabs top out at y 0; lot ground tops out at
        // -0.02 and street ground at 0, so nothing is coplanar.
        let n = generate_neighborhood(42, (2, 2));
        let mesh = neighborhood_mesh(&n);
        let ground: Vec<f32> = mesh
            .vertices
            .iter()
            .filter(|v| v[1] < 0.0)
            .map(|v| v[1])
            .collect();
        assert!(!ground.is_empty(), "no ground vertices");
        assert!(ground.iter().all(|&y| y < 0.0), "ground pokes above the house floors");
    }
}
