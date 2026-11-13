//! Pitched gable roofs: the house reads as a house from the street,
//! not a box with a flat cap.
//!
//! One gable over the footprint bounding box (footprint rects plus
//! the garage), with a 0.4 m eave overhang. The ridge runs along the
//! longer axis; the two slopes are quads and the gable ends are
//! triangles. The porch gets its own roof in `use_cases::porch`.

use crate::domain::mesh::Mesh;
use crate::domain::model::House;
use crate::use_cases::materials::{ROOF, SIDING};
use crate::use_cases::walls::STOREY_H;

/// Eave overhang past the walls, in metres.
pub const EAVE: f32 = 0.4;
/// Roof rise per metre of half-span: a 45° pitch at 0.5.
pub const PITCH: f32 = 0.5;

/// Adds the pitched roof to `mesh`, at the top of the top storey.
pub fn add_roof(mesh: &mut Mesh, house: &House) {
    let base = (house.floors.len() - 1) as f32 * STOREY_H;
    let (x0, z0, x1, z1) = footprint_bounds(house);
    let (x0, z0, x1, z1) = (x0 - EAVE, z0 - EAVE, x1 + EAVE, z1 + EAVE);
    let wall_top = base + STOREY_H;
    if (x1 - x0) >= (z1 - z0) {
        let ridge_z = (z0 + z1) / 2.0;
        let ridge_y = wall_top + (z1 - z0) / 2.0 * PITCH;
        mesh.add_quad([x0, wall_top, z1], [x1, wall_top, z1], [x1, ridge_y, ridge_z], [x0, ridge_y, ridge_z], ROOF);
        mesh.add_quad([x1, wall_top, z0], [x0, wall_top, z0], [x0, ridge_y, ridge_z], [x1, ridge_y, ridge_z], ROOF);
        mesh.add_triangle([x0, wall_top, z0], [x0, wall_top, z1], [x0, ridge_y, ridge_z], SIDING);
        mesh.add_triangle([x1, wall_top, z1], [x1, wall_top, z0], [x1, ridge_y, ridge_z], SIDING);
    } else {
        let ridge_x = (x0 + x1) / 2.0;
        let ridge_y = wall_top + (x1 - x0) / 2.0 * PITCH;
        mesh.add_quad([x0, wall_top, z1], [x0, wall_top, z0], [ridge_x, ridge_y, z0], [ridge_x, ridge_y, z1], ROOF);
        mesh.add_quad([x1, wall_top, z0], [x1, wall_top, z1], [ridge_x, ridge_y, z1], [ridge_x, ridge_y, z0], ROOF);
        mesh.add_triangle([x0, wall_top, z0], [x1, wall_top, z0], [ridge_x, ridge_y, z0], SIDING);
        mesh.add_triangle([x1, wall_top, z1], [x0, wall_top, z1], [ridge_x, ridge_y, z1], SIDING);
    }
}

/// The bounding box of the footprint rects and the garage, in plan
/// coordinates. The porch is excluded: it has its own roof.
fn footprint_bounds(house: &House) -> (f32, f32, f32, f32) {
    let mut bounds = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
    for r in house.footprint.rects.iter().chain(house.footprint.garage.iter()) {
        bounds.0 = bounds.0.min(r.x as f32);
        bounds.1 = bounds.1.min(r.y as f32);
        bounds.2 = bounds.2.max(r.right() as f32);
        bounds.3 = bounds.3.max(r.bottom() as f32);
    }
    bounds
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::mesh::Mesh;
    use crate::domain::style::Style;
    use crate::use_cases::generate::generate_house;
    use crate::use_cases::materials::material_table;

    fn roofed(seed: u32, style: Style) -> Mesh {
        let house = generate_house(seed, style);
        let mut mesh = Mesh::new(material_table());
        add_roof(&mut mesh, &house);
        mesh
    }

    #[test]
    fn a_one_storey_roof_rises_above_the_walls() {
        let mesh = roofed(7, Style::Ranch);
        let ys: Vec<f32> = mesh.vertices.iter().map(|v| v[1]).collect();
        assert!(ys.iter().any(|&y| (y - STOREY_H).abs() < 0.01), "no eave at the wall top");
        assert!(ys.iter().any(|&y| y > STOREY_H + 1.0), "the roof does not rise");
    }

    #[test]
    fn a_two_storey_roof_sits_on_the_top_storey() {
        let mesh = roofed(7, Style::Colonial);
        let eave = mesh.vertices.iter().map(|v| v[1]).fold(f32::INFINITY, f32::min);
        let top = 2.0 * STOREY_H;
        assert!((eave - top).abs() < 0.01, "eave at {eave}, not the top storey's {top}");
    }

    #[test]
    fn the_roof_is_a_shell_of_two_slopes_and_two_gables() {
        let mesh = roofed(42, Style::Modern);
        let roof_faces = mesh.faces.iter().zip(mesh.materials.iter()).filter(|(_, m)| **m == ROOF).count();
        let gable_faces = mesh.faces.iter().zip(mesh.materials.iter()).filter(|(_, m)| **m == SIDING).count();
        assert_eq!(roof_faces, 4, "two slopes as quads");
        assert_eq!(gable_faces, 2, "two gable triangles");
        assert_eq!(mesh.validate(), Ok(()));
    }

    #[test]
    fn the_roof_is_deterministic() {
        assert_eq!(roofed(42, Style::Modern).vertices, roofed(42, Style::Modern).vertices);
    }
}
