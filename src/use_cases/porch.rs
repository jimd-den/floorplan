//! Porch: the front deck, two posts, and a small roof.
//!
//! The porch is part of the footprint in the house's local frame, so
//! the whole assembly rotates with the house when the neighborhood
//! places it. The porch roof sits below the main eave, which is what
//! makes a front door read as a front door. The deck is at grade, so
//! it takes no steps: steps below grade would be buried, not built.

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
    fn a_porch_gets_a_deck_posts_and_a_roof_and_no_buried_steps() {
        let house = house_with_porch();
        let mut mesh = Mesh::new(material_table());
        add_porch(&mut mesh, &house);
        assert!(mesh.materials.contains(&PORCH), "no porch deck");
        assert!(mesh.materials.contains(&TRIM), "no porch posts");
        assert!(mesh.materials.contains(&ROOF), "no porch roof");
        // Every porch box sits at or above grade: nothing buried.
        assert!(
            mesh.vertices.iter().all(|v| v[1] >= -0.12),
            "a porch box is buried below its slab"
        );
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
