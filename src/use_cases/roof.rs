//! Pitched gable roofs: the house reads as a house from the street,
//! not a box with a flat cap.
//!
//! Each footprint rectangle gets its own gable, so an L, U or T
//! footprint roofs its wings separately instead of spanning a
//! courtyard. The ridge runs along the rectangle's longer axis, with
//! a 0.4 m eave overhang; gable ends are siding triangles. A wing
//! with no upper floor roofs at one storey, the main body at the top
//! storey, and the garage at one storey. The porch gets its own roof
//! in `use_cases::porch`.

use crate::domain::geometry::Rect;
use crate::domain::mesh::Mesh;
use crate::domain::model::House;
use crate::use_cases::floor::upper_rects;
use crate::use_cases::materials::{ROOF, SIDING};
use crate::use_cases::walls::STOREY_H;

/// Eave overhang past the walls, in metres.
pub const EAVE: f32 = 0.4;
/// Roof rise per metre of half-span: a 45° pitch at 0.5.
pub const PITCH: f32 = 0.5;

/// Adds a pitched gable over every footprint rectangle, at that
/// rectangle's own wall top.
pub fn add_roof(mesh: &mut Mesh, house: &House) {
    let top = house.floors.len() as f32 * STOREY_H;
    let upper = upper_rects(&house.footprint, house.style);
    for r in &house.footprint.rects {
        let wall_top = if upper.contains(r) { top } else { STOREY_H };
        add_gable(mesh, *r, wall_top);
    }
    if let Some(garage) = house.footprint.garage {
        add_gable(mesh, garage, STOREY_H);
    }
}

/// One gable over a rectangle: two slope quads and two gable
/// triangles, all `EAVE` past the rectangle.
fn add_gable(mesh: &mut Mesh, r: Rect, wall_top: f32) {
    let (x0, z0, x1, z1) = (
        r.x as f32 - EAVE,
        r.y as f32 - EAVE,
        r.right() as f32 + EAVE,
        r.bottom() as f32 + EAVE,
    );
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::mesh::Mesh;
    use crate::domain::style::Style;
    use crate::use_cases::generate::generate_house;
    use crate::use_cases::materials::material_table;

    fn roofed(seed: u32, style: Style) -> (House, Mesh) {
        let house = generate_house(seed, style);
        let mut mesh = Mesh::new(material_table());
        add_roof(&mut mesh, &house);
        (house, mesh)
    }

    fn count(mesh: &Mesh, material: u32) -> usize {
        mesh.materials.iter().filter(|m| **m == material).count()
    }

    #[test]
    fn a_rectangle_is_two_slopes_and_two_gables() {
        // Townhouse is always a single rectangle.
        let (_, mesh) = roofed(7, Style::Townhouse);
        assert_eq!(count(&mesh, ROOF), 4, "two slopes as quads");
        assert_eq!(count(&mesh, SIDING), 2, "two gable triangles");
        assert_eq!(mesh.validate(), Ok(()));
    }

    #[test]
    fn each_footprint_rectangle_gets_its_own_gable() {
        // Find a multi-rectangle footprint and check the roof count.
        for seed in 0..50u32 {
            for style in [Style::Ranch, Style::Modern, Style::Craftsman] {
                let house = generate_house(seed, style);
                if house.footprint.rects.len() < 2 {
                    continue;
                }
                let mut mesh = Mesh::new(material_table());
                add_roof(&mut mesh, &house);
                let rects = house.footprint.rects.len()
                    + house.footprint.garage.is_some() as usize;
                assert_eq!(
                    count(&mesh, ROOF),
                    4 * rects,
                    "seed {seed} {style:?}: {} rects, {} roof faces",
                    rects,
                    count(&mesh, ROOF)
                );
                assert_eq!(count(&mesh, SIDING), 2 * rects);
                return;
            }
        }
        panic!("no multi-rectangle footprint in 50 seeds");
    }

    #[test]
    fn a_two_storey_main_body_roofs_at_the_top_storey() {
        let (house, mesh) = roofed(7, Style::Colonial);
        assert_eq!(house.floors.len(), 2);
        let roof_top = mesh.vertices.iter().map(|v| v[1]).fold(0.0f32, f32::max);
        assert!(roof_top > 2.0 * STOREY_H, "the main roof does not rise above the top storey");
    }

    #[test]
    fn a_single_storey_wing_roofs_at_one_storey() {
        // A two-storey Colonial with an L footprint: the wing has no
        // upper floor, so its roof must sit at one storey, not two.
        for seed in 0..50u32 {
            let house = generate_house(seed, Style::Colonial);
            if house.footprint.rects.len() < 2 {
                continue;
            }
            let mut mesh = Mesh::new(material_table());
            add_roof(&mut mesh, &house);
            let low: Vec<f32> = mesh.vertices.iter().map(|v| v[1]).filter(|&y| y < 1.5 * STOREY_H).collect();
            assert!(
                low.iter().any(|&y| (y - STOREY_H).abs() < 0.01),
                "seed {seed}: no one-storey wing roof"
            );
            return;
        }
        panic!("no two-storey Colonial with a wing in 50 seeds");
    }

    #[test]
    fn the_garage_roofs_at_one_storey_even_in_a_two_storey_house() {
        for seed in 0..50u32 {
            let house = generate_house(seed, Style::Colonial);
            if house.footprint.garage.is_none() {
                continue;
            }
            let mut mesh = Mesh::new(material_table());
            add_roof(&mut mesh, &house);
            let garage = house.footprint.garage.unwrap();
            let pad = 0.01;
            let g = Rect::new(
                garage.x - EAVE as f64 - pad,
                garage.y - EAVE as f64 - pad,
                garage.w + 2.0 * EAVE as f64 + 2.0 * pad,
                garage.d + 2.0 * EAVE as f64 + 2.0 * pad,
            );
            let garage_faces = mesh.faces.iter().filter(|f| {
                f.iter().all(|&v| {
                    let p = mesh.vertices[v as usize];
                    g.contains_point(p[0] as f64, p[2] as f64) && p[1] <= STOREY_H + 1.8
                })
            }).count();
            assert!(
                garage_faces >= 4,
                "seed {seed}: no garage roof at one storey (found {garage_faces} faces)"
            );
            return;
        }
        panic!("no two-storey Colonial with a garage in 50 seeds");
    }

    #[test]
    fn the_roof_is_deterministic() {
        let (_, a) = roofed(42, Style::Modern);
        let (_, b) = roofed(42, Style::Modern);
        assert_eq!(a.vertices, b.vertices);
    }
}
