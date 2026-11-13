//! Extrude: House -> Mesh. One mesh, two writers (OBJ, glTF).
//! Slabs, wall boxes with real openings, stairs, a flat roof cap, and furniture.

use crate::domain::mesh::{Box3, Material, Mesh};
use crate::domain::model::{House, Program};
use crate::use_cases::door_panel::{door_material, door_panels};
use crate::use_cases::furniture_mesh::{furniture_boxes, furniture_materials};
use crate::use_cases::walls::{STOREY_H, wall_boxes};

/// Material ids: the first five are structural, then one per program.
pub const WALL: u32 = 0;
pub const CEILING: u32 = 1;
pub const ROOF: u32 = 2;
pub const STAIRS: u32 = 3;
pub const PORCH: u32 = 4;
const PROGRAM_BASE: u32 = 5;

pub fn material_table() -> Vec<Material> {
    let mut table = vec![
        Material { name: "wall", color: [0.85, 0.85, 0.85] },
        Material { name: "ceiling", color: [0.95, 0.95, 0.95] },
        Material { name: "roof", color: [0.35, 0.35, 0.38] },
        Material { name: "stairs", color: [0.55, 0.38, 0.22] },
        Material { name: "porch", color: [0.70, 0.70, 0.68] },
        Material { name: "living", color: [0.90, 0.80, 0.65] },
        Material { name: "kitchen", color: [0.65, 0.75, 0.85] },
        Material { name: "dining", color: [0.70, 0.85, 0.65] },
        Material { name: "bedroom", color: [0.75, 0.70, 0.85] },
        Material { name: "bath", color: [0.60, 0.85, 0.85] },
        Material { name: "hall", color: [0.80, 0.80, 0.80] },
        Material { name: "entry", color: [0.85, 0.55, 0.40] },
        Material { name: "study", color: [0.90, 0.85, 0.60] },
        Material { name: "laundry", color: [0.85, 0.70, 0.75] },
        Material { name: "garage", color: [0.60, 0.60, 0.60] },
    ];
    table.extend(furniture_materials());
    table.push(door_material());
    table
}

/// The material id for a room program, in `material_table` order.
fn program_material(program: Program) -> u32 {
    const ORDER: [Program; 10] = [
        Program::Living, Program::Kitchen, Program::Dining, Program::Bedroom, Program::Bath,
        Program::Hall, Program::Entry, Program::Study, Program::Laundry, Program::Garage,
    ];
    PROGRAM_BASE + ORDER.iter().position(|&p| p == program).unwrap() as u32
}

/// Builds the mesh for a whole house.
pub fn extrude(house: &House) -> Mesh {
    let mut mesh = Mesh::new(material_table());
    for (fi, floor) in house.floors.iter().enumerate() {
        let base = fi as f32 * STOREY_H;
        for room in &floor.rooms {
            let r = room.rect;
            let (x0, z0) = (r.x as f32, r.y as f32);
            let (x1, z1) = (r.right() as f32, r.bottom() as f32);
            mesh.add_box(Box3::new([x0, base - 0.1, z0], [x1, base, z1]), program_material(room.program));
            let ceil = ceiling_material(house, fi, room);
            mesh.add_box(Box3::new([x0, base + STOREY_H, z0], [x1, base + STOREY_H + 0.1, z1]), ceil);
        }
        for (b, m) in wall_boxes(floor, base, WALL) {
            mesh.add_box(b, m);
        }
        for (b, m) in furniture_boxes(floor, base).into_iter().chain(door_panels(floor, base)) { mesh.add_box(b, m); }
        if fi == 0
            && let Some(p) = house.footprint.porch
        {
            mesh.add_box(
                Box3::new([p.x as f32, -0.05, p.y as f32], [p.right() as f32, 0.0, p.bottom() as f32]),
                PORCH,
            );
        }
    }
    for (b, m) in stairs_boxes(house) {
        mesh.add_box(b, m);
    }
    mesh
}

/// The ceiling material for a room: the flat roof cap when nothing
/// sits above it, otherwise the interior ceiling.
fn ceiling_material(house: &House, fi: usize, room: &crate::domain::model::Room) -> u32 {
    if fi == house.floors.len() - 1 {
        return ROOF;
    }
    let covered = house.floors[fi + 1].rooms.iter().any(|r| r.rect.overlaps(&room.rect));
    if covered { CEILING } else { ROOF }
}
/// Stepped boxes in the ground hall: solid steps, 0.25 m treads,
/// 1.0 m wide, rising one storey. Empty for one-storey houses.
fn stairs_boxes(house: &House) -> Vec<(Box3, u32)> {
    let Some((ground_hall, _)) = house.stairs else { return vec![] };
    let Some(hall) = house.floors[0].rooms.iter().find(|r| r.id == ground_hall) else {
        return vec![];
    };
    let r = hall.rect;
    let horizontal = r.w >= r.d;
    let run = if horizontal { r.w } else { r.d };
    let across = if horizontal { r.d } else { r.w };
    let steps = ((run / 0.25).floor() as usize).clamp(2, 12);
    let rise = STOREY_H / steps as f32;
    let tread = run as f32 / steps as f32;
    let width = (across as f32).min(1.0);
    let mut out = vec![];
    for i in 0..steps {
        let top = (i + 1) as f32 * rise;
        let (min, max) = if horizontal {
            let x0 = r.x as f32 + i as f32 * tread;
            let zc = r.y as f32 + r.d as f32 / 2.0;
            ([x0, 0.0, zc - width / 2.0], [x0 + tread, top, zc + width / 2.0])
        } else {
            let z0 = r.y as f32 + i as f32 * tread;
            let xc = r.x as f32 + r.w as f32 / 2.0;
            ([xc - width / 2.0, 0.0, z0], [xc + width / 2.0, top, z0 + tread])
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
    fn the_top_floor_ceiling_is_the_roof() {
        let mesh = extrude(&generate_house(7, Style::Ranch));
        assert!(mesh.materials.contains(&ROOF), "no roof cap");
        let mesh2 = extrude(&generate_house(7, Style::Colonial));
        assert!(mesh2.materials.contains(&ROOF));
        assert!(mesh2.materials.contains(&CEILING), "ground ceiling missing");
    }

    #[test]
    fn an_exposed_ground_ceiling_is_the_roof() {
        // A Colonial with a garage: no upper room sits above the
        // garage, so its ceiling slab at 2.7 m is the roof cap.
        for seed in 0..50u32 {
            let house = generate_house(seed, Style::Colonial);
            if house.footprint.garage.is_none() {
                continue;
            }
            let mesh = extrude(&house);
            let ys: Vec<f32> = mesh.faces.iter().zip(mesh.materials.iter())
                .filter(|(_, m)| **m == ROOF)
                .flat_map(|(f, _)| f.iter().map(|&v| mesh.vertices[v as usize][1]))
                .collect();
            assert!(ys.iter().any(|&y| (y - 2.7).abs() < 0.01), "seed {seed}: no roof at 2.7 m");
            return;
        }
        panic!("no Colonial with a garage in 50 seeds");
    }
}
