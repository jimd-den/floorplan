//! Extrude: House -> Mesh. The foundation slab, wall boxes with
//! real openings, stairs, a pitched roof, and trim.
//!
//! The mesh is the one 3D representation: the OBJ and glTF writers,
//! the screenshot adapter, and the WASM payload all read it. Walls
//! are siding on the exterior and painted inside; the roof is a
//! pitched gable above an interior ceiling.

use crate::domain::mesh::{Box3, Lod, Mesh};
use crate::domain::model::House;
use crate::use_cases::basement_mesh::add_basement;
use crate::use_cases::door_panel::door_panels;
use crate::use_cases::foundation::add_stem_wall;
use crate::use_cases::framing_mesh::add_framing;
use crate::use_cases::lining::add_wall_linings;
use crate::use_cases::materials::{CEILING, CONCRETE, SIDING, WALL, program_material};
use crate::use_cases::porch::{add_garage_door, add_porch};
use crate::use_cases::relief::add_exterior_relief;
use crate::use_cases::roof::{add_roof, add_roof_with_accessories};
use crate::use_cases::stairs::stairs_boxes;
use crate::use_cases::trim::add_trim;
use crate::use_cases::walls::{STOREY_H, wall_boxes_toned};

pub use crate::use_cases::materials::{ROOF, STAIRS, material_table};

pub fn extrude(house: &House) -> Mesh {
    extrude_lod(house, Lod::Full)
}

/// Extrudes `house` at the requested level of detail (`Lod::Full`,
/// `Lod::Simplified`, or `Lod::Massing`) without mutating or
/// regenerating the plan.
pub fn extrude_lod(house: &House, lod: Lod) -> Mesh {
    let mut mesh = Mesh::new(material_table());
    // The foundation first: one concrete slab box per base rect,
    // from the slab thickness below floor level up to floor level.
    // It is part of the envelope, so every LOD carries it.
    for r in &house.base.rects {
        mesh.add_box(
            Box3::new(
                [r.x as f32, -(house.base.thickness as f32), r.y as f32],
                [r.right() as f32, 0.0, r.bottom() as f32],
            ),
            CONCRETE,
        );
    }
    // The stem wall: concrete from the footing to 0.2 m above grade,
    // so the foundation reads as a wall, not a buried slab.
    add_stem_wall(&mut mesh, house);
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
        match lod {
            Lod::Full => {
                for (b, m) in door_panels(floor, base) {
                    mesh.add_box(b, m);
                }
                add_trim(&mut mesh, floor, base);
                add_wall_linings(&mut mesh, floor, base);
                add_exterior_relief(&mut mesh, floor, base);
            }
            Lod::Simplified => {
                for (b, m) in door_panels(floor, base) {
                    mesh.add_box(b, m);
                }
                add_wall_linings(&mut mesh, floor, base);
            }
            Lod::Massing => {}
        }
    }
    // The structural frame and the basement, Full LOD only: studs,
    // plates, joists, rim, and ridge beam, plus the foundation
    // walls, basement slab, and stair down. They are detail, not
    // massing.
    if lod == Lod::Full {
        add_framing(&mut mesh, house);
        add_basement(&mut mesh, house);
    }
    if lod != Lod::Massing {
        for (b, m) in stairs_boxes(house) {
            mesh.add_box(b, m);
        }
    }
    add_porch(&mut mesh, house);
    add_garage_door(&mut mesh, house);
    if lod == Lod::Full {
        add_roof(&mut mesh, house);
    } else {
        add_roof_with_accessories(&mut mesh, house, false);
    }
    mesh
}

