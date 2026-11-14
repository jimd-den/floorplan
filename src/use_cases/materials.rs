//! The material table: one id per structural material, room program,
//! furniture kind, door, and exterior/interior trim.
//!
//! The ids are stable. The first five are structural, then ten
//! programs, then the furniture kinds, then the door; trim and
//! grounds append after. The renderer reads colours from this table
//! and the writers read names, so adding a material is a table
//! change, not a code change.

use crate::domain::mesh::Material;
use crate::domain::model::Program;
use crate::use_cases::door_panel::door_material;
use crate::use_cases::furniture_mesh::furniture_materials;

pub const WALL: u32 = 0;
pub const CEILING: u32 = 1;
pub const ROOF: u32 = 2;
pub const STAIRS: u32 = 3;
pub const PORCH: u32 = 4;
pub const PROGRAM_BASE: u32 = 5;
pub const TRIM: u32 = 29;
pub const SIDING: u32 = 30;
pub const WINDOW_FRAME: u32 = 31;
pub const GLASS: u32 = 32;
pub const BASEBOARD: u32 = 33;
pub const DOOR_FRAME: u32 = 34;
pub const DRIVEWAY: u32 = 35;
pub const TRUNK: u32 = 36;
pub const LEAF: u32 = 37;
pub const CLUTTER: u32 = 38;

/// The full material table, in id order.
pub fn material_table() -> Vec<Material> {
    let mut table = vec![
        Material { name: "wall", color: [0.85, 0.85, 0.85] },
        Material { name: "ceiling", color: [0.95, 0.95, 0.95] },
        Material { name: "roof", color: [0.32, 0.30, 0.28] },
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
    table.extend([
        Material { name: "trim", color: [0.92, 0.92, 0.90] },
        Material { name: "siding", color: [0.72, 0.68, 0.58] },
        Material { name: "window_frame", color: [0.88, 0.88, 0.86] },
        Material { name: "glass", color: [0.55, 0.72, 0.82] },
        Material { name: "baseboard", color: [0.90, 0.88, 0.84] },
        Material { name: "door_frame", color: [0.88, 0.86, 0.82] },
        Material { name: "driveway", color: [0.74, 0.73, 0.70] },
        Material { name: "trunk", color: [0.42, 0.30, 0.20] },
        Material { name: "leaf", color: [0.28, 0.48, 0.26] },
        Material { name: "clutter", color: [0.60, 0.45, 0.35] },
    ]);
    table
}

/// The material id for a room program, in `material_table` order.
pub fn program_material(program: Program) -> u32 {
    const ORDER: [Program; 10] = [
        Program::Living, Program::Kitchen, Program::Dining, Program::Bedroom, Program::Bath,
        Program::Hall, Program::Entry, Program::Study, Program::Laundry, Program::Garage,
    ];
    PROGRAM_BASE + ORDER.iter().position(|&p| p == program).unwrap() as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_id_has_a_material() {
        let table = material_table();
        assert_eq!(table.len(), CLUTTER as usize + 1);
        assert_eq!(table[ROOF as usize].name, "roof");
        assert_eq!(table[SIDING as usize].name, "siding");
        assert_eq!(table[CLUTTER as usize].name, "clutter");
    }

    #[test]
    fn program_ids_follow_the_order() {
        assert_eq!(program_material(Program::Living), PROGRAM_BASE);
        assert_eq!(program_material(Program::Garage), PROGRAM_BASE + 9);
    }
}
