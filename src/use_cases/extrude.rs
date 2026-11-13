//! Extrude: House -> Mesh. Slabs, wall boxes with real openings,
//! stairs, a pitched roof, and trim.
//!
//! The mesh is the one 3D representation: the OBJ and glTF writers,
//! the screenshot adapter, and the WASM payload all read it. Walls
//! are siding on the exterior and painted inside; the roof is a
//! pitched gable above an interior ceiling.

use crate::domain::mesh::{Box3, Mesh};
use crate::domain::model::House;
use crate::use_cases::clutter::clutter_boxes;
use crate::use_cases::door_panel::door_panels;
use crate::use_cases::furniture_mesh::furniture_boxes;
use crate::use_cases::lining::add_wall_linings;
use crate::use_cases::materials::{CEILING, SIDING, WALL, program_material};
use crate::use_cases::porch::add_porch;
use crate::use_cases::roof::add_roof;
use crate::use_cases::stairs::stairs_boxes;
use crate::use_cases::trim::add_trim;
use crate::use_cases::walls::{STOREY_H, wall_boxes_toned};

pub use crate::use_cases::materials::{ROOF, STAIRS, material_table};

pub fn extrude(house: &House) -> Mesh {
    let mut mesh = Mesh::new(material_table());
    for (fi, floor) in house.floors.iter().enumerate() {
        let base = fi as f32 * STOREY_H;
        for room in &floor.rooms {
            let r = room.rect;
            let (x0, z0) = (r.x as f32, r.y as f32);
            let (x1, z1) = (r.right() as f32, r.bottom() as f32);
            mesh.add_box(Box3::new([x0, base - 0.1, z0], [x1, base, z1]), program_material(room.program));
            mesh.add_box(Box3::new([x0, base + STOREY_H, z0], [x1, base + STOREY_H + 0.1, z1]), CEILING);
        }
        for (b, m) in wall_boxes_toned(floor, base, WALL, SIDING) {
            mesh.add_box(b, m);
        }
        for (b, m) in furniture_boxes(floor, base)
            .into_iter()
            .chain(door_panels(floor, base))
            .chain(clutter_boxes(floor, base))
        {
            mesh.add_box(b, m);
        }
        add_trim(&mut mesh, floor, base);
        add_wall_linings(&mut mesh, floor, base);
    }
    for (b, m) in stairs_boxes(house) {
        mesh.add_box(b, m);
    }
    add_porch(&mut mesh, house);
    add_roof(&mut mesh, house);
    mesh
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::style::Style;
    use crate::use_cases::generate::generate_house;

    #[test]
    fn extrude_is_deterministic() {
        let a = extrude(&generate_house(42, Style::Ranch));
        let b = extrude(&generate_house(42, Style::Ranch));
        assert_eq!(a.vertices, b.vertices);
        assert_eq!(a.faces, b.faces);
        assert_eq!(a.materials, b.materials);
    }

    #[test]
    fn every_extruded_house_is_a_valid_mesh() {
        for seed in [0u32, 7, 42, 106, 199] {
            for style in Style::all() {
                let mesh = extrude(&generate_house(seed, style));
                assert_eq!(mesh.validate(), Ok(()), "seed {seed} {style:?}");
            }
        }
    }

    #[test]
    fn a_two_storey_house_has_stairs_reaching_the_upper_floor() {
        let house = generate_house(7, Style::Colonial);
        let mesh = extrude(&house);
        let stair_faces: Vec<_> = mesh
            .faces
            .iter()
            .zip(mesh.materials.iter())
            .filter(|(_, m)| **m == STAIRS)
            .collect();
        assert!(!stair_faces.is_empty(), "no stairs geometry");
        let top = stair_faces
            .iter()
            .flat_map(|(f, _)| f.iter().map(|&v| mesh.vertices[v as usize][1]))
            .fold(0.0f32, f32::max);
        assert!((top - STOREY_H).abs() < 0.01, "stairs top at {top}, not {STOREY_H}");
    }

    #[test]
    fn a_one_storey_house_has_no_stairs() {
        let mesh = extrude(&generate_house(7, Style::Ranch));
        assert!(!mesh.materials.contains(&STAIRS));
    }

    #[test]
    fn the_top_floor_has_an_interior_ceiling_and_a_pitched_roof() {
        let mesh = extrude(&generate_house(7, Style::Colonial));
        assert!(mesh.materials.contains(&ROOF), "no pitched roof");
        assert!(mesh.materials.contains(&CEILING), "no interior ceiling");
        let roof_top = mesh.faces.iter().zip(mesh.materials.iter())
            .filter(|(_, m)| **m == ROOF)
            .flat_map(|(f, _)| f.iter().map(|&v| mesh.vertices[v as usize][1]))
            .fold(0.0f32, f32::max);
        assert!(roof_top > STOREY_H, "the roof does not rise above the top storey");
    }

    #[test]
    fn an_exposed_ground_ceiling_is_an_interior_ceiling() {
        // A Colonial with a garage: no upper room sits above the
        // garage, so its ceiling slab at 2.7 m is an interior
        // ceiling, and the pitched roof is above the top storey.
        for seed in 0..50u32 {
            let house = generate_house(seed, Style::Colonial);
            if house.footprint.garage.is_none() {
                continue;
            }
            let mesh = extrude(&house);
            let ys: Vec<f32> = mesh.faces.iter().zip(mesh.materials.iter())
                .filter(|(_, m)| **m == CEILING)
                .flat_map(|(f, _)| f.iter().map(|&v| mesh.vertices[v as usize][1]))
                .collect();
            assert!(ys.iter().any(|&y| (y - 2.7).abs() < 0.01), "seed {seed}: no ceiling at 2.7 m");
            return;
        }
        panic!("no Colonial with a garage in 50 seeds");
    }
}
