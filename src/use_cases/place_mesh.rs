//! Placing one house's mesh onto its lot.
//!
//! The house mesh is in local coordinates (x east, y up, z
//! south, origin at the extent's north-west corner); the placed
//! centre and rotation put it on the lot. The placement is a
//! proper rotation about the extent centre (see `placed_point`),
//! so the winding never flips — the old 90° reflection is gone.

use crate::domain::curve::Pt;
use crate::domain::geometry::Rect;
use crate::domain::mesh::Mesh;
use crate::domain::neighborhood::PlacedHouse;
use crate::domain::placement::{house_extent, placed_point};

/// Appends one house's mesh, placed and rotated on its lot.
pub fn append_placed(mesh: &mut Mesh, house_mesh: &Mesh, placed: &PlacedHouse) {
    let extent = house_extent(&placed.house);
    let base = mesh.vertices.len() as u32;
    for v in &house_mesh.vertices {
        let (wx, wz) = place_point(v[0] as f64, v[2] as f64, extent, placed);
        mesh.vertices.push([wx as f32, v[1], wz as f32]);
    }
    for (i, f) in house_mesh.faces.iter().enumerate() {
        mesh.faces.push([f[0] + base, f[1] + base, f[2] + base]);
        mesh.materials.push(house_mesh.materials[i]);
    }
}

/// Maps a house-local (x, z) point into its placed position: a
/// proper rotation about the extent centre (see `placed_point`),
/// so the winding never flips — the old 90° reflection is gone.
fn place_point(x: f64, z: f64, extent: Rect, placed: &PlacedHouse) -> (f64, f64) {
    let p = placed_point(placed.center, placed.rotation, extent, Pt::new(x, z));
    (p.x, p.y)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn placing_a_point_puts_the_front_on_the_facing_side() {
        let extent = Rect::new(0.0, 0.0, 2.0, 3.0);
        let centre = Pt::new(10.0, 20.0);
        // The front edge midpoint, local (1, 3), at each cardinal
        // rotation lands on the facing side of the placed extent.
        let front = |rotation: f64| {
            let placed = PlacedHouse {
                house: crate::use_cases::generate::generate_house(1, crate::domain::style::Style::Ranch),
                lot: 0,
                rotation,
                center: centre,
                walkway: [Pt::new(0.0, 0.0); 4],
            };
            place_point(1.0, 3.0, extent, &placed)
        };
        assert_eq!(front(0.0), (10.0, 21.5));
        assert_eq!(front(180.0), (10.0, 18.5));
        assert_eq!(front(-90.0), (11.5, 20.0));
        assert_eq!(front(90.0), (8.5, 20.0));
    }
}
