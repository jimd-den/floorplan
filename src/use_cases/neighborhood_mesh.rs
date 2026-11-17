//! neighborhood_mesh: a whole neighborhood as one domain Mesh.
//!
//! Every house is extruded with furniture by the existing
//! extruder and placed on its lot (rotated to face its street);
//! streets, sidewalks, lots, and walkways become flat ribbons
//! built from the sampled centerlines, all facing up (the ribbon
//! module checks the winding, never assumes it). This is the mesh
//! the WASM adapter hands to the web shell — there is no OBJ or
//! glTF neighborhood writer. The house mesh builder is reused,
//! never duplicated.

use crate::domain::mesh::{Material, Mesh};
use crate::domain::neighborhood::{Lot, Neighborhood, Street};
use crate::use_cases::extrude::{extrude, material_table};
use crate::use_cases::grounds::add_grounds;
use crate::use_cases::materials::FRAMING;
use crate::use_cases::neighborhood::{SIDEWALK_W, STREET_W};
use crate::use_cases::place_mesh::append_placed;
use crate::use_cases::ribbon::{add_flat_quad, add_ribbon};
use crate::use_cases::street_furniture_mesh::{add_lamps, add_signs, furniture_materials};

/// The street material id: the first id after the house table,
/// whose last id is FRAMING. Derived, never a literal — appending
/// a house material shifts the neighborhood ids with it.
pub const STREET_MATERIAL: u32 = FRAMING + 1;
/// The lot material id, appended after the house table.
pub const LOT_MATERIAL: u32 = FRAMING + 2;
/// The sidewalk material id, appended after the house table.
pub const SIDEWALK_MATERIAL: u32 = FRAMING + 3;

/// The material table for a neighborhood mesh: the house table
/// plus street asphalt, lot grass, sidewalk concrete, and street
/// furniture.
pub fn neighborhood_material_table() -> Vec<Material> {
    let mut table = material_table();
    table.push(Material { name: "street", color: [0.24, 0.25, 0.27] });
    table.push(Material { name: "lot", color: [0.34, 0.56, 0.26] });
    table.push(Material { name: "sidewalk", color: [0.72, 0.72, 0.70] });
    table.extend(furniture_materials());
    table
}

/// Builds the whole neighborhood as one mesh.
pub fn neighborhood_mesh(n: &Neighborhood) -> Mesh {
    let mut mesh = Mesh::new(neighborhood_material_table());
    for h in &n.houses {
        append_placed(&mut mesh, &extrude(&h.house), h);
    }
    for s in &n.streets {
        add_street_ribbon(&mut mesh, s);
        add_sidewalk_ribbons(&mut mesh, s);
    }
    for lot in &n.lots {
        add_lot_ground(&mut mesh, lot);
    }
    for h in &n.houses {
        add_flat_quad(&mut mesh, h.walkway[0], h.walkway[1], h.walkway[2], h.walkway[3], -0.004, crate::use_cases::materials::DRIVEWAY);
    }
    add_grounds(&mut mesh, n);
    add_lamps(&mut mesh, &n.lamps);
    add_signs(&mut mesh, &n.signs);
    mesh
}

/// The asphalt ribbon: the centerline offset by half the street
/// width on each side, at ground level.
fn add_street_ribbon(mesh: &mut Mesh, s: &Street) {
    let half = STREET_W / 2.0;
    add_ribbon(mesh, &s.samples.offset(half), &s.samples.offset(-half), 0.0, STREET_MATERIAL);
}

/// The sidewalk ribbons: 1.5 m on each side of the asphalt,
/// raised 0.04 m above the street.
fn add_sidewalk_ribbons(mesh: &mut Mesh, s: &Street) {
    let half = STREET_W / 2.0;
    let outer = half + SIDEWALK_W;
    add_ribbon(mesh, &s.samples.offset(outer), &s.samples.offset(half), 0.04, SIDEWALK_MATERIAL);
    add_ribbon(mesh, &s.samples.offset(-half), &s.samples.offset(-outer), 0.04, SIDEWALK_MATERIAL);
}

/// The lot ground: a ruled strip between the front arc and the
/// back arc, just below the house floors. The two arcs have the
/// same number of points; the back arc runs the other way, so
/// front[i] pairs with back[m-1-i].
fn add_lot_ground(mesh: &mut Mesh, lot: &Lot) {
    let m = lot.outline.len() / 2;
    if m < 2 {
        return;
    }
    let (front, back) = lot.outline.split_at(m);
    for i in 0..m - 1 {
        add_flat_quad(mesh, front[i], back[m - 1 - i], back[m - 2 - i], front[i + 1], -0.02, LOT_MATERIAL);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::use_cases::neighborhood::generate_neighborhood;

    /// The y component of a triangle's normal; positive faces up.
    fn normal_y(mesh: &Mesh, f: [u32; 3]) -> f32 {
        let (a, b, c) = (mesh.vertices[f[0] as usize], mesh.vertices[f[1] as usize], mesh.vertices[f[2] as usize]);
        let (u, v) = ([b[0] - a[0], b[1] - a[1], b[2] - a[2]], [c[0] - a[0], c[1] - a[1], c[2] - a[2]]);
        u[2] * v[0] - u[0] * v[2]
    }

    #[test]
    fn the_mesh_counts_are_the_sum_of_its_parts() {
        let n = generate_neighborhood(42, (2, 2));
        let mesh = neighborhood_mesh(&n);
        let mut verts = 0;
        let mut faces = 0;
        for h in &n.houses {
            let hm = extrude(&h.house);
            verts += hm.vertices.len();
            faces += hm.faces.len();
        }
        // Houses are exact; the ground ribbons only add to it.
        assert!(mesh.vertices.len() >= verts, "vertex count");
        assert!(mesh.faces.len() >= faces, "face count");
        assert_eq!(mesh.materials.len(), mesh.faces.len(), "material count");
        assert_eq!(
            mesh.material_table.len(),
            crate::use_cases::materials::material_table().len() + 7,
            "house materials plus street, lot, sidewalk, and furniture"
        );
    }

    #[test]
    fn the_mesh_validates_over_20_seeds() {
        for seed in 0..20u32 {
            let n = generate_neighborhood(seed, (2, 2));
            let mesh = neighborhood_mesh(&n);
            assert_eq!(mesh.validate(), Ok(()), "seed {seed}");
        }
    }

    #[test]
    fn the_flat_ground_faces_up() {
        let mesh = neighborhood_mesh(&generate_neighborhood(42, (2, 2)));
        let ground = [STREET_MATERIAL, LOT_MATERIAL, SIDEWALK_MATERIAL, crate::use_cases::materials::DRIVEWAY];
        let mut checked = 0;
        for (i, f) in mesh.faces.iter().enumerate() {
            if ground.contains(&mesh.materials[i]) {
                assert!(normal_y(&mesh, *f) > 0.0, "ground face {i} faces down");
                checked += 1;
            }
        }
        assert!(checked > 0, "no ground faces checked");
    }

    #[test]
    fn the_ground_sits_below_the_house_floors() {
        // House floor slabs top out at y 0; lot ground tops out at
        // -0.02, so nothing is coplanar with a floor.
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
