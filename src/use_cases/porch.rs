//! Porch: the front slab, two steps, two posts, and a small roof.
//!
//! The porch is part of the footprint in the house's local frame, so
//! the whole assembly rotates with the house when the neighborhood
//! places it. The porch roof sits below the main eave, which is what
//! makes a front door read as a front door.

use crate::domain::mesh::{Box3, Mesh};
use crate::domain::model::{House, Program};
use crate::use_cases::materials::{DOOR_FRAME, PORCH, ROOF, TRIM};

/// Post thickness, in metres.
const POST: f32 = 0.12;
/// Porch roof underside, in metres.
const ROOF_Y: f32 = 2.35;
/// Porch roof thickness, in metres.
const ROOF_T: f32 = 0.15;

/// Adds the porch assembly when the house has one.
pub fn add_porch(mesh: &mut Mesh, house: &House) {
    let Some(p) = house.footprint.porch else { return };
    let (x0, z0, x1, z1) = (p.x as f32, p.y as f32, p.right() as f32, p.bottom() as f32);
    mesh.add_box(Box3::new([x0, -0.12, z0], [x1, 0.0, z1]), PORCH);
    add_steps(mesh, x0, x1, z1);
    for x in [x0 + 0.1, x1 - 0.1 - POST] {
        mesh.add_box(Box3::new([x, 0.0, z1 - 0.1 - POST], [x + POST, ROOF_Y, z1 - 0.1]), TRIM);
    }
    mesh.add_box(
        Box3::new([x0 - 0.2, ROOF_Y, z0 - 0.2], [x1 + 0.2, ROOF_Y + ROOF_T, z1 + 0.2]),
        ROOF,
    );
}

/// Adds an overhead garage door on the garage room's front wall.
pub fn add_garage_door(mesh: &mut Mesh, house: &House) {
    let Some(floor) = house.floors.first() else { return };
    let Some(garage) = floor.rooms.iter().find(|r| r.program == Program::Garage) else {
        return;
    };
    let g = garage.rect;
    let cx = (g.x + g.w / 2.0) as f32;
    let half_w = ((g.w as f32 - 1.2) / 2.0).max(1.2);
    let z = g.bottom() as f32;
    mesh.add_box(
        Box3::new([cx - half_w - 0.08, 0.0, z + 0.07], [cx + half_w + 0.08, 2.22, z + 0.10]),
        TRIM,
    );
    mesh.add_box(
        Box3::new([cx - half_w, 0.02, z + 0.08], [cx + half_w, 2.14, z + 0.12]),
        DOOR_FRAME,
    );
}

/// Two steps south of the slab, each 0.3 m deep.
fn add_steps(mesh: &mut Mesh, x0: f32, x1: f32, z1: f32) {
    let width = (x1 - x0).min(3.0);
    let sx = (x0 + x1) / 2.0 - width / 2.0;
    for i in 0..2 {
        let top = -0.04 - i as f32 * 0.04;
        let za = z1 + i as f32 * 0.3;
        mesh.add_box(Box3::new([sx, -0.12, za], [sx + width, top, za + 0.3]), PORCH);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::style::Style;
    use crate::use_cases::generate::generate_house;
    use crate::use_cases::materials::material_table;

    fn house_with_porch() -> House {
        for seed in 0..20u32 {
            for style in Style::all() {
                let house = generate_house(seed, style);
                if house.footprint.porch.is_some() {
                    return house;
                }
            }
        }
        panic!("no house with a porch in 20 seeds");
    }

    #[test]
    fn a_porch_gets_steps_posts_and_a_roof() {
        let house = house_with_porch();
        let mut mesh = Mesh::new(material_table());
        add_porch(&mut mesh, &house);
        assert!(mesh.materials.contains(&PORCH), "no porch slab or steps");
        assert!(mesh.materials.contains(&TRIM), "no porch posts");
        assert!(mesh.materials.contains(&ROOF), "no porch roof");
    }

    #[test]
    fn a_house_without_a_porch_adds_nothing() {
        let house = Style::all()
            .iter()
            .map(|style| generate_house(42, *style))
            .find(|house| house.footprint.porch.is_none())
            .expect("no porchless style at seed 42");
        let mut mesh = Mesh::new(material_table());
        add_porch(&mut mesh, &house);
        assert!(mesh.faces.is_empty());
    }
}
