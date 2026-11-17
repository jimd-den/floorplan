//! basement_mesh: the basement as mesh — concrete foundation walls
//! from the footing to grade, the basement floor slab, and the
//! stair down from the ground-floor hall. Full LOD only: the
//! basement is detail, not massing. The walls and slab are
//! CONCRETE, so the foundation reads as construction, never as a
//! ground patch; the stair is STAIRS, like the upper flight.

use crate::domain::mesh::{Box3, Mesh};
use crate::domain::model::House;
use crate::use_cases::materials::{CONCRETE, STAIRS};

/// Adds the basement of `house` to `mesh`: foundation walls, floor
/// slab, and the stair down from the hall.
pub fn add_basement(mesh: &mut Mesh, house: &House) {
    let b = &house.basement;
    let r = b.rect;
    let depth = b.depth as f32;
    let t = crate::domain::basement::FOUNDATION_WALL as f32;
    let (x0, z0) = (r.x as f32, r.y as f32);
    let (x1, z1) = (r.right() as f32, r.bottom() as f32);
    // Foundation walls: a concrete perimeter from the footing
    // (the basement floor) up to grade. North/south run the full
    // width; east/west are inset so the corners are not doubled.
    mesh.add_box(Box3::new([x0, -depth, z0], [x1, 0.0, z0 + t]), CONCRETE);
    mesh.add_box(Box3::new([x0, -depth, z1 - t], [x1, 0.0, z1]), CONCRETE);
    mesh.add_box(Box3::new([x0, -depth, z0 + t], [x0 + t, 0.0, z1 - t]), CONCRETE);
    mesh.add_box(Box3::new([x1 - t, -depth, z0 + t], [x1, 0.0, z1 - t]), CONCRETE);
    // Basement floor slab: concrete under the whole rect, its top
    // the basement floor.
    let slab = crate::domain::basement::BASEMENT_SLAB as f32;
    mesh.add_box(Box3::new([x0, -depth - slab, z0], [x1, -depth, z1]), CONCRETE);
    // The stair down from the hall, solid boxes from the basement
    // floor up to each tread (mirroring `stairs_boxes`, inverted).
    for (b, m) in basement_stairs_boxes(house) {
        mesh.add_box(b, m);
    }
}

/// The basement stair as stepped boxes: step `i` (0 = lowest, at
/// the far end of the run) is solid from the basement floor up to
/// its tread; the top step is flush with the ground floor.
fn basement_stairs_boxes(house: &House) -> Vec<(Box3, u32)> {
    use crate::domain::basement::{BASEMENT_RISE, BASEMENT_TREAD};
    use crate::domain::geometry::Axis;
    let b = &house.basement;
    let depth = b.depth;
    let steps = (depth / BASEMENT_RISE).round() as usize;
    if steps == 0 {
        return vec![];
    }
    let rise = (depth / steps as f64) as f32;
    let tread = BASEMENT_TREAD as f32;
    let depth_f = depth as f32;
    let width = 1.0f32;
    let (tx, tz) = (b.stair_top.0 as f32, b.stair_top.1 as f32);
    let mut out = vec![];
    for i in 0..steps {
        // Step i from the bottom: its top is (i+1) rises above the
        // basement floor; it sits (steps-1-i) treads from the top.
        let top = -depth_f + (i + 1) as f32 * rise;
        let off = (steps - 1 - i) as f32 * tread;
        let (min, max) = match b.stair_axis {
            Axis::Horizontal => (
                [tx + off, -depth_f, tz - width / 2.0],
                [tx + off + tread, top, tz + width / 2.0],
            ),
            Axis::Vertical => (
                [tx - width / 2.0, -depth_f, tz + off],
                [tx + width / 2.0, top, tz + off + tread],
            ),
        };
        out.push((Box3::new(min, max), STAIRS));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::style::Style;
    use crate::use_cases::generate::generate_house;
    use crate::use_cases::materials::material_table;

    fn basement_mesh(seed: u32, style: Style) -> Mesh {
        let house = generate_house(seed, style);
        let mut mesh = Mesh::new(material_table());
        add_basement(&mut mesh, &house);
        mesh
    }

    #[test]
    fn the_basement_has_concrete_foundation_walls_below_grade() {
        let house = generate_house(42, Style::Modern);
        let mesh = basement_mesh(42, Style::Modern);
        // Foundation walls: CONCRETE faces entirely below grade
        // (y <= 0), forming the basement perimeter.
        let below: Vec<f32> = mesh
            .faces
            .iter()
            .zip(mesh.materials.iter())
            .filter(|(_, m)| **m == CONCRETE)
            .flat_map(|(f, _)| f.iter().map(|&v| mesh.vertices[v as usize][1]))
            .collect();
        assert!(!below.is_empty(), "no concrete below grade");
        assert!(below.iter().all(|&y| y <= 0.0), "concrete above grade");
        let lowest = below.iter().fold(0.0f32, |a, &b| a.min(b));
        // The walls reach the basement floor; the slab bottom sits
        // one slab-thickness below it.
        let expect = -(house.basement.depth as f32)
            - crate::domain::basement::BASEMENT_SLAB as f32;
        assert!(
            (lowest - expect).abs() < 1e-6,
            "deepest concrete {lowest} should be the slab bottom {expect}"
        );
    }

    #[test]
    fn the_basement_stair_descends_to_the_basement_floor() {
        let house = generate_house(42, Style::Modern);
        let mesh = basement_mesh(42, Style::Modern);
        let stair_low = mesh
            .faces
            .iter()
            .zip(mesh.materials.iter())
            .filter(|(_, m)| **m == STAIRS)
            .flat_map(|(f, _)| f.iter().map(|&v| mesh.vertices[v as usize][1]))
            .fold(0.0f32, |a, b| a.min(b));
        assert!(
            (stair_low + house.basement.depth as f32).abs() < 0.25,
            "stair lowest {stair_low} should reach the basement floor {}",
            -house.basement.depth
        );
    }

    #[test]
    fn the_basement_stair_stays_inside_the_basement_rect() {
        let house = generate_house(42, Style::Modern);
        let mesh = basement_mesh(42, Style::Modern);
        let r = house.basement.rect;
        let (x0, z0) = (r.x as f32, r.y as f32);
        let (x1, z1) = (r.right() as f32, r.bottom() as f32);
        for (f, m) in mesh.faces.iter().zip(mesh.materials.iter()) {
            if *m != STAIRS {
                continue;
            }
            for &v in f {
                let p = mesh.vertices[v as usize];
                assert!(
                    p[0] >= x0 - 1e-6 && p[0] <= x1 + 1e-6 && p[2] >= z0 - 1e-6 && p[2] <= z1 + 1e-6,
                    "stair vertex {p:?} outside the basement rect"
                );
            }
        }
    }
}
